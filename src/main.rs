#![doc = include_str!("./README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/87333478?s=200&v=4")]
// This cfg_attr is needed because `rustdoc::all` includes lints not supported on stable
#![cfg_attr(doc, allow(unknown_lints))]
#![deny(rustdoc::all)]
#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// This will cause Bevy to be dynamically linked during development,
// which can greatly reduce re-compile times in some circumstances.
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use bevy_dylib;

pub mod assets;
pub mod audio;
pub mod config;
pub mod debug;
pub mod input;
pub mod loading;
pub mod localization;
pub mod metadata;
pub mod platform;
pub mod session;
pub mod ui;

pub mod camera;
pub mod prelude;
pub use prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EngineState {
    LoadingPlatformStorage,
    LoadingGameData,
    MainMenu,
    InGame,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InGameState {
    Playing,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GameEditorState {
    Hidden,
    Visible,
}

#[derive(StageLabel, Eq, PartialEq, Hash)]
pub enum RollbackStage {
    Input,
    First,
    PreUpdate,
    Update,
    PostUpdate,
    Last,
}

pub fn main() {
    // Load engine config. This will parse CLI arguments or web query string so we want to do it
    // before we create the app to make sure everything is in order.
    let engine_config = Lazy::force(&config::ENGINE_CONFIG);

    let mut app = App::new();

    app
        // Initialize resources
        .insert_resource(ClearColor(Color::BLACK))
        // Set initial game state
        .add_loopless_state(EngineState::LoadingPlatformStorage)
        .add_loopless_state(InGameState::Playing)
        .add_loopless_state(GameEditorState::Hidden)
        // Install plugins
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    window: WindowDescriptor {
                        title: "Fish Folk: Jumpy".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    },
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(bevy::log::LogPlugin {
                    filter: engine_config.log_level.clone(),
                    ..default()
                })
                .set(bevy::asset::AssetPlugin {
                    watch_for_changes: engine_config.hot_reload,
                    asset_folder: engine_config
                        .asset_dir
                        .clone()
                        .unwrap_or_else(|| "assets".into()),
                }),
        )
        .add_plugin(bevy_tweening::TweeningPlugin)
        .add_plugin(bevy_framepace::FramepacePlugin)
        .add_plugin(JumpyPlayerInputPlugin)
        .add_plugin(JumpySessionPlugin)
        .add_plugin(JumpyUiPlugin)
        .add_plugin(JumpyAudioPlugin)
        .add_plugin(JumpyPlatformPlugin)
        .add_plugin(JumpyLoadingPlugin)
        .add_plugin(JumpyAssetPlugin)
        .add_plugin(JumpyLocalizationPlugin)
        .add_plugin(JumpyDebugPlugin);

    debug!(?engine_config, "Starting game");

    // Get the game handle
    let asset_server = app.world.get_resource::<AssetServer>().unwrap();
    let game_asset = &engine_config.game_asset;
    let game_handle = GameMetaHandle(asset_server.load(game_asset));

    // Insert game handle resource
    app.world.insert_resource(game_handle);

    // Start the game
    app.run()
}
