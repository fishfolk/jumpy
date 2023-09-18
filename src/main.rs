#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/87333478?s=200&v=4")]
// This cfg_attr is needed because `rustdoc::all` includes lints not supported on stable
#![cfg_attr(doc, allow(unknown_lints))]
#![deny(rustdoc::all)]
#![allow(clippy::too_many_arguments)]
// TODO: Warn on dead code.
// This is temporarily disabled while migrating to the new bones.
#![allow(dead_code)]

use bones_bevy_renderer::BonesBevyRenderer;
use bones_framework::prelude::*;

mod core;

mod prelude {
    pub use crate::{core::prelude::*, impl_system_param, GameMeta};
    pub use bones_framework::prelude::*;
    pub use once_cell::sync::Lazy;
    pub use serde::{Deserialize, Serialize};
    pub use std::sync::Arc;
    pub use tracing::{debug, error, info, trace, warn};
}
use crate::prelude::*;

#[derive(HasSchema, Clone, Debug, Default)]
#[type_data(metadata_asset("game"))]
#[repr(C)]
pub struct GameMeta {
    pub core: CoreMeta,
}

#[derive(HasSchema, Clone, Default, Debug)]
#[repr(C)]
#[type_data(metadata_asset("dummy"))]
pub struct Dummy;

fn main() {
    // First create bones game.
    let mut game = Game::new();

    game
        // We initialize the asset server.
        .init_shared_resource::<AssetServer>()
        // We must register all of our asset types before they can be loaded.
        // TODO: Evaluate ways to decentralize asset registration.
        // We want to see if there's a way to avoid putting all of our asset types listed in main.
        .register_default_assets()
        .register_asset::<GameMeta>()
        .register_asset::<PlayerMeta>()
        .register_asset::<AudioSource>()
        .register_asset::<HatMeta>()
        .register_asset::<MapMeta>()
        .register_asset::<ElementMeta>()
        .register_asset::<FishSchoolMeta>()
        .register_asset::<KickBombMeta>()
        .register_asset::<AnimatedDecorationMeta>()
        .register_asset::<PlayerSpawner>()
        .register_asset::<SwordMeta>();

    // Create a new session for the game menu. Each session is it's own bones world with it's own
    // plugins, systems, and entities.
    game.sessions
        .create("menu")
        // Install the default bones_framework plugin for this session
        .install_plugin(DefaultPlugin)
        // Add our menu system to the update stage
        .add_system_to_stage(Update, menu_system);

    // Create a bevy renderer for the bones game and run it.
    BonesBevyRenderer::new(game).app().run();
}

/// System to render the home menu.
fn menu_system(
    meta: Root<GameMeta>,
    asset_server: Res<AssetServer>,
    ctx: Egui,
    mut sessions: ResMutInit<Sessions>,
    mut session_options: ResMutInit<SessionOptions>,
) {
    egui::CentralPanel::default().show(&ctx, |ui| {
        if ui.button("Start Game").clicked() {
            session_options.delete = true;

            let session = sessions.create("game");
            session.install_plugin(core::plugin);
            let map_meta = asset_server.get(meta.core.stable_maps[0]);
            session
                .world
                .insert_resource(LoadedMap(Arc::new(map_meta.clone())));
        }
    });
}
