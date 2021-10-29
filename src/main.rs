use fishsticks::GamepadContext;

use std::env;
use std::path::PathBuf;

use macroquad::{
    experimental::{collections::storage, coroutines::start_coroutine},
    prelude::*,
};

mod capabilities;
pub mod components;
pub mod config;
mod decoration;
pub mod editor;
mod gui;
mod items;
pub mod json;
pub mod map;
pub mod math;
mod noise;
pub mod resources;
pub mod text;
#[macro_use]
pub mod error;
#[cfg(debug_assertions)]
pub mod debug;
pub mod effects;
pub mod events;
pub mod game;
pub mod particles;
pub mod player;

pub mod input;

pub use input::is_gamepad_btn_pressed;

use editor::{Editor, EditorCamera, EditorInputScheme};

pub use error::{Error, Result};

use map::{Map, MapLayerKind, MapObjectKind};

pub use config::Config;
pub use items::{EquippedItem, Item, Sproinger, Weapon};

pub use events::{dispatch_application_event, ApplicationEvent};

pub use game::{
    collect_input, create_game_scene, start_music, stop_music, GameCamera, GameInput,
    GameInputScheme, GameScene, GameWorld, LocalGame, NetworkGame, NetworkMessage,
};

pub use particles::ParticleEmitters;

pub use resources::Resources;

pub use player::{Player, PlayerEventParams};

pub use decoration::Decoration;

use crate::effects::passive::init_passive_effects;
pub use effects::{
    ActiveEffectCoroutine, ActiveEffectKind, ActiveEffectParams, ParticleControllers,
    PassiveEffectInstance, PassiveEffectParams, Projectiles, TriggeredEffects,
};

pub type CollisionWorld = macroquad_platformer::World;

const ASSETS_DIR_ENV_VAR: &str = "FISHFIGHT_ASSETS";
const CONFIG_FILE_ENV_VAR: &str = "FISHFIGHT_CONFIG";

const WINDOW_TITLE: &str = "FishFight";

/// Exit to main menu
pub fn exit_to_main_menu() {
    ApplicationEvent::MainMenu.dispatch();
}

/// Quit to desktop
pub fn quit_to_desktop() {
    ApplicationEvent::Quit.dispatch()
}

fn window_conf() -> Conf {
    let config = Config::load(
        env::var(CONFIG_FILE_ENV_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                #[cfg(debug_assertions)]
                return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.json");
                #[cfg(not(debug_assertions))]
                return PathBuf::from("./config.json");
            }),
    )
    .unwrap();

    storage::store(config.clone());

    Conf {
        window_title: WINDOW_TITLE.to_owned(),
        high_dpi: config.high_dpi,
        fullscreen: config.fullscreen,
        window_width: config.resolution.width,
        window_height: config.resolution.height,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() -> Result<()> {
    use events::iter_events;
    use gui::MainMenuResult;

    let assets_dir = env::var(ASSETS_DIR_ENV_VAR).unwrap_or_else(|_| "./assets".to_string());

    {
        let gui_resources = gui::GuiResources::load(&assets_dir).await;
        storage::store(gui_resources);
    }

    rand::srand(0);

    let resources_loading = start_coroutine({
        let assets_dir = assets_dir.clone();
        async move {
            let resources = match Resources::new(&assets_dir).await {
                Ok(val) => val,
                Err(err) => panic!("{}: {}", err.kind().as_str(), err),
            };

            storage::store(resources);
        }
    });

    while !resources_loading.is_done() {
        clear_background(BLACK);
        draw_text(
            &format!(
                "Loading resources {}",
                ".".repeat(((get_time() * 2.0) as usize) % 4)
            ),
            screen_width() / 2.0 - 160.0,
            screen_height() / 2.0,
            40.,
            WHITE,
        );

        next_frame().await;
    }

    {
        let gamepad_system = fishsticks::GamepadContext::init().unwrap();
        storage::store(gamepad_system);
    }

    init_passive_effects();

    'outer: loop {
        match gui::show_main_menu().await {
            MainMenuResult::LocalGame(player_input) => {
                let map_resource = gui::show_select_map_menu().await;

                assert_eq!(
                    player_input.len(),
                    2,
                    "Local: There should be two player input schemes for this game mode"
                );

                let players = create_game_scene(map_resource.map, true);
                scene::add_node(LocalGame::new(player_input, players[0], players[1]));

                start_music("fish_tide");
            }
            MainMenuResult::Editor {
                input_scheme,
                is_new_map,
            } => {
                let map_resource = if is_new_map {
                    gui::show_create_map_menu().await?
                } else {
                    gui::show_select_map_menu().await
                };

                let position = map_resource.map.get_size() * 0.5;

                scene::add_node(EditorCamera::new(position));
                scene::add_node(Editor::new(input_scheme, map_resource));
            }
            MainMenuResult::Quit => {
                quit_to_desktop();
            }
        };

        'inner: loop {
            #[allow(clippy::never_loop)]
            for event in iter_events() {
                match event {
                    ApplicationEvent::MainMenu => break 'inner,
                    ApplicationEvent::Quit => break 'outer,
                }
            }

            {
                let mut gamepad_system = storage::get_mut::<GamepadContext>();
                gamepad_system.update()?;
            }

            next_frame().await;
        }

        scene::clear();
        stop_music();
    }

    Ok(())
}
