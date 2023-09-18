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
pub mod utils;

/// The target fixed frames-per-second that the game sumulation runs at.
pub const FPS: f32 = 60.0;

/// The maximum number of players per match.
pub const MAX_PLAYERS: usize = 4;

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        attachment::*, bullet::*, camera::*, damage::*, debug::*, editor::*, editor::*,
        elements::prelude::*, elements::prelude::*, globals::*, input::*, item::*, lifetime::*, map::*,
        map_constructor::*, metadata::*, physics::*, player::*, random::*, utils::*, FPS,
        MAX_PLAYERS,
    };
}

pub fn plugin(session: &mut Session) {
    physics::install(session);
    input::install(session);
    map::install(session);
    player::plugin(session);
    elements::install(session);
    damage::install(session);
    camera::install(session);
    lifetime::install(session);
    random::plugin(session);
    debug::plugin(session);
    item::install(session);
    attachment::install(session);
    bullet::install(session);
    editor::install(session);
}
