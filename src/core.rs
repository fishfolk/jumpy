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

use std::array;

use crate::prelude::*;

pub mod prelude {
    pub use super::{
        attachment::*, bullet::*, camera::*, damage::*, debug::*, editor::*, editor::*,
        elements::prelude::*, elements::prelude::*, globals::*, input::*, item::*, lifetime::*,
        map::*, map_constructor::*, metadata::*, physics::*, player::*, random::*, utils::*, FPS,
        MAX_PLAYERS,
    };
}

pub struct MatchPlugin {
    pub map: MapMeta,
    pub selected_players: [Option<Handle<PlayerMeta>>; MAX_PLAYERS],
}

impl Plugin for MatchPlugin {
    fn install(self, session: &mut Session) {
        session.install_plugin(DefaultPlugin);

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

        session.world.insert_resource(LoadedMap(Arc::new(self.map)));
        session.world.insert_resource(PlayerInputs {
            players: array::from_fn(|i| {
                self.selected_players[i]
                    .map(|selected_player| PlayerInput {
                        active: true,
                        selected_player,
                        ..default()
                    })
                    .unwrap_or_default()
            }),
        });
    }
}
