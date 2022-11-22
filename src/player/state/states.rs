use crate::prelude::*;

use crate::{
    animation::AnimationBankSprite,
    item::Item,
    physics::collisions::CollisionWorld,
    physics::KinematicBody,
    player::{
        input::PlayerInputs, state::PlayerState, PlayerIdx, PlayerSetInventoryCommand,
        PlayerUseItemCommand,
    },
};

use super::PlayerStateStage;

pub const JUMP_SPEED: f32 = 17.0;

pub mod crouch;
pub mod dead;
pub mod idle;
pub mod midair;
pub mod walk;

/// Helper macro that adds the `player_state_transition` and `handle_player_state` systems from
/// `module` to the appropriate stages in `app`.
macro_rules! add_state_module {
    ($app:ident, $module:ident) => {
        $app.extend_rollback_schedule(|schedule| {
            schedule.add_system_to_stage(
                PlayerStateStage::PerformTransitions,
                $module::player_state_transition,
            );
            schedule
                .add_system_to_stage(PlayerStateStage::HandleState, $module::handle_player_state);
        });
    };
}

/// Implements built-in player states
pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        // Add default state
        app.extend_rollback_schedule(|schedule| {
            schedule.add_system_to_stage(
                PlayerStateStage::PerformTransitions,
                default::player_state_transition,
            );
        });

        // Add other states
        add_state_module!(app, idle);
        add_state_module!(app, midair);
        add_state_module!(app, walk);
        add_state_module!(app, crouch);
        add_state_module!(app, dead);
    }
}

/// The meaningless default state that players start at when spawned.
pub mod default {
    use super::*;

    pub fn player_state_transition(mut states: Query<&mut PlayerState>) {
        for mut state in &mut states {
            // If the current state is the default, meaningless state
            if state.id.is_empty() {
                // Transition to idle
                state.id = idle::ID.into();
            }
        }
    }
}
