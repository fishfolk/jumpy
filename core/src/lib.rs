#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/87333478?s=200&v=4")]
// This cfg_attr is needed because `rustdoc::all` includes lints not supported on stable
#![cfg_attr(doc, allow(unknown_lints))]
#![deny(rustdoc::all)]
#![allow(clippy::too_many_arguments)]

/// Prelude for inside the crate
mod prelude;

/// Prelude for use outside the crate
pub mod bevy_prelude {
    pub use {
        crate::{
            metadata::*,
            session::{GameSession, GameSessionInfo},
            MAX_PLAYERS,
        },
        bones_lib::prelude as bones,
    };
}

pub mod attachment;
pub mod bullet;
pub mod camera;
pub mod damage;
pub mod debug;
pub mod elements;
pub mod input;
pub mod item;
pub mod lifetime;
pub mod map;
pub mod math;
pub mod metadata;
pub mod physics;
pub mod player;
pub mod random;
pub mod session;
pub mod testing;

/// The target fixed frames-per-second that the game sumulation runs at.
pub const FPS: f32 = 60.0;
pub const MAX_PLAYERS: usize = 4;

/// Install game modules into the session.
pub fn install_modules(session: &mut session::GameSession) {
    testing::install(session);
    physics::install(session);
    input::install(session);
    map::install(session);
    elements::install(session);
    player::install(session);
    camera::install(session);
    damage::install(session);
    lifetime::install(session);
    random::install(session);
    debug::install(session);
    attachment::install(session);
    bullet::install(session);
}
