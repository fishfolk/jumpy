//! Jumpy is a pixel-style, tactical 2D shooter with a fishy theme.
//!
//! This is the project's internal developer API documentation. API documentation is usually meant
//! for libraries with public APIs, but this is a game, so we use it to document the internal game
//! architecture for contributors.
//!
//! TODO: Write essentially an Architecture.md type of document here, and fill out the other game
//! module's documentation.

#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    asset::AssetServerSettings, log::LogSettings, render::texture::ImageSettings, text::TextPlugin,
};
use bevy_ggrs::{
    ggrs::{self},
    GGRSPlugin,
};
use bevy_parallax::ParallaxResource;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

pub mod animation;
pub mod assets;
pub mod audio;
pub mod camera;
pub mod config;
pub mod damage;
pub mod debug;
pub mod item;
pub mod lifetime;
pub mod lines;
pub mod loading;
pub mod localization;
pub mod map;
pub mod metadata;
pub mod name;
pub mod networking;
pub mod physics;
pub mod platform;
pub mod player;
pub mod prelude;
pub mod random;
pub mod run_criteria;
pub mod schedule;
// pub mod scripting;
pub mod session;
pub mod ui;
pub mod utils;
pub mod workarounds;

use crate::{
    animation::AnimationPlugin,
    assets::AssetPlugin,
    audio::AudioPlugin,
    camera::CameraPlugin,
    damage::DamagePlugin,
    debug::DebugPlugin,
    item::ItemPlugin,
    lifetime::LifetimePlugin,
    lines::LinesPlugin,
    loading::LoadingPlugin,
    localization::LocalizationPlugin,
    map::MapPlugin,
    metadata::{GameMeta, MetadataPlugin},
    name::NamePlugin,
    networking::NetworkingPlugin,
    physics::PhysicsPlugin,
    platform::PlatformPlugin,
    player::PlayerPlugin,
    prelude::*,
    random::RandomPlugin,
    session::SessionPlugin,
    ui::UiPlugin,
    utils::{is_in_game_run_criteria, UtilsPlugin},
    workarounds::WorkaroundsPlugin,
};

/// The game logic frames per second, aka. the fixed updates per second ( UPS/FPS ).
pub const FPS: usize = 45;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameState {
    LoadingPlatformStorage,
    LoadingGameData,
    MainMenu,
    InGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InGameState {
    Playing,
    Editing,
    Paused,
}

#[derive(StageLabel)]
pub enum RollbackStage {
    First,
    PreUpdate,
    PreUpdateInGame,
    Update,
    UpdateInGame,
    PostUpdate,
    Last,
}

#[derive(Debug)]
pub struct GgrsConfig;
impl ggrs::Config for GgrsConfig {
    type Input = player::input::DensePlayerControl;
    type State = u8;
    /// Addresses are the same as the player handle for our custom socket.
    type Address = usize;
}

pub fn main() {
    // Load engine config. This will parse CLI arguments or web query string so we want to do it
    // before we create the app to make sure everything is in order.
    let engine_config = &*config::ENGINE_CONFIG;

    let mut app = App::new();

    app.insert_resource(WindowDescriptor {
        title: "Fish Folk: Jumpy".to_string(),
        fit_canvas_to_parent: true,
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

    // Create the GGRS rollback schedule and plugin
    let mut rollback_schedule = Schedule::default();
    let rollback_plugin = GGRSPlugin::<GgrsConfig>::new();

    // Add fixed update stagesrefs/branchless/2fd80952e26d905aa258ebb7e6175a7cfc4cb76f
    rollback_schedule
        .add_stage(RollbackStage::First, SystemStage::parallel())
        .add_stage_after(
            RollbackStage::First,
            RollbackStage::PreUpdate,
            SystemStage::parallel(),
        )
        .add_stage_after(
            RollbackStage::PreUpdate,
            RollbackStage::Update,
            SystemStage::parallel(),
        )
        .add_stage_after(
            RollbackStage::PreUpdate,
            RollbackStage::PreUpdateInGame,
            SystemStage::parallel().with_run_criteria(is_in_game_run_criteria),
        )
        .add_stage_after(
            RollbackStage::Update,
            RollbackStage::PostUpdate,
            SystemStage::parallel(),
        )
        .add_stage_after(
            RollbackStage::Update,
            RollbackStage::UpdateInGame,
            SystemStage::parallel().with_run_criteria(is_in_game_run_criteria),
        )
        .add_stage_after(
            RollbackStage::PostUpdate,
            RollbackStage::Last,
            SystemStage::parallel(),
        );

    // Add the rollback schedule and plugin as resources, temporarily.
    // This allows plugins to modify them using `crate::schedule::RollbackScheduleAppExt`.
    app.insert_resource(rollback_schedule);
    app.insert_resource(rollback_plugin);

    // Install game plugins

    app.add_plugins_with(DefaultPlugins, |group| {
        // TODO: We should figure out how to not include these dependencies, so we can remove
        // this disable section.
        group
            .disable::<bevy::ui::UiPlugin>()
            .disable::<TextPlugin>()
    })
    .add_plugin(LinesPlugin)
    .add_plugin(UiPlugin);

    app.add_plugin(bevy_tweening::TweeningPlugin)
        .add_plugin(AudioPlugin)
        .add_plugin(UtilsPlugin)
        .add_plugin(MetadataPlugin)
        .add_plugin(PlatformPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(AssetPlugin)
        .add_plugin(LocalizationPlugin)
        .add_plugin(NamePlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(ItemPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(DamagePlugin)
        .add_plugin(LifetimePlugin)
        .add_plugin(WorkaroundsPlugin)
        .add_plugin(DebugPlugin)
        .add_plugin(RandomPlugin)
        // .add_plugin(ScriptingPlugin)
        .add_plugin(NetworkingPlugin)
        .add_plugin(SessionPlugin);

    // Pull the schedule back out of the world
    let rollback_schedule: Schedule = app.world.remove_resource().unwrap();
    let ggrs_plugin: GGRSPlugin<GgrsConfig> = app.world.remove_resource().unwrap();

    // Build the GGRS plugin
    ggrs_plugin
        .with_input_system(player::input::input_system)
        .with_update_frequency(crate::FPS)
        .with_rollback_schedule(rollback_schedule)
        .register_rollback_type::<Transform>()
        .register_rollback_type::<Handle<Image>>()
        .build(&mut app);

    debug!(?engine_config, "Starting game");

    // Get the game handle
    let asset_server = app.world.get_resource::<AssetServer>().unwrap();
    let game_asset = &engine_config.game_asset;
    let game_handle: Handle<GameMeta> = asset_server.load(game_asset);

    // Insert game handle resource
    app.world.insert_resource(game_handle);

    app.run()
}
