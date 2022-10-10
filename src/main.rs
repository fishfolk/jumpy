#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    asset::AssetServerSettings, log::LogSettings, render::texture::ImageSettings,
    time::FixedTimestep,
};
use bevy_parallax::ParallaxResource;

mod animation;
mod assets;
mod camera;
mod config;
mod debug;
mod input;
mod lines;
mod loading;
mod localization;
mod map;
mod metadata;
mod name;
mod physics;
mod platform;
mod player;
mod prelude;
mod scripting;
mod ui;
mod utils;
mod workarounds;

use crate::{
    animation::AnimationPlugin, assets::AssetPlugin, camera::CameraPlugin, debug::DebugPlugin,
    input::InputPlugin, lines::LinesPlugin, loading::LoadingPlugin,
    localization::LocalizationPlugin, map::MapPlugin, metadata::GameMeta, name::NamePlugin,
    physics::PhysicsPlugin, platform::PlatformPlugin, player::PlayerPlugin, prelude::*,
    scripting::ScriptingPlugin, ui::UiPlugin, workarounds::WorkaroundsPlugin,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    LoadingPlatformStorage,
    LoadingGameData,
    MainMenu,
    InGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InGameState {
    Playing,
    Editing,
    Paused,
}

#[derive(StageLabel)]
pub enum FixedUpdateStage {
    First,
    PreUpdate,
    Update,
    PostUpdate,
    Last,
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
    app.add_loopless_state(GameState::LoadingPlatformStorage)
        .add_loopless_state(InGameState::Playing);

    // Add fixed update stages
    app.add_stage_after(
        CoreStage::First,
        FixedUpdateStage::First,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(1.0 / 60.0)),
    )
    .add_stage_after(
        CoreStage::PreUpdate,
        FixedUpdateStage::PreUpdate,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(1.0 / 60.0)),
    )
    .add_stage_after(
        CoreStage::Update,
        FixedUpdateStage::Update,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(1.0 / 60.0)),
    )
    .add_stage_after(
        CoreStage::PostUpdate,
        FixedUpdateStage::PostUpdate,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(1.0 / 60.0)),
    )
    .add_stage_after(
        CoreStage::Last,
        FixedUpdateStage::Last,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(1.0 / 60.0)),
    );

    // Install game plugins
    app.add_plugins(DefaultPlugins)
        .add_plugin(LinesPlugin)
        .add_plugin(PlatformPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(AssetPlugin)
        .add_plugin(LocalizationPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(NamePlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(UiPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(WorkaroundsPlugin)
        .add_plugin(DebugPlugin)
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
