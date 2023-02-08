use super::*;

pub const ID: Key = key!("core::walk");

pub fn player_state_transition(
    entities: Res<Entities>,
    player_inputs: Res<PlayerInputs>,
    player_indexes: Comp<PlayerIdx>,
    mut player_states: CompMut<PlayerState>,
    bodies: Comp<KinematicBody>,
) {
    for (_ent, (player_idx, player_state, body)) in
        entities.iter_with((&player_indexes, &mut player_states, &bodies))
    {
        if player_state.current != ID {
            continue;
        }

        let control = &player_inputs.players[player_idx.0].control;

        if !body.is_on_ground {
            player_state.current = midair::ID;
        } else if control.move_direction.y < -0.5 {
            player_state.current = crouch::ID;
        } else if control.move_direction.x == 0.0 {
            player_state.current = idle::ID;
        }
    }
}

pub fn handle_player_state(
    entities: Res<Entities>,
    player_inputs: Res<PlayerInputs>,
    player_indexes: Comp<PlayerIdx>,
    player_states: Comp<PlayerState>,
    player_assets: BevyAssets<PlayerMeta>,
    mut inventories: CompMut<Inventory>,
    mut sprites: CompMut<AtlasSprite>,
    mut animations: CompMut<AnimationBankSprite>,
    mut bodies: CompMut<KinematicBody>,
    items: Comp<Item>,
    mut player_events: ResMut<PlayerEvents>,
    mut audio_events: ResMut<AudioEvents>,
    collision_world: CollisionWorld,
) {
    // Collect a list of items that are being held by players
    let held_items = entities
        .iter_with(&inventories)
        .filter_map(|(_ent, inventory)| inventory.0)
        .collect::<Vec<_>>();

    let players = entities.iter_with((
        &player_states,
        &player_indexes,
        &mut animations,
        &mut sprites,
        &mut bodies,
        &mut inventories,
    ));
    for (player_ent, (player_state, player_idx, animation, sprite, body, inventory)) in players {
        if player_state.current != ID {
            continue;
        }
        let meta_handle = player_inputs.players[player_idx.0]
            .selected_player
            .get_bevy_handle();
        let Some(meta) = player_assets.get(&meta_handle) else {
            continue;
        };
        let control = &player_inputs.players[player_idx.0].control;

        // If this is the first frame of this state
        if player_state.age == 0 {
            // set our animation
            animation.current = key!("walk");
        }
        use_drop_or_grab_items(
            player_ent,
            meta,
            control,
            inventory,
            &collision_world,
            &items,
            &held_items,
            &mut player_events,
            &mut audio_events,
        );

        // If we are jumping
        if control.jump_just_pressed {
            audio_events.play(meta.sounds.jump.clone(), meta.sounds.jump_volume);

            // Move up
            body.velocity.y = meta.stats.jump_speed;
        }

        // Walk in movement direction
        body.velocity.x += meta.stats.accel_walk_speed * control.move_direction.x;
        if control.move_direction.x.is_sign_positive() {
            body.velocity.x = body.velocity.x.min(meta.stats.walk_speed);
        } else {
            body.velocity.x = body.velocity.x.max(-meta.stats.walk_speed);
        }

        // Point in movement direction
        if control.move_direction.x > 0.0 {
            sprite.flip_x = false;
        } else if control.move_direction.x < 0.0 {
            sprite.flip_x = true;
        }
    }
}
