#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

use bevy::{asset::AssetServerSettings, log::LogSettings, render::texture::ImageSettings};
use bevy_parallax::ParallaxResource;

mod assets;
mod config;
mod input;
mod loading;
mod localization;
mod metadata;
mod platform;
mod prelude;
mod scripting;
mod ui;
mod utils;

use crate::{
    assets::AssetPlugin, input::InputPlugin, loading::LoadingPlugin,
    localization::LocalizationPlugin, metadata::GameMeta, platform::PlatformPlugin, prelude::*,
    scripting::ScriptingPlugin, ui::UiPlugin,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GameState {
    LoadingPlatformStorage,
    LoadingGame,
    MainMenu,
    InGame,
    Paused,
}

pub fn main() {
    // Load engine config. This will parse CLI arguments or web query string so we want to do it
    // before we create the app to make sure everything is in order.
    let engine_config = &*config::ENGINE_CONFIG;

    let mut app = App::new();
    app.insert_resource(WindowDescriptor {
        title: "Fish Folk: Jumpy".to_string(),
        scale_factor_override: Some(1.0),
        ..default()
    })
    .insert_resource(ImageSettings::default_nearest());

    // Configure log level
    app.insert_resource(LogSettings {
        filter: engine_config.log_level.clone(),
        ..default()
    });

    // Configure asset server
    let mut asset_server_settings = AssetServerSettings {
        watch_for_changes: engine_config.hot_reload,
        ..default()
    };
    if let Some(asset_dir) = &engine_config.asset_dir {
        asset_server_settings.asset_folder = asset_dir.clone();
    }
    app.insert_resource(asset_server_settings);

    // Initialize resources
    app.insert_resource(ClearColor(Color::BLACK))
        .init_resource::<ParallaxResource>();

    // Set initial game state
    app.add_loopless_state(GameState::LoadingPlatformStorage);

    // Install game plugins
    app.add_plugins(DefaultPlugins)
        .add_plugin(PlatformPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(AssetPlugin)
        .add_plugin(LocalizationPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(ScriptingPlugin);

    debug!(?engine_config, "Starting game");

    // Get the game handle
    let asset_server = app.world.get_resource::<AssetServer>().unwrap();
    let game_asset = &engine_config.game_asset;
    let game_handle: Handle<GameMeta> = asset_server.load(game_asset);

    // Insert game handle resource
    app.world.insert_resource(game_handle);

    // Start the game!
    app.run();
}
