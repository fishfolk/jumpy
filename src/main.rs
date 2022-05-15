extern crate core;

use std::env;
use std::path::PathBuf;

#[cfg(not(feature = "macroquad"))]
use ff_core::map::get_map;

//use ultimate::UltimateApi;

#[cfg(not(feature = "macroquad"))]
use ff_core::prelude::*;

#[cfg(feature = "macroquad")]
pub mod editor;

pub mod gui;

pub mod camera;
pub mod critters;
pub mod debug;
pub mod effects;
pub mod game;
pub mod items;
pub mod network;
pub mod player;
pub mod sproinger;

// use network::api::Api;

use ff_core::map::{Map, MapLayerKind, MapObjectKind};

pub use ff_core::config::Config;
pub use items::Item;

pub use ff_core::prelude::*;

pub use game::Camera;

pub use player::character::get_character;
pub use player::PlayerEvent;

use crate::effects::passive::init_passive_effects;
use crate::game::{build_state_for_game_mode, GameMode};
pub use effects::{ActiveEffectKind, ActiveEffectMetadata, PassiveEffect, PassiveEffectMetadata};
use ff_core::gui::rebuild_gui_theme;

const CONFIG_FILE_ENV_VAR: &str = "FISHFIGHT_CONFIG";
const ASSETS_DIR_ENV_VAR: &str = "FISHFIGHT_ASSETS";
const MODS_DIR_ENV_VAR: &str = "FISHFIGHT_MODS";

#[allow(dead_code)]
const WINDOW_TITLE: &str = "Fish Fight";

pub fn config_path() -> String {
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

#[cfg_attr(
    feature = "macroquad",
    ff_core::async_main(
        core_rename = "ff_core",
        window_title = "Fish Fight",
        config_path_fn = "config_path",
        custom_resources = "[items::MapItemMetadata, player::CharacterMetadata]",
        backend = "macroquad"
    )
)]
#[cfg_attr(
    not(feature = "macroquad"),
    ff_core::async_main(
        core_rename = "ff_core",
        custom_resources = "[items::MapItemMetadata, player::CharacterMetadata]",
        backend = "internal"
    )
)]
async fn main() -> Result<()> {
    let assets_dir = env::var(ASSETS_DIR_ENV_VAR).unwrap_or_else(|_| "assets/".to_string());
    let mods_dir = env::var(MODS_DIR_ENV_VAR).unwrap_or_else(|_| "mods/".to_string());

    init_core(0, assets_dir.as_str(), mods_dir.as_str()).await?;

    ff_core::cfg_if! {
        if #[cfg(feature = "macroquad")] {
            macroquad_main().await?;
        } else if #[cfg(feature = "ultimate")] {
            ultimate_main().await?;
        } else {
            internal_main().await?;
        }
    }

    Ok(())
}

#[cfg(not(any(feature = "macroquad", feature = "ultimate")))]
async fn internal_main() -> Result<()> {
    use ff_core::gl::init_gl_context;
    use ff_core::glutin::event_loop;

    let config = load_config(config_path()).await?;

    let event_loop = new_event_loop();

    create_context(WINDOW_TITLE, &event_loop, &config).await?;

    load_resources().await?;

    init_passive_effects();

    let map_resource = get_map(0).clone();
    let players = &[
        PlayerParams {
            index: 0,
            controller: PlayerControllerKind::LocalInput(GameInputScheme::KeyboardLeft),
            character: get_character(0).clone(),
        },
        PlayerParams {
            index: 1,
            controller: PlayerControllerKind::LocalInput(GameInputScheme::KeyboardRight),
            character: get_character(1).clone(),
        },
    ];

    let initial_state = build_state_for_game_mode(GameMode::Local, map_resource.map, players)?;

    //let initial_state = MainMenuState::new();

    Game::new(initial_state)
        .with_config(config)
        .with_event_loop(event_loop)
        .with_event_handler(DefaultEventHandler)
        .with_clear_color(colors::BLACK)
        .run()
        .await?;

    Ok(())
}

#[cfg(feature = "ultimate")]
async fn ultimate_main() -> Result<()> {
    use ff_core::gl::init_gl_context;
    use ff_core::glutin::event_loop;

    let config = load_config(config_path()).await?;

    let event_loop = new_event_loop();

    {
        let window = create_window(&game.window_title, &event_loop, &game.config)?;
        let _ = init_gl_context(window);
    }

    load_resources().await?;

    init_passive_effects();

    init_gamepad_context().await?;

    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // let mut api = UltimateApi::init().await.unwrap();

    let map_resource = get_map(0).clone();
    let players = &[
        PlayerParams {
            index: 0,
            controller: PlayerControllerKind::LocalInput(GameInputScheme::KeyboardLeft),
            character: get_character(0).clone(),
        },
        PlayerParams {
            index: 1,
            controller: PlayerControllerKind::LocalInput(GameInputScheme::KeyboardLeft),
            character: get_character(1).clone(),
        },
    ];

    let initial_state = build_state_for_game_mode(GameMode::Local, map_resource.map, players)?;

    Game::new(initial_state)
        .with_config(config)
        .with_event_loop(event_loop)
        .with_event_handler(DefaultEventHandler)
        .run()
        .await?;

    Ok(())
}

#[cfg(feature = "macroquad")]
async fn macroquad_main() -> Result<()> {
    load_resources().await?;

    rebuild_gui_theme();

    init_passive_effects();

    init_gamepad_context().await?;

    use ff_core::macroquad::experimental::scene;

    use gui::MainMenuState;

    {
        let _camera = Camera::default();

        let game = Game::new(MainMenuState::new())?;

        scene::add_node(game);
    }

    'outer: loop {
        #[allow(clippy::never_loop)]
        for event in iter_events() {
            match event {
                Event::StateTransition(state) => {
                    let mut game = scene::find_node_by_type::<Game>().unwrap();
                    game.change_state(state)?;
                }
                Event::Quit => break 'outer,
                _ => {}
            }
        }

        update_gamepad_context()?;

        clear_screen(None);

        end_frame().await;
    }

    scene::clear();

    stop_music();

    Ok(())
}
