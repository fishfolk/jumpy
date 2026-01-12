#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/87333478?s=200&v=4")]
// This cfg_attr is needed because `rustdoc::all` includes lints not supported on stable
#![cfg_attr(doc, allow(unknown_lints))]
#![deny(rustdoc::all)]
#![allow(clippy::too_many_arguments)]
// TODO: Warn on dead code.
// This is temporarily disabled while migrating to the new bones.
#![allow(dead_code)]
#![allow(ambiguous_glob_reexports)]
#![doc = include_str!("./README.md")]

use bones_bevy_renderer::BonesBevyRenderer;
use bones_framework::prelude::*;

pub mod audio;
pub mod core;
pub mod debug;
pub mod fullscreen;
pub mod input;
pub mod profiler;
pub mod sessions;
pub mod settings;
pub mod ui;

mod prelude {
    pub use crate::{
        audio::*, core::prelude::*, impl_system_param, input::*, sessions::*, settings::*, GameMeta,
    };
    pub use bones_framework::prelude::*;
    pub use once_cell::sync::Lazy;
    pub use serde::{Deserialize, Serialize};
    pub use std::{sync::Arc, time::Duration};
    #[allow(unused)]
    pub use tracing::{debug, error, info, trace, warn};
}
use crate::prelude::*;

// This will cause Bevy to be dynamically linked during development,
// which can greatly reduce re-compile times in some circumstances.
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use bevy_dylib;

#[derive(HasSchema, Clone, Debug, Default)]
#[type_data(metadata_asset("game"))]
#[repr(C)]
pub struct GameMeta {
    pub plugins: SVec<Handle<LuaPlugin>>,
    pub core: CoreMeta,
    pub default_settings: settings::Settings,
    pub localization: Handle<LocalizationAsset>,
    pub theme: ui::UiTheme,
    pub main_menu: ui::main_menu::MainMenuMeta,
    pub music: GameMusic,
    pub network: NetworkMeta,
}

#[derive(HasSchema, Copy, Clone, Debug)]
#[repr(C)]
pub struct NetworkMeta {
    pub max_prediction_window: usize,
    pub local_input_delay: usize,
}

// In wasm build get derivable_impls clippy warning which breaks CI
#[allow(clippy::derivable_impls)]
impl Default for NetworkMeta {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self {
                local_input_delay: 0,
                max_prediction_window: 0,
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self {
                local_input_delay: bones_framework::networking::NETWORK_LOCAL_INPUT_DELAY_DEFAULT,
                max_prediction_window:
                    bones_framework::networking::NETWORK_MAX_PREDICTION_WINDOW_DEFAULT,
            }
        }
    }
}

#[derive(HasSchema, Clone, Debug, Default)]
#[type_data(metadata_asset("assets"))]
#[repr(C)]
pub struct PackMeta {
    pub plugins: SVec<Handle<LuaPlugin>>,
    pub map_tilesets: SVec<Handle<Atlas>>,
    pub players: SVec<Handle<PlayerMeta>>,
    pub player_hats: SVec<Handle<HatMeta>>,
    pub maps: SVec<Handle<MapMeta>>,
    pub map_elements: SVec<Handle<ElementMeta>>,
}

