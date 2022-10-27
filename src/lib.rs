#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

use std::time::Duration;

use bevy::{
    app::{RunMode, ScheduleRunnerPlugin, ScheduleRunnerSettings},
    asset::AssetServerSettings,
    log::{LogPlugin, LogSettings},
    pbr::PbrPlugin,
    render::{texture::ImageSettings, RenderPlugin},
    sprite::SpritePlugin,
    text::TextPlugin,
    time::FixedTimestep,
    window::WindowPlugin,
    winit::WinitPlugin,
};
use bevy_parallax::ParallaxResource;

pub mod animation;
pub mod assets;
pub mod camera;
pub mod config;
pub mod debug;
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
pub mod run_criteria;
pub mod scripting;
pub mod ui;
pub mod utils;
pub mod workarounds;

use crate::{
    animation::AnimationPlugin,
    assets::AssetPlugin,
    camera::CameraPlugin,
    debug::DebugPlugin,
    lines::LinesPlugin,
    loading::LoadingPlugin,
    localization::LocalizationPlugin,
    map::MapPlugin,
    metadata::{GameMeta, MetadataPlugin},
    name::NamePlugin,
    networking::{proto, server::NetServer, NetworkingPlugin},
    physics::PhysicsPlugin,
    platform::PlatformPlugin,
    player::PlayerPlugin,
    prelude::*,
    scripting::ScriptingPlugin,
    ui::UiPlugin,
    workarounds::WorkaroundsPlugin,
};

/// The timestep used for fixed update systems
pub const FIXED_TIMESTEP: f64 = 1.0 / 60.;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameState {
    LoadingPlatformStorage,
    LoadingGameData,
    MainMenu,
    InGame,
    ServerPlayerSelect,
    ServerInGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

pub fn build_app(net_server: Option<NetServer>) -> App {
    // Load engine config. This will parse CLI arguments or web query string so we want to do it
    // before we create the app to make sure everything is in order.
    let engine_config = &*config::ENGINE_CONFIG;

    let mut app = App::new();

    if !engine_config.server_mode {
        app.insert_resource(WindowDescriptor {
            title: "Fish Folk: Jumpy".to_string(),
            fit_canvas_to_parent: true,
            ..default()
        })
        .insert_resource(ImageSettings::default_nearest());
    }

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

    if !engine_config.server_mode {
        // Initialize resources
        app.insert_resource(ClearColor(Color::BLACK))
            .init_resource::<ParallaxResource>();
    }

    // Set initial game state
    app.add_loopless_state(GameState::LoadingPlatformStorage)
        .add_loopless_state(InGameState::Playing);

    // Add fixed update stages
    app.add_stage_after(
        CoreStage::First,
        FixedUpdateStage::First,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
    )
    .add_stage_after(
        CoreStage::PreUpdate,
        FixedUpdateStage::PreUpdate,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
    )
    .add_stage_after(
        CoreStage::Update,
        FixedUpdateStage::Update,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
    )
    .add_stage_after(
        CoreStage::PostUpdate,
        FixedUpdateStage::PostUpdate,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
    )
    .add_stage_after(
        CoreStage::Last,
        FixedUpdateStage::Last,
        SystemStage::parallel().with_run_criteria(FixedTimestep::step(crate::FIXED_TIMESTEP)),
    );

    // Install game plugins

    // Server mode requires special configuration to disable rendering, etc.
    if engine_config.server_mode {
        if let Some(net_server) = net_server {
            // Send each client their player index
            let player_count = net_server.client_count();
            for i in 0..player_count {
                info!("Sending net idx for player {i}");
                net_server.send_reliable(
                    &proto::ClientMatchInfo {
                        player_idx: i,
                        player_count,
                    },
                    i,
                );
            }

            app.insert_resource(net_server);
        } else {
            panic!("Net server required when in server mode");
        }

        app.add_plugins_with(DefaultPlugins, |group| {
            group
                .disable::<LogPlugin>()
                .disable::<RenderPlugin>()
                .disable::<WindowPlugin>()
                .disable::<WinitPlugin>()
                .disable::<SpritePlugin>()
                .disable::<bevy::ui::UiPlugin>()
                .disable::<TextPlugin>()
                .disable::<PbrPlugin>()
        })
        .init_resource::<Windows>()
        .add_asset::<TextureAtlas>()
        .insert_resource(ScheduleRunnerSettings {
            run_mode: RunMode::Loop {
                wait: Some(Duration::from_secs_f64(FIXED_TIMESTEP)),
            },
        })
        .add_plugin(ScheduleRunnerPlugin)
        .register_type::<bevy::render::view::ComputedVisibility>()
        .register_type::<bevy::render::view::Visibility>();

    // If we're not in server mode
    } else {
        app.add_plugins(DefaultPlugins)
            .add_plugin(LinesPlugin)
            .add_plugin(UiPlugin);
    }

    app.add_plugin(bevy_tweening::TweeningPlugin)
        .add_plugin(MetadataPlugin)
        .add_plugin(PlatformPlugin)
        .add_plugin(LoadingPlugin)
        .add_plugin(AssetPlugin)
        .add_plugin(LocalizationPlugin)
        .add_plugin(NamePlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(PhysicsPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(MapPlugin)
        .add_plugin(WorkaroundsPlugin)
        .add_plugin(DebugPlugin)
        .add_plugin(ScriptingPlugin)
        .add_plugin(NetworkingPlugin);

    debug!(?engine_config, "Starting game");

    // Get the game handle
    let asset_server = app.world.get_resource::<AssetServer>().unwrap();
    let game_asset = &engine_config.game_asset;
    let game_handle: Handle<GameMeta> = asset_server.load(game_asset);

    // Insert game handle resource
    app.world.insert_resource(game_handle);

    app
}
