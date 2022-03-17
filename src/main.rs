use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::{env, fs};
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::Instant;

use fishsticks::GamepadContext;

//use ultimate::UltimateApi;

use ff_core::prelude::*;

#[cfg(not(feature = "ultimate"))]
pub mod editor;
#[cfg(not(feature = "ultimate"))]
mod gui;

#[cfg(not(feature = "ultimate"))]
use ff_core::gui::GuiTheme;

pub mod debug;
pub mod effects;
pub mod game;
pub mod items;
pub mod network;
pub mod particles;
pub mod physics;
pub mod player;
pub mod sproinger;

pub use physics::*;

// use network::api::Api;

#[cfg(not(feature = "ultimate"))]
use editor::{Editor, EditorCamera};

use ff_core::map::{Map, MapLayerKind, MapObjectKind};

use ff_core::Result;

pub use ff_core::Config;
pub use items::Item;

pub use ff_core::prelude::*;

pub use game::GameCamera;

pub use player::PlayerEvent;

use crate::effects::passive::init_passive_effects;
use crate::game::{create_main_game_state, GameMode, spawn_map_objects};
use ff_core::particles::{draw_particles, update_particle_emitters};
pub use effects::{
    ActiveEffectKind, ActiveEffectMetadata, PassiveEffectInstance, PassiveEffectMetadata,
};

pub type CollisionWorld = macroquad_platformer::World;

const CONFIG_FILE_ENV_VAR: &str = "FISHFIGHT_CONFIG";
const ASSETS_DIR_ENV_VAR: &str = "FISHFIGHT_ASSETS";
const MODS_DIR_ENV_VAR: &str = "FISHFIGHT_MODS";

const WINDOW_TITLE: &str = "Fish Fight";

use crate::effects::active::debug_draw_active_effects;
use crate::effects::active::projectiles::fixed_update_projectiles;
use crate::effects::active::triggered::fixed_update_triggered_effects;
use crate::items::{MapItemMetadata};
use crate::sproinger::fixed_update_sproingers;
use crate::network::{fixed_update_network_client, fixed_update_network_host, update_network_client, update_network_host};
use crate::player::{CharacterMetadata, draw_weapons_hud, PlayerControllerKind, PlayerParams, spawn_player, update_player_animations, update_player_camera_box, update_player_controllers, update_player_events, update_player_inventory, update_player_passive_effects, update_player_states};

fn config_path() -> String {
    let path = env::var(CONFIG_FILE_ENV_VAR)
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            #[cfg(debug_assertions)]
            return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.toml");
            #[cfg(not(debug_assertions))]
            return PathBuf::from("config.toml");
        });

    path.to_string_lossy().to_string()
}

init_resources!("ff_core", "json", [
    items::MapItemMetadata,
    player::CharacterMetadata,
]);

#[ff_core::main(crate_name = "ff_core", window_title = "Fish Fight", config_path_fn = "config_path", backend = "macroquad")]
async fn main() -> Result<()> {
    let assets_dir = env::var(ASSETS_DIR_ENV_VAR).unwrap_or_else(|_| "./assets".to_string());
    let mods_dir = env::var(MODS_DIR_ENV_VAR).unwrap_or_else(|_| "./mods".to_string());

    ff_core::rand::srand(0);

    load_resources(&assets_dir, &mods_dir).await?;

    init_passive_effects();

    init_gamepad_context().await?;

    #[cfg(feature = "ultimate")]
    ultimate_main().await?;
    #[cfg(not(feature = "ultimate"))]
    macroquad_main().await?;
    
    Ok(())
}

