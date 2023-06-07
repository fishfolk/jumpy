#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/87333478?s=200&v=4")]
// This cfg_attr is needed because `rustdoc::all` includes lints not supported on stable
#![cfg_attr(doc, allow(unknown_lints))]
#![deny(rustdoc::all)]
#![allow(clippy::too_many_arguments)]

/// Prelude for inside the crate
mod prelude;

/// Prelude for use outside the crate
#[doc(hidden)]
pub mod bevy_prelude {
    pub use {
        crate::{
            input::EditorInput,
            metadata::*,
            session::{CoreSession, CoreSessionInfo, GameSessionPlayerInfo},
            MAX_PLAYERS,
        },
        bones_lib::prelude as bones,
    };
}

/// External crate documentation.
///
/// This module only exists during docs builds and serves to make it eaiser to link to relevant
/// documentation in external crates.
#[cfg(doc)]
pub mod external {
    #[doc(inline)]
    pub use bones_bevy_asset;
    #[doc(inline)]
    pub use bones_lib;
}

pub mod attachment;
pub mod bullet;
pub mod camera;
pub mod damage;
pub mod debug;
pub mod editor;
pub mod elements;
pub mod globals;
pub mod input;
pub mod item;
pub mod lifetime;
pub mod map;
pub mod map_constructor;
pub mod metadata;
pub mod physics;
pub mod player;
pub mod random;
pub mod session;
pub mod utils;

/// The target fixed frames-per-second that the game sumulation runs at.
pub const FPS: f32 = 60.0;
/// The maximum number of players per match.
pub const MAX_PLAYERS: usize = 4;

/// Install game modules into the given [`CoreSession`][session::CoreSession].
///
/// Each game module should have a public install function that adds the systems needed by that
/// module:
///
/// ```ignore
/// pub fn install(session: &mut crate::session::CoreSession) {
///      session
///         .stages
///         .add_system_to_stage(CoreStage::Last, camera_controller);
/// }
/// ```
///
/// The module will usually contain the Rust structs for any components related to the module as
/// well.
///
/// To include the module in the game, add add a line to the body of this [`install_modules()`]
/// function that calls your module's `install()` function:
///
/// ```ignore
/// pub fn install_modules(session: &mut session::CoreSession) {
///     // other modules...
///     camera::install(session);
/// }
/// ```
///
/// Note that in some edge cases the order that the modules are installed can make a difference
/// because it will change the order that the module systems are run, if two modules add systems to
/// the same system stage.
pub fn install_modules(session: &mut session::CoreSession) {
    bones_lib::install(&mut session.stages);
    physics::install(session);
    input::install(session);
    map::install(session);
    player::install(session);
    elements::install(session);
    damage::install(session);
    camera::install(session);
    lifetime::install(session);
    random::install(session);
    debug::install(session);
    item::install(session);
    attachment::install(session);
    bullet::install(session);
    editor::install(session);
}
