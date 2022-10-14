use crate::{
    metadata::{GameMeta, Settings},
    platform::Storage,
    prelude::*,
};

use self::state::PlayerState;

pub mod input;
pub mod state;

/// The maximum number of players we may have in the game. This may change in the future.
pub const MAX_PLAYERS: usize = 4;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(input::PlayerInputPlugin)
            .add_plugin(state::PlayerStatePlugin)
            .register_type::<PlayerIdx>()
            .add_system_to_stage(
                FixedUpdateStage::PreUpdate,
                hydrate_players.run_if_resource_exists::<GameMeta>(),
            );
    }
}

/// The player index, for example Player 1, Player 2, and so on
#[derive(Component, Deref, DerefMut, Reflect, Default)]
#[reflect(Default, Component)]
pub struct PlayerIdx(pub usize);

fn hydrate_players(
    mut commands: Commands,
    players: Query<(Entity, &PlayerIdx), Without<PlayerState>>,
    mut storage: ResMut<Storage>,
    game: Res<GameMeta>,
) {
    let settings = storage.get(Settings::STORAGE_KEY);
    let settings = settings.as_ref().unwrap_or(&game.default_settings);

    for (entity, player_idx) in &players {
        commands
            .entity(entity)
            .insert(Name::new(format!("Player {}", player_idx.0)))
            .insert(PlayerState::default())
            .insert_bundle(InputManagerBundle {
                input_map: settings.player_controls.get_input_map(player_idx.0),
                ..default()
            });
    }
}
