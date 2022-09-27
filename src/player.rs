use crate::prelude::*;

/// The maximum number of players we may have in the game. This may change in the future.
pub const MAX_PLAYERS: usize = 4;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerIdx>()
            .register_type::<PlayerActive>();
    }
}

/// The player index, for example Player 1, Player 2, and so on
#[derive(Component, Deref, DerefMut, Reflect)]
pub struct PlayerIdx(pub usize);

#[derive(Component, Reflect)]
#[component(storage = "SparseSet")]
pub struct PlayerActive;
