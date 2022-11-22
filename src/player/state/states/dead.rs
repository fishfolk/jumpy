use crate::player::{PlayerDespawnCommand, PlayerKilled};

use super::*;

pub const ID: &str = "core:dead";

pub fn player_state_transition(mut players: Query<&mut PlayerState, With<PlayerKilled>>) {
    // Transition all killed players to this state
    for mut player_state in &mut players {
        if player_state.id != ID {
            player_state.id = ID.into();
        }
    }
}

pub fn handle_player_state(
    mut commands: Commands,
    mut players: Query<(Entity, &PlayerState, &mut AnimationBankSprite), With<PlayerKilled>>,
) {
    for (entity, state, mut sprite) in &mut players {
        if state.age == 0 {
            sprite.current_animation = "death_1".into();
        }

        if state.age >= 80 {
            commands.add(PlayerDespawnCommand::new(entity));
        }
    }
}