impl GameMeta {
    /// Get the lua plugins loaded by the game.
    pub fn get_plugins(&self, asset_server: &AssetServer) -> Arc<Vec<Handle<LuaPlugin>>> {
        let mut plugins = Vec::new();
        plugins.extend(self.plugins.iter().copied());
        plugins.extend(
            self.core
                .map_elements
                .iter()
                .map(|eh| asset_server.get(*eh).plugin)
                .filter(|plugin_handle| plugin_handle != &Handle::default()),
        );

        for pack in asset_server.packs() {
            let pack_meta = asset_server.get(pack.root.typed::<PackMeta>());
            plugins.extend(pack_meta.plugins.iter().copied());
            plugins.extend(
                pack_meta
                    .map_elements
                    .iter()
                    .map(|eh| asset_server.get(*eh).plugin)
                    .filter(|plugin_handle| plugin_handle != &Handle::default()),
            );
        }
        Arc::new(plugins)
    }
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct GameMusic {
    pub title_screen: Handle<AudioSource>,
    pub fight: SVec<Handle<AudioSource>>,
    pub character_screen: Handle<AudioSource>,
    pub results_screen: Handle<AudioSource>,
    pub credits: Handle<AudioSource>,
}

fn main() {
    // Init logging
    setup_logs!("org", "fishfolk", "jumpy");

    // Initialize the Bevy task pool manually so that we can use it during startup.
    bevy_tasks::IoTaskPool::init(bevy_tasks::TaskPool::new);

    // Register types that we will load from persistent storage.
    settings::Settings::register_schema();

    // First create bones game.
    let mut game = Game::new();

    // Register our game and pack meta types
    GameMeta::register_schema();
    PackMeta::register_schema();

    game
        // Install game plugins
        .install_plugin(DefaultGamePlugin)
        .install_plugin(audio::game_plugin)
        .install_plugin(settings::game_plugin)
        .install_plugin(fullscreen::game_plugin)
        .install_plugin(input::game_plugin)
        .install_plugin(core::game_plugin)
        .install_plugin(debug::game_plugin)
        .install_plugin(profiler::game_plugin)
        .install_plugin(ui::scoring::game_plugin)
        // We initialize the asset server and register asset types
        .init_shared_resource::<AssetServer>()
        .register_default_assets();

    // Create a new session for the game menu. Each session is it's own bones world with it's own
    // plugins, systems, and entities.
    game.sessions.start_menu();

    // Create a new session for the pause menu, which sits in the background by default and only
    // does anything while the game is running.
    game.sessions
        .create_with(SessionNames::PAUSE_MENU, |builder: &mut SessionBuilder| {
            builder.install_plugin(ui::pause_menu::session_plugin);
        });

    // Set priority to ensure pause menu comes before scoring menu (drawn on top)
    game.sessions
        .get_mut(SessionNames::PAUSE_MENU)
        .unwrap()
        .priority = 1;

    // Scoring menu plugin, activated by game between round tarnsitions when appropriate
    game.sessions
        .create_with(SessionNames::SCORING, |builder: &mut SessionBuilder| {
            builder.install_plugin(ui::scoring::session_plugin);
        });

    // session for pop-ups / nofication UI
    game.sessions.create_with(
        SessionNames::NOTIFICATION,
        |builder: &mut SessionBuilder| {
            builder.install_plugin(ui::notification::session_plugin);
        },
    );

    // Create a bevy renderer for the bones game and run it.
    BonesBevyRenderer {
        game,
        pixel_art: true,
        game_version: Version::new(
            env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap(),
            env!("CARGO_PKG_VERSION_MINOR").parse().unwrap(),
            env!("CARGO_PKG_VERSION_PATCH").parse().unwrap(),
        ),
        app_namespace: ("org".into(), "fishfolk".into(), "jumpy".into()),
        asset_dir: std::env::var("JUMPY_ASSETS")
            .unwrap_or_else(|_| "assets".into())
            .into(),
        packs_dir: std::env::var("JUMPY_ASSET_PACKS")
            .unwrap_or_else(|_| "packs".into())
            .into(),
        custom_load_progress: Some(Box::new(load_progress)),
        preload: true,
    }
    .app()
    .run();
}

fn load_progress(assets: &AssetServer, ctx: &egui::Context) {
    let errored = assets.load_progress.errored();
    egui::CentralPanel::default()
        .frame(egui::Frame::default().fill(egui::Color32::from_rgb(0x26, 0x2b, 0x44)))
        .show(ctx, |ui| {
            let height = ui.available_height();
            let ctx = ui.ctx().clone();

            let space_size = 0.03;
            let spinner_size = 0.10;
            let text_size = 0.034;
            ui.vertical_centered(|ui| {
                ui.add_space(height * 0.3);

                if errored > 0 {
                    let err_color = egui::Color32::RED;
                    ui.label(
                        egui::RichText::new("⚠")
                            .color(err_color)
                            .size(height * spinner_size),
                    );
                    ui.add_space(height * space_size);
                    ui.label(
                        egui::RichText::new(format!(
                            "Error loading {errored} asset{}.",
                            if errored > 1 { "s" } else { "" }
                        ))
                        .color(err_color)
                        .size(height * text_size * 0.75),
                    );
                } else {
                    let rect = ui
                        .label(
                            egui::RichText::new("⚓")
                                .color(egui::Color32::WHITE)
                                .size(height * spinner_size),
                        )
                        .rect;
                    egui::Spinner::new().paint_at(ui, rect.expand(spinner_size * height * 0.2));
                    ui.add_space(height * space_size);
                    ui.label(
                        egui::RichText::new("Loading")
                            .color(egui::Color32::WHITE)
                            .size(height * text_size),
                    );
                }
            });

            ctx.data_mut(|d| {
                d.insert_temp(ui.id(), (spinner_size, space_size, text_size));
            })
        });
}
