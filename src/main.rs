use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::future::Future;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::{env, fs};

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
use crate::game::{
    build_state_for_game_mode, spawn_map_objects, GameMode, LOCAL_GAME_STATE_ID,
    NETWORK_GAME_CLIENT_STATE_ID, NETWORK_GAME_HOST_STATE_ID,
};
pub use effects::{ActiveEffectKind, ActiveEffectMetadata, PassiveEffect, PassiveEffectMetadata};
use ff_core::particles::{draw_particles, update_particle_emitters};

pub type CollisionWorld = macroquad_platformer::World;

const CONFIG_FILE_ENV_VAR: &str = "FISHFIGHT_CONFIG";
const ASSETS_DIR_ENV_VAR: &str = "FISHFIGHT_ASSETS";
const MODS_DIR_ENV_VAR: &str = "FISHFIGHT_MODS";

const WINDOW_TITLE: &str = "Fish Fight";

use crate::effects::active::debug_draw_active_effects;
use crate::effects::active::projectiles::fixed_update_projectiles;
use crate::effects::active::triggered::fixed_update_triggered_effects;
use crate::items::MapItemMetadata;
use crate::network::{
    fixed_update_network_client, fixed_update_network_host, update_network_client,
    update_network_host,
};
use crate::player::{
    draw_weapons_hud, spawn_player, update_player_animations, update_player_camera_box,
    update_player_controllers, update_player_events, update_player_inventory,
    update_player_passive_effects, update_player_states, CharacterMetadata, PlayerControllerKind,
    PlayerParams,
};
use crate::sproinger::fixed_update_sproingers;

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

init_resources!(
    "ff_core",
    "json",
    [items::MapItemMetadata, player::CharacterMetadata,]
);

#[ff_core::main(
    crate_name = "ff_core",
    window_title = "Fish Fight",
    config_path_fn = "config_path",
    backend = "macroquad"
)]
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
    use ff_core::macroquad::window::clear_background;
    use ff_core::macroquad::window::next_frame;
    use ff_core::scene;

    use crate::game::Game;

    use ff_core::events::iter_events;

    use gui::MainMenuState;

    {
        let state = MainMenuState::new();
        let game = Game::new(None, Box::new(state))?;
        scene::add_node(game);
    }

    'outer: loop {
        #[allow(clippy::never_loop)]
        for event in iter_events() {
            match event {
                GameEvent::StateTransition(state) => {
                    let mut game = scene::find_node_by_type::<Game>().unwrap();
                    game.set_state(state)?;
                }
                GameEvent::ReloadResources => {
                    reload_resources().await?;
                }
                GameEvent::Quit => break 'outer,
            }
        }

        update_gamepad_context()?;

        next_frame().await;
    }

    scene::clear();

    stop_music();

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

    let map_resource = get_map(0).clone();
    let players = &[
        PlayerParams {
            index: 0,
            controller: PlayerControllerKind::LocalInput(GameInputScheme::KeyboardLeft),
            character: get_character(0),
        },
        PlayerParams {
            index: 1,
            controller: PlayerControllerKind::LocalInput(GameInputScheme::KeyboardLeft),
            character: get_character(1),
        },
    ];

    let mut game_state = Box::new(build_state_for_game_mode(
        GameMode::Local,
        map_resource.map,
        players,
    ));

    'outer: loop {
        let mut last_update = Instant::now();
        let mut last_fixed_update = Instant::now();
        let mut last_draw = Instant::now();

        let mut fixed_accumulator = 0.0;

        'inner: loop {
            let now = Instant::now();

            #[allow(clippy::never_loop)]
            for event in iter_events() {
                match event {
                    GameEvent::StateTransition(state) => {
                        let world = game_state.end()?;

                        game_state = state;
                        game_state.begin(world)?;

                        break 'inner;
                    }
                    GameEvent::ReloadResources => reload_resources().await,
                    GameEvent::Quit => break 'outer,
                }
            }

            update_gamepad_context()?;

            let delta_time = now.duration_since(last_update).as_secs_f32();

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

                    last_draw = now;
                }
            }
        }

        stop_music();
    }

    Ok(())
}
