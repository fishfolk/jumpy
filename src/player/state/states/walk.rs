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
    items: Query<(Option<&Parent>, &KinematicBody), (With<Item>, Without<PlayerIdx>)>,
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
        'items: for (item_parent, ..) in &items {
            if item_parent.filter(|x| x.get() == player_ent).is_some() {
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
                    if let Ok((.., item_body)) = items.get(collider) {
                        if !item_body.is_deactivated {
                            commands
                                .add(PlayerSetInventoryCommand::new(player_ent, Some(collider)));
                            break 'colliders;
                        }
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
