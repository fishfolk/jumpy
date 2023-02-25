use super::*;

pub const ID: Key = key!("core::crouch");

pub fn player_state_transition(
    entities: Res<Entities>,
    player_inputs: Res<PlayerInputs>,
    player_indexes: Comp<PlayerIdx>,
    mut player_states: CompMut<PlayerState>,
    bodies: Comp<KinematicBody>,
) {
    for (_ent, (state, player_idx, body)) in
        entities.iter_with((&mut player_states, &player_indexes, &bodies))
    {
        if state.current != ID {
            continue;
        }
        let control = &player_inputs.players[player_idx.0].control;

        if !body.is_on_ground || control.move_direction.y > -0.5 {
            state.current = idle::ID;
        }
    }
}

pub fn handle_player_state(
    entities: Res<Entities>,
    player_inputs: Res<PlayerInputs>,
    player_states: Comp<PlayerState>,
    player_indexes: Comp<PlayerIdx>,
    mut animations: CompMut<AnimationBankSprite>,
    mut bodies: CompMut<KinematicBody>,
    player_assets: BevyAssets<PlayerMeta>,
    mut inventories: CompMut<Inventory>,
    items: Comp<Item>,
    mut audio_events: ResMut<AudioEvents>,
    collision_world: CollisionWorld,
    mut commands: Commands,
) {
    // Collect a list of items that are being held by players
    let held_items = entities
        .iter_with(&inventories)
        .filter_map(|(_ent, inventory)| inventory.0)
        .collect::<Vec<_>>();

    for (player_ent, (state, player_idx, animation, body, inventory)) in entities.iter_with((
        &player_states,
        &player_indexes,
        &mut animations,
        &mut bodies,
        &mut inventories,
    )) {
        if state.current != ID {
            continue;
        }

        if state.age == 0 {
            animation.current = key!("crouch");
        }

        let control = &player_inputs.players[player_idx.0].control;

        if control.jump_just_pressed {
            body.fall_through = true;
        }

        let meta_handle = player_inputs.players[player_idx.0]
            .selected_player
            .get_bevy_handle();
        let Some(meta) = player_assets.get(&meta_handle) else {
            continue;
        };

        use_drop_or_grab_items(
            player_ent,
            meta,
            control,
            inventory,
            &collision_world,
            &items,
            &held_items,
            &mut audio_events,
            &mut commands,
        );
    }
}
