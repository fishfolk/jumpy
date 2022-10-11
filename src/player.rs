use crate::prelude::*;

pub mod input;

/// The maximum number of players we may have in the game. This may change in the future.
pub const MAX_PLAYERS: usize = 4;

pub struct PlayerPlugin;

#[derive(StageLabel)]
pub struct PlayerInputFixedUpdate;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(input::PlayerInputPlugin)
            .register_type::<PlayerIdx>();
    }
}

/// The player index, for example Player 1, Player 2, and so on
#[derive(Component, Deref, DerefMut, Reflect, Default)]
#[reflect(Default)]
pub struct PlayerIdx(pub usize);
