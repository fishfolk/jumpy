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

/// Implements built-in player states
pub struct StatesPlugin;

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

pub const JUMP_SPEED: f32 = 17.0;

/// The idling state, when the player is standing still
pub mod idle {
    use super::*;

    pub const ID: &str = "core:idle";

    pub fn player_state_transition(
        player_inputs: Res<PlayerInputs>,
        mut players: Query<(&mut PlayerState, &PlayerIdx, &KinematicBody)>,
    ) {
        for (mut player_state, player_idx, body) in &mut players {
            if player_state.id != ID {
                continue;
            }

            let control = &player_inputs.players[player_idx.0].control;

            if !body.is_on_ground {
                player_state.id = midair::ID.into();
            } else if control.move_direction.y < -0.5 {
                player_state.id = crouch::ID.into();
            } else if control.move_direction.x != 0.0 {
                player_state.id = walk::ID.into();
            }
        }
    }

    pub fn handle_player_state(
        mut commands: Commands,
        player_inputs: Res<PlayerInputs>,
        items: Query<&Parent, With<Item>>,
        mut players: Query<(
            Entity,
            &PlayerState,
            &PlayerIdx,
            &mut AnimationBankSprite,
            &mut KinematicBody,
        )>,
        collision_world: CollisionWorld,
    ) {
        for (player_ent, player_state, player_idx, mut sprite, mut body) in &mut players {
            if player_state.id != ID {
                continue;
            }

            // If this is the first frame of this state
            if player_state.age == 0 {
                // set our animation to idle
                sprite.current_animation = "idle".into();
            }

            let control = &player_inputs.players[player_idx.0].control;

            // Check for item in player inventory
            let mut has_item = false;
            'items: for item_parent in &items {
                if item_parent.get() == player_ent {
                    has_item = true;
                    break 'items;
                }
            }

            // If we are grabbing
            if control.grab_just_pressed {
                // If we don't have an item
                if !has_item {
                    // For each actor colliding with the player
                    'colliders: for collider in collision_world.actor_collisions(player_ent) {
                        // If this is an item
                        if items.contains(collider) {
                            commands
                                .add(PlayerSetInventoryCommand::new(player_ent, Some(collider)));
                            break 'colliders;
                        }
                    }

                // If we are already carrying an item
                } else {
                    // Drop it
                    commands.add(PlayerSetInventoryCommand::new(player_ent, None));
                }
            }

            // If we are using an item
            if control.shoot_just_pressed && has_item {
                commands.add(PlayerUseItemCommand::new(player_ent));
            }

            // If we are jumping
            if control.jump_just_pressed {
                // Move up
                body.velocity.y = JUMP_SPEED;
            }

            // Since we are idling, don't move
            body.velocity.x = 0.0;
        }
    }
}

pub mod midair {
    use super::*;

    pub const ID: &str = "core:midair";

    pub const AIR_MOVE_SPEED: f32 = 7.0;

    pub fn player_state_transition(mut players: Query<(&mut PlayerState, &KinematicBody)>) {
        for (mut player_state, body) in &mut players {
            if player_state.id != ID {
                continue;
            }

            if body.is_on_ground {
                player_state.id = idle::ID.into();
            }
        }
    }

    pub fn handle_player_state(
        mut commands: Commands,
        player_inputs: Res<PlayerInputs>,
        items: Query<&Parent, With<Item>>,
        mut players: Query<(
            Entity,
            &PlayerState,
            &PlayerIdx,
            &mut AnimationBankSprite,
            &mut KinematicBody,
        )>,
        collision_world: CollisionWorld,
    ) {
        for (player_ent, player_state, player_idx, mut sprite, mut body) in &mut players {
            if player_state.id != ID {
                continue;
            }

            if body.velocity.y > 0.0 {
                sprite.current_animation = "rise".into();
            } else {
                sprite.current_animation = "fall".into();
            }

            let control = &player_inputs.players[player_idx.0].control;

            // Check for item in player inventory
            let mut has_item = false;
            'items: for item_parent in &items {
                if item_parent.get() == player_ent {
                    has_item = true;
                    break 'items;
                }
            }

            // If we are grabbing
            if control.grab_just_pressed {
                // If we don't have an item
                if !has_item {
                    // For each actor colliding with the player
                    'colliders: for collider in collision_world.actor_collisions(player_ent) {
                        // If this is an item
                        if items.contains(collider) {
                            commands
                                .add(PlayerSetInventoryCommand::new(player_ent, Some(collider)));
                            break 'colliders;
                        }
                    }

                // If we are already carrying an item
                } else {
                    // Drop it
                    commands.add(PlayerSetInventoryCommand::new(player_ent, None));
                }
            }