#[cfg(not(feature = "ultimate"))]
async fn macroquad_main() -> Result<()> {
    use ff_core::scene;

    use ff_core::macroquad::window::next_frame;

    use crate::game::Game;

    use ff_core::events::iter_events;
    use gui::MainMenuResult;

    'outer: loop {
        match gui::show_main_menu().await {
            MainMenuResult::LocalGame { map, players } => {
                let game = Game::new(GameMode::Local, *map, &players);
                scene::add_node(game);

                play_sound("fish_tide", true);
            }
            MainMenuResult::Editor {
                is_new_map,
            } => {
                let map_resource = if is_new_map {
                    let res = gui::show_create_map_menu().await?;
                    if res.is_none() {
                        continue 'outer;
                    }

                    res.unwrap()
                } else {
                    gui::show_select_map_menu().await
                };

                let position = Vec2::from(map_resource.map.get_size()) * 0.5;

                scene::add_node(EditorCamera::new(position));
                scene::add_node(Editor::new(map_resource));
            }
            MainMenuResult::ReloadResources => {
                unimplemented!("Resource reloading is unimplemented");
                //reload_resources();
                continue 'outer;
            }
            MainMenuResult::Credits => {
                play_sound("thanks_for_all_the_fished", true);

                gui::show_game_credits(assets_dir()).await;

                stop_music();

                continue 'outer;
            }
            MainMenuResult::Quit => {
                GameEvent::Quit.dispatch();
            }
        };

        'inner: loop {
            #[allow(clippy::never_loop)]
            for event in iter_events() {
                match event {
                    GameEvent::MainMenu => break 'inner,
                    GameEvent::Quit => break 'outer,
                    _ => {},
                }
            }

            update_gamepad_context()?;

            next_frame().await;
        }

        scene::clear();

        stop_music();
    }

    Ok(())
}

#[cfg(feature = "ultimate")]
async fn ultimate_main() -> Result<()> {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    const FIXED_DELTA_TIME: f32 = 1.0 / 120.0;

    let draw_delta_time = 1.0 / config.rendering.max_fps.unwrap_or(0) as f32;

    let mut window = create_window(WINDOW_TITLE, None, &config.window)?;

    // let mut api = UltimateApi::init().await.unwrap();

    let mut game_states = HashMap::new();

    game_states.insert("main".to_string(), create_main_game_state(GameMode::NetworkHost));

    let mut current_game_state = "main".to_string();

    'outer: loop {
        let mut game_state = game_states
            .get_mut(&current_game_state)
            .unwrap();

        let mut last_update = Instant::now();
        let mut last_fixed_update = Instant::now();
        let mut last_draw = Instant::now();

        let mut fixed_accumulator = 0.0;

        'inner: loop {
            let now = Instant::now();

            #[allow(clippy::never_loop)]
            for event in iter_events() {
                match event {
                    GameEvent::ModeTransition(id) => {
                        current_game_state = id;
                        break 'inner;
                    },
                    GameEvent::Quit => break 'outer,
                }
            }

            update_gamepad_context()?;

            let delta_time = now.duration_since(last_update).as_secs_f32();

            #[cfg(debug_assertions)]
            if is_key_pressed(KeyCode::U) {
                crate::debug::toggle_debug_draw();
            }

            let gamepad_ctx = get_gamepad_context();
            if is_key_pressed(macroquad::prelude::KeyCode::Escape)
                || is_gamepad_btn_pressed(Some(&gamepad_ctx), Button::Start)
            {
                gui::toggle_game_menu();
            }

            game_state.update(delta_time);

            last_update = now;

            fixed_accumulator += delta_time;

            while fixed_accumulator >= FIXED_DELTA_TIME {
                fixed_accumulator -= FIXED_DELTA_TIME;

                let integration_factor = if fixed_accumulator >= FIXED_DELTA_TIME {
                    1.0
                } else {
                    fixed_accumulator / FIXED_DELTA_TIME
                };

                game_state.fixed_update(FIXED_DELTA_TIME, integration_factor);

                last_fixed_update = now;
            }

            {
                let draw_dt = now.duration_since(last_draw).as_secs_f32();

                if draw_dt >= draw_delta_time {
                    let mut camera = storage::get_mut::<GameCamera>();
                    camera.update();

                    {
                        let map = storage::get::<Map>();
                        map.draw(None, true);
                    }

                    game_state.draw();

                    #[cfg(not(feature = "ultimate"))]
                    if gui::is_game_menu_open() {
                        if let Some(res) = gui::draw_game_menu(&mut *root_ui()) {
                            match res.into_usize() {
                                GAME_MENU_RESULT_MAIN_MENU => exit_to_main_menu(),
                                GAME_MENU_RESULT_QUIT => quit_to_desktop(),
                                _ => {}
                            }
                        }
                    }

                    last_draw = now;
                }
            }
        }

        stop_music();
    }

    Ok(())
}