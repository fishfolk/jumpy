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
    items: Query<(Option<&Parent>, &Rollback), With<Item>>,
    mut players: Query<(
        Entity,
        &PlayerState,
        &PlayerIdx,
        &Handle<PlayerMeta>,
        &mut AnimationBankSprite,
        &mut KinematicBody,
    )>,
    collision_world: CollisionWorld,
    player_assets: Res<Assets<PlayerMeta>>,
    effects: Res<AudioChannel<EffectsChannel>>,
) {
    for (player_ent, player_state, player_idx, meta_handle, mut sprite, mut body) in &mut players {
        let meta = player_assets.get(meta_handle).unwrap();

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

                    // Play grab sound
                    if player_inputs.is_confirmed {
                        effects
                            .play(meta.sounds.grab_handle.clone_weak())
                            .with_volume(meta.sounds.grab_volume as _);
                    }
                }

            // If we are already carrying an item
            } else {
                // Drop it
                commands.add(PlayerSetInventoryCommand::new(player_ent, None));

                // Play drop sound
                if player_inputs.is_confirmed {
                    effects
                        .play(meta.sounds.drop_handle.clone_weak())
                        .with_volume(meta.sounds.drop_volume as _);
                }
            }
        }

        // If we are using an item
        if control.shoot_just_pressed && has_item {
            commands.add(PlayerUseItemCommand::new(player_ent));
        }

        // If we are jumping
        if control.jump_just_pressed {
            // Play jump sound
            if player_inputs.is_confirmed {
                effects
                    .play(meta.sounds.jump_handle.clone_weak())
                    // TODO: This volume should be relative to the current channel volume, not
                    // hard-coded, so that when the user changes the sound effect volume it's relative.
                    .with_volume(meta.sounds.jump_volume as _);
            }

            // Move up
            body.velocity.y = JUMP_SPEED;
        }

        // Since we are idling, don't move
        body.velocity.x = 0.0;
    }
}
