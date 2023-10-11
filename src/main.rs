#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/87333478?s=200&v=4")]
// This cfg_attr is needed because `rustdoc::all` includes lints not supported on stable
#![cfg_attr(doc, allow(unknown_lints))]
#![deny(rustdoc::all)]
#![allow(clippy::too_many_arguments)]
// TODO: Warn on dead code.
// This is temporarily disabled while migrating to the new bones.
#![allow(dead_code)]
#![allow(ambiguous_glob_reexports)]

use bones_bevy_renderer::BonesBevyRenderer;
use bones_framework::prelude::*;

pub mod core;
pub mod input;
pub mod sessions;
pub mod settings;
pub mod ui;

mod prelude {
    pub use crate::{core::prelude::*, impl_system_param, input::*, sessions::*, GameMeta};
    pub use bones_framework::prelude::*;
    pub use once_cell::sync::Lazy;
    pub use serde::{Deserialize, Serialize};
    pub use std::{sync::Arc, time::Duration};
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
    pub core: CoreMeta,
    pub default_settings: settings::Settings,
    pub localization: Handle<LocalizationAsset>,
    pub theme: ui::UiTheme,
    pub main_menu: ui::main_menu::MainMenuMeta,
}

fn main() {
    // Initialize the Bevy task pool manually so that we can use it during startup.
    bevy_tasks::IoTaskPool::init(bevy_tasks::TaskPool::new);

    // Register types that we will load from persistent storage.
    settings::Settings::schema();

    // First create bones game.
    let mut game = Game::new();

    // Register our game asset type
    GameMeta::schema();

    game
        // Install game plugins
        .install_plugin(DefaultGamePlugin)
        .install_plugin(settings::game_plugin)
        .install_plugin(input::game_plugin)
        .install_plugin(core::game_plugin)
        // We initialize the asset server and register asset types
        .init_shared_resource::<AssetServer>()
        .register_default_assets();

    // Create a new session for the game menu. Each session is it's own bones world with it's own
    // plugins, systems, and entities.
    game.sessions.start_menu();

    // Create a new session for the pause menu, which sits in the background by default and only
    // does anything while the game is running.
    game.sessions
        .create(SessionNames::PAUSE_MENU)
        .install_plugin(ui::pause_menu::session_plugin);

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
        asset_dir: "assets".into(),
        packs_dir: "packs".into(),
    }
    .app()
    .run();
}
