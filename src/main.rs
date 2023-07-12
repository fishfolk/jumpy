#![doc = include_str!("./README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/87333478?s=200&v=4")]
// This cfg_attr is needed because `rustdoc::all` includes lints not supported on stable
#![cfg_attr(doc, allow(unknown_lints))]
#![deny(rustdoc::all)]
#![allow(rustdoc::private_intra_doc_links)]
#![allow(clippy::type_complexity)]
#![allow(clippy::forget_non_drop)]
#![allow(clippy::too_many_arguments)]

/// Sets the global Rust allocator to MiMalloc instead of the system one.
///
/// Doesn't do anything on WASM builds. ( We may want an alternative allocator for WASM later. )
#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static GLOBAL_ALLOCATOR: mimalloc::MiMalloc = mimalloc::MiMalloc;

/// External crate documentation.
///
/// This module only exists during docs builds and serves to make it eaiser to link to relevant
/// documentation in external crates.
#[cfg(doc)]
pub mod external {
    #[doc(inline)]
    pub use bevy;
    #[doc(inline)]
    pub use bevy_egui::egui;
    #[doc(inline)]
    pub use bones_matchmaker_proto;
    #[doc(inline)]
    pub use ggrs;
}

// This will cause Bevy to be dynamically linked during development,
// which can greatly reduce re-compile times in some circumstances.
#[cfg(all(debug_assertions, not(target_arch = "wasm32")))]
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use bevy_dylib;

pub mod assets;
pub mod audio;
pub mod bevy_states;
pub mod config;
pub mod console;
pub mod debug;
pub mod input;
pub mod loading;
pub mod localization;
pub mod logs;
pub mod metadata;
pub mod platform;
pub mod profiling;
pub mod puffin_tracing;
pub mod session;
pub mod ui;
pub mod utils;

pub mod camera;
#[cfg(not(target_arch = "wasm32"))]
pub mod networking;
pub mod prelude;
use prelude::*;

/// The entrypoint function for the jumpy game.
///
/// This function:
///
/// - Parses engine config:
///     - On native: commandline arguments
///     - On web: query string parameters
/// - Initializes the Bevy [`App`]
/// - Initializes settings and resources
/// - Installs our Bevy plugins
///     - This includes 3rd party plugins, and our custom bevy plugins
///     - Nearly all of the game logic resides in these plugins
/// - Starts the game
pub fn main() {
    // Load engine config. This will parse CLI arguments or web query string so we want to do it
    // before we create the app to make sure everything is in order.
    let engine_config = Lazy::force(&config::ENGINE_CONFIG);

    let mut app = App::new();

    app
        // Initialize resources
        .insert_resource(ClearColor(Color::BLACK))
        // Set initial game state
        .add_state::<EngineState>()
        .add_state::<InGameState>()
        .add_state::<GameEditorState>()
        // Install plugins
        //
        // Log plugin is added first to ensure console_error_panic_hook is set early on,
        // otherwise in wasm an exception may occur without being logged to browser console.
        .add_plugin(JumpyLogPlugin {
            filter: engine_config.log_level.clone(),
            ..default()
        })
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Fish Folk: Jumpy".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(bevy::asset::AssetPlugin {
                    watch_for_changes: engine_config.hot_reload,
                    asset_folder: engine_config
                        .asset_dir
                        .clone()
                        .unwrap_or_else(|| "assets".into()),
                })
                // We are using JumpyLogPlugin, disabled to avoid conflicts in global logging/tracing.
                .disable::<bevy::log::LogPlugin>(),
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
        .add_plugin(JumpyDebugPlugin)
        .add_plugin(JumpyConsolePlugin);

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
