use super::*;

pub const ID: Key = key!("core::crouch");

pub fn player_state_transition(
    entities: Res<Entities>,
    player_inputs: Res<PlayerInputs>,
    player_indexes: Comp<PlayerIdx>,
    mut player_states: CompMut<PlayerState>,
    player_assets: BevyAssets<PlayerMeta>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
) {
    for (_ent, (state, player_idx, body, transform)) in entities.iter_with((
        &mut player_states,
        &player_indexes,
        &mut bodies,
        &mut transforms,
    )) {
        let meta_handle = player_inputs.players[player_idx.0]
            .selected_player
            .get_bevy_handle();
        let Some(meta) = player_assets.get(&meta_handle) else {
            continue;
        };

        // Reset the body size and position if we stop sliding
        if state.last == ID && state.current != ID {
            if let ColliderShape::Rectangle { size } = &body.shape {
                if *size != meta.body_size {
                    body.shape = ColliderShape::Rectangle {
                        size: meta.body_size,
                    };
                    transform.translation += (meta.body_size.y - meta.slide_body_size.y) / 2.0;
                }
            }
        }

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
    mut transforms: CompMut<Transform>,
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

    for (player_ent, (state, player_idx, animation, body, inventory, transform)) in entities
        .iter_with((
            &player_states,
            &player_indexes,
            &mut animations,
            &mut bodies,
            &mut inventories,
            &mut transforms,
        ))
    {
        if state.current != ID {
            continue;
        }
        let meta_handle = player_inputs.players[player_idx.0]
            .selected_player
            .get_bevy_handle();
        let Some(meta) = player_assets.get(&meta_handle) else {
            continue;
        };

        if body.velocity.x == 0.0 {
            animation.current = key!("crouch");
            if let ColliderShape::Rectangle { size } = &body.shape {
                if *size != meta.body_size {
                    body.shape = ColliderShape::Rectangle {
                        size: meta.body_size,
                    };
                    transform.translation += (meta.body_size.y - meta.slide_body_size.y) / 2.0;
                }
            }
        } else if let ColliderShape::Rectangle { size } = &body.shape {
            animation.current = key!("slide");

            if *size != meta.slide_body_size {
                body.shape = ColliderShape::Rectangle {
                    size: meta.slide_body_size,
                };
                transform.translation -= (meta.body_size.y - meta.slide_body_size.y) / 2.0;
            }
        }

        let control = &player_inputs.players[player_idx.0].control;

        if control.jump_just_pressed {
            body.fall_through = true;
        }

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
