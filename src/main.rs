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
pub mod platform;
pub mod settings;

mod prelude {
    pub use crate::{core::prelude::*, impl_system_param, input::*, GameMeta};
    pub use bones_framework::prelude::*;
    pub use once_cell::sync::Lazy;
    pub use serde::{Deserialize, Serialize};
    pub use std::{sync::Arc, time::Duration};
    pub use tracing::{debug, error, info, trace, warn};
}
use crate::prelude::*;

// // This will cause Bevy to be dynamically linked during development,
// // which can greatly reduce re-compile times in some circumstances.
// #[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
// #[allow(unused_imports)]
// #[allow(clippy::single_component_path_imports)]
// use bevy_dylib;

#[derive(HasSchema, Clone, Debug, Default)]
#[type_data(metadata_asset("game"))]
#[repr(C)]
pub struct GameMeta {
    pub core: CoreMeta,
    pub default_settings: settings::Settings,
    pub localization: Handle<LocalizationAsset>,
}

fn main() {
    // Initialize the Bevy task pool manually so that we can use it during startup.
    bevy_tasks::IoTaskPool::init(bevy_tasks::TaskPool::new);

    // First create bones game.
    let mut game = Game::new();

    game
        // Install game plugins
        .install_plugin(platform::game_plugin)
        .install_plugin(core::game_plugin)
        // We initialize the asset server and register asset types
        .init_shared_resource::<AssetServer>()
        .register_default_assets()
        .register_asset::<GameMeta>();

    // Create a new session for the game menu. Each session is it's own bones world with it's own
    // plugins, systems, and entities.
    game.sessions
        .create("menu")
        // Install the default bones_framework plugin for this session
        .install_plugin(DefaultSessionPlugin)
        // Add our menu system to the update stage
        .add_system_to_stage(Update, menu_system);

    // Create a bevy renderer for the bones game and run it.
    BonesBevyRenderer::new(game).app().run();
}

/// System to render the home menu.
fn menu_system(
    meta: Root<GameMeta>,
    assets: Res<AssetServer>,
    ctx: Egui,
    mut sessions: ResMutInit<Sessions>,
    mut session_options: ResMutInit<SessionOptions>,
) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        if ui.button("Start Game").clicked() {
            session_options.delete = true;

            let session = sessions.create("game");
            session.install_plugin(core::MatchPlugin {
                map: assets.get(meta.core.stable_maps[3]).clone(),
                selected_players: [Some(meta.core.players[0]), None, None, None],
            });
        }
    });
}
