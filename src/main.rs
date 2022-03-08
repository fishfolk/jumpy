use std::borrow::BorrowMut;
use std::collections::HashMap;
use std::env;
use std::future::Future;
use std::path::PathBuf;
use std::time::Instant;

use fishsticks::GamepadContext;

//use ultimate::UltimateApi;

use core::prelude::*;

#[cfg(not(feature = "ultimate"))]
pub mod editor;
#[cfg(not(feature = "ultimate"))]
mod gui;

#[cfg(feature = "ultimate")]
mod gui {
    mod background;
}

pub mod debug;
pub mod effects;
pub mod game;
mod items;
pub mod json;
pub mod map;
pub mod network;
pub mod particles;
pub mod physics;
pub mod player;
pub mod resources;

pub mod drawables;

pub use drawables::*;
pub use physics::*;

// use network::api::Api;

#[cfg(not(feature = "ultimate"))]
use editor::{Editor, EditorCamera};

use map::{Map, MapLayerKind, MapObjectKind};

use core::Result;

pub use core::Config;
pub use items::Item;

pub use core::prelude::*;

pub use game::{start_music, stop_music, GameCamera};

pub use resources::Resources;

pub use player::PlayerEvent;

use crate::effects::passive::init_passive_effects;
use crate::game::{create_main_game_state, GameMode, spawn_map_objects};
use crate::particles::{draw_particles, Particles, update_particle_emitters};
use crate::resources::load_resources;
pub use effects::{
    ActiveEffectKind, ActiveEffectMetadata, PassiveEffectInstance, PassiveEffectMetadata,
};

pub type CollisionWorld = macroquad_platformer::World;

const CONFIG_FILE_ENV_VAR: &str = "FISHFIGHT_CONFIG";
const ASSETS_DIR_ENV_VAR: &str = "FISHFIGHT_ASSETS";
const MODS_DIR_ENV_VAR: &str = "FISHFIGHT_MODS";

const WINDOW_TITLE: &str = "Fish Fight";

async fn setup(assets_dir: &str, mods_dir: &str) -> Result<()> {
    core::rand::srand(0);

    load_resources(&assets_dir, &mods_dir).await?;

    {
        let gamepad_context = fishsticks::GamepadContext::init().unwrap();
        storage::store(gamepad_context);
    }

    {
        let particles = Particles::new();
        storage::store(particles);
    }

    init_passive_effects();

    Ok(())
}

use crate::effects::active::debug_draw_active_effects;
use crate::effects::active::projectiles::fixed_update_projectiles;
use crate::effects::active::triggered::fixed_update_triggered_effects;
use crate::map::fixed_update_sproingers;
use crate::network::{fixed_update_network_client, fixed_update_network_host, update_network_client, update_network_host};
use crate::player::{draw_weapons_hud, PlayerControllerKind, PlayerParams, spawn_player, update_player_animations, update_player_camera_box, update_player_controllers, update_player_events, update_player_inventory, update_player_passive_effects, update_player_states};


cfg_if! {
    if #[cfg(not(feature = "ultimate"))] {
        use core::macroquad;

        use core::scene;

        use macroquad::window::{Conf, next_frame};

        use crate::game::Game;

        fn get_window_conf() -> Conf {
            let path = env::var(CONFIG_FILE_ENV_VAR)
                .map(PathBuf::from)
                .unwrap_or_else(|_| {
                    #[cfg(debug_assertions)]
                    return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.toml");
                    #[cfg(not(debug_assertions))]
                    return PathBuf::from("./config.toml");
                });

            let config = load_config_file_sync(&path).unwrap();

            storage::store(config.clone());

            use core::video::resolutions::HD720;

            let (fullscreen, window_width, window_height) = match config.window.mode {
                WindowMode::Windowed { size } => (false, size.width as i32, size.height as i32),
                WindowMode::Borderless => (true, HD720.width as i32, HD720.height as i32),
            };

            Conf {
                window_title: WINDOW_TITLE.to_owned(),
                high_dpi: config.window.is_high_dpi,
                fullscreen,
                window_width,
                window_height,
                ..Default::default()
            }
        }

        #[macroquad::main(get_window_conf)]
        async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
            use core::events::iter_events;
            use gui::MainMenuResult;

            let assets_dir = env::var(ASSETS_DIR_ENV_VAR).unwrap_or_else(|_| "./assets".to_string());
            let mods_dir = env::var(MODS_DIR_ENV_VAR).unwrap_or_else(|_| "./mods".to_string());

            setup(&assets_dir, &mods_dir).await?;

            'outer: loop {
                match gui::show_main_menu().await {
                    MainMenuResult::LocalGame { map, players } => {
                        let game = Game::new(GameMode::Local, *map, &players);
                        scene::add_node(game);

                        start_music("fish_tide");
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
                        start_music("thanks_for_all_the_fished");

                        gui::show_game_credits(&assets_dir).await;

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

                    {
                        let mut gamepad_context = storage::get_mut::<GamepadContext>();
                        gamepad_context.update()?;
                    }

                    next_frame().await;
                }

                scene::clear();

                stop_music();
            }

            Ok(())
        }
    }
}

#[cfg(feature = "ultimate")]
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let assets_dir = env::var(ASSETS_DIR_ENV_VAR).unwrap_or_else(|_| "./assets".to_string());
    let mods_dir = env::var(MODS_DIR_ENV_VAR).unwrap_or_else(|_| "./mods".to_string());

    setup(&assets_dir, &mods_dir).await?;

    //console_error_panic_hook::set_once();

    const FIXED_DELTA_TIME: f32 = 1.0 / 120.0;

    let config = {
        let path = env::var(CONFIG_FILE_ENV_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                #[cfg(debug_assertions)]
                return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config.toml");
                #[cfg(not(debug_assertions))]
                return PathBuf::from("./config.toml");
            });

        load_config_file(&path).await?
    };

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

            {
                let mut gamepad_context = storage::get_mut::<GamepadContext>();
                gamepad_context.update()?;
            }

            let delta_time = now.duration_since(last_update).as_secs_f32();

            #[cfg(debug_assertions)]
            if is_key_pressed(KeyCode::U) {
                crate::debug::toggle_debug_draw();
            }

            #[cfg(not(feature = "ultimate"))]
            {
                let gamepad_context = storage::get::<GamepadContext>();
                if is_key_pressed(macroquad::prelude::KeyCode::Escape)
                    || is_gamepad_btn_pressed(Some(&gamepad_context), Button::Start)
                {
                    gui::toggle_game_menu();
                }
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