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
    items: Query<(Option<&Parent>, &Rollback), With<Item>>,
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
        'items: for (item_parent, ..) in &items {
            if item_parent.filter(|x| x.get() == player_ent).is_some() {
                has_item = true;
                break 'items;
            }
        }

        // If we are grabbing
        if control.grab_just_pressed {
            if !has_item {
                let mut colliders = collision_world
                    // Get all things colliding with the player
                    .actor_collisions(player_ent)
                    .into_iter()
                    // Filter out anything not an item
                    .filter_map(|ent| items.get(ent).ok().map(|x| (ent, x)))
                    // Filter out any items that are being held by another player
                    .filter(|(_ent, (parent, _))| parent.is_none())
                    .collect::<Vec<_>>();

                // Sort the items to provide deterministic item selection if we hare touching multiples
                colliders.sort_by_key(|(_, (_, rollback))| rollback.id());

                // Grab the first item we are touching
                if let Some((item, _)) = colliders.get(0) {
                    commands.add(PlayerSetInventoryCommand::new(player_ent, Some(*item)));
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
