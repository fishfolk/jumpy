use super::*;

pub const ID: Key = key!("core::midair");

pub fn player_state_transition(
    entities: Res<Entities>,
    player_inputs: Res<PlayerInputs>,
    player_indexes: Comp<PlayerIdx>,
    player_assets: BevyAssets<PlayerMeta>,
    mut player_states: CompMut<PlayerState>,
    bodies: Comp<KinematicBody>,
    mut audio_events: ResMut<AudioEvents>,
) {
    for (_ent, (player_idx, player_state, body)) in
        entities.iter_with((&player_indexes, &mut player_states, &bodies))
    {
        let meta_handle = player_inputs.players[player_idx.0]
            .selected_player
            .get_bevy_handle();
        let Some(meta) = player_assets.get(&meta_handle) else {
            continue;
        };
        if player_state.current != ID {
            continue;
        }

        if body.is_on_ground {
            // Play land sound
            audio_events.play(meta.sounds.land.clone(), meta.sounds.land_volume);
            // Switch to idle state
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

        if body.velocity.y > 0.0 {
            animation.current = key!("rise");
        } else {
            animation.current = key!("fall");
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

        // Limit fall speed if holding jump button
        if control.jump_pressed {
            body.velocity.y = body.velocity.y.max(-meta.stats.slow_fall_speed);
        }

        // Walk in movement direction
        body.velocity.x += meta.stats.accel_air_speed * control.move_direction.x;
        if control.move_direction.x.is_sign_positive() {
            body.velocity.x = body.velocity.x.min(meta.stats.air_speed);
        } else {
            body.velocity.x = body.velocity.x.max(-meta.stats.air_speed);
        }

        if control.move_direction.x == 0.0 {
            if body.velocity.x.is_sign_positive() {
                body.velocity.x = (body.velocity.x - meta.stats.slowdown).max(0.0);
            } else {
                body.velocity.x = (body.velocity.x + meta.stats.slowdown).min(0.0);
            }
        }

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