            // If we are using an item
            if control.shoot_just_pressed && has_item {
                commands.add(PlayerUseItemCommand::new(player_ent));
            }

            // Add controls
            body.velocity.x = control.move_direction.x * AIR_MOVE_SPEED;

            // Fall through platforms
            body.fall_through = control.move_direction.y < -0.5 && control.jump_pressed;

            // Point in movement direction
            if control.move_direction.x > 0.0 {
                sprite.flip_x = false;
            } else if control.move_direction.x < 0.0 {
                sprite.flip_x = true;
            }
        }
    }
}

pub mod walk {
    use super::*;

    pub const ID: &str = "core:walk";
    pub const WALK_SPEED: f32 = 7.0;

    pub fn player_state_transition(
        player_inputs: Res<PlayerInputs>,
        mut players: Query<(&mut PlayerState, &PlayerIdx, &KinematicBody)>,
    ) {
        for (mut player_state, player_idx, body) in &mut players {
            if player_state.id != ID {
                continue;
            }

            let control = &player_inputs.players[player_idx.0].control;

            if !body.is_on_ground {
                player_state.id = midair::ID.into();
            } else if control.move_direction.y < -0.5 {
                player_state.id = crouch::ID.into();
            } else if control.move_direction.x == 0.0 {
                player_state.id = idle::ID.into();
            }
        }
    }

    pub fn handle_player_state(
        mut commands: Commands,
        player_inputs: Res<PlayerInputs>,
        items: Query<&Parent, With<Item>>,
        mut players: Query<(
            Entity,
            &PlayerState,
            &PlayerIdx,
            &mut AnimationBankSprite,
            &mut KinematicBody,
        )>,
        collision_world: CollisionWorld,
    ) {
        for (player_ent, player_state, player_idx, mut sprite, mut body) in &mut players {
            if player_state.id != ID {
                continue;
            }

            // If this is the first frame of this state
            if player_state.age == 0 {
                // set our animation
                sprite.current_animation = "walk".into();
            }

            let control = &player_inputs.players[player_idx.0].control;

            // Check for item in player inventory
            let mut has_item = false;
            'items: for item_parent in &items {
                if item_parent.get() == player_ent {
                    has_item = true;
                    break 'items;
                }
            }

            // If we are grabbing
            if control.grab_just_pressed {
                // If we don't have an item
                if !has_item {
                    // For each actor colliding with the player
                    'colliders: for collider in collision_world.actor_collisions(player_ent) {
                        // If this is an item
                        if items.contains(collider) {
                            commands
                                .add(PlayerSetInventoryCommand::new(player_ent, Some(collider)));
                            break 'colliders;
                        }
                    }

                // If we are already carrying an item
                } else {
                    // Drop it
                    commands.add(PlayerSetInventoryCommand::new(player_ent, None));
                }
            }

            // If we are using an item
            if control.shoot_just_pressed && has_item {
                commands.add(PlayerUseItemCommand::new(player_ent));
            }

            // If we are jumping
            if control.jump_just_pressed {
                // Move up
                body.velocity.y = JUMP_SPEED;
            }

            // Walk in movement direction
            body.velocity.x = control.move_direction.x * WALK_SPEED;

            // Point in movement direction
            if control.move_direction.x > 0.0 {
                sprite.flip_x = false;
            } else if control.move_direction.x < 0.0 {
                sprite.flip_x = true;
            }
        }
    }
}

pub mod crouch {
    use super::*;

    pub const ID: &str = "core:crouch";

    pub fn player_state_transition(
        player_inputs: Res<PlayerInputs>,
        mut players: Query<(&mut PlayerState, &PlayerIdx, &KinematicBody)>,
    ) {
        for (mut player_state, player_idx, body) in &mut players {
            if player_state.id != ID {
                continue;
            }

            let control = &player_inputs.players[player_idx.0].control;

            if !body.is_on_ground || control.move_direction.y > -0.5 {
                player_state.id = idle::ID.into();
            }
        }
    }

    pub fn handle_player_state(
        player_inputs: Res<PlayerInputs>,
        mut players: Query<(
            &PlayerState,
            &PlayerIdx,
            &mut AnimationBankSprite,
            &mut KinematicBody,
        )>,
    ) {
        for (player_state, player_idx, mut sprite, mut body) in &mut players {
            if player_state.id != ID {
                continue;
            }

            // Set animation
            if player_state.age == 0 {
                sprite.current_animation = "crouch".into();
            }

            let control = &player_inputs.players[player_idx.0].control;

            if control.jump_just_pressed {
                body.fall_through = true;
            }
        }
    }
}
