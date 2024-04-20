use super::*;

pub static ID: Lazy<Ustr> = Lazy::new(|| ustr("core::dead"));

pub fn install(session: &mut Session) {
    PlayerState::add_player_state_transition_system(session, player_state_transition);
    PlayerState::add_player_state_update_system(session, handle_player_state);
}

pub fn player_state_transition(
    entities: Res<Entities>,
    killed_players: Comp<PlayerKilled>,
    mut player_states: CompMut<PlayerState>,
) {
    for (_ent, (state, _killed)) in entities.iter_with((&mut player_states, &killed_players)) {
        state.current = *ID;
    }
}

pub fn handle_player_state(
    entities: Res<Entities>,
    mut commands: Commands,
    assets: Res<AssetServer>,
    player_indices: Comp<PlayerIdx>,
    player_inputs: Res<MatchInputs>,
    player_states: Comp<PlayerState>,
    killed_players: Comp<PlayerKilled>,
    _sprites: Comp<AtlasSprite>,
    _transform: Comp<Transform>,
    mut kinematic_bodies: CompMut<KinematicBody>,
    mut dynamic_bodies: CompMut<DynamicBody>,
    mut animations: CompMut<AnimationBankSprite>,
    game_meta: Root<GameMeta>,
    mut collision_world: CollisionWorld,
) {
    for (player_ent, (state, animation, _killed_player, player_idx)) in entities.iter_with((
        &player_states,
        &mut animations,
        &killed_players,
        &player_indices,
    )) {
        if state.current != *ID {
            continue;
        };

        if state.age == 0 {
            // let sprite = sprites.get(player_ent).unwrap();
            // let player_on_right = !sprite.flip_x;
            // let transform = transform.get(player_ent).unwrap();

            // Knock the player's hat off if they had one.
            commands.add(PlayerCommand::drop_hat(player_ent));

            if !dynamic_bodies.contains(player_ent) {
                let dynamic_body = DynamicBody::new(true);
                dynamic_bodies.insert(player_ent, dynamic_body);
            } else {
                dynamic_bodies.get_mut(player_ent).unwrap().is_dynamic = true;
            }

            let player_physics = &game_meta.core.physics.player;
            let dynamic = dynamic_bodies.get_mut(player_ent).unwrap();
            let pop_vel = Vec2::new(0.0, player_physics.ragdoll_initial_pop);
            let ang_vel = player_physics.ragdoll_initial_ang_vel;
            let current_vel = kinematic_bodies.get_mut(player_ent).unwrap().velocity;
            dynamic.push_simulation_command(Box::new(move |body: &mut rapier::RigidBody| {
                let mass = body.mass();
                body.set_linvel(current_vel.into(), true);

                // apply initial impulse
                body.apply_impulse((pop_vel * mass).into(), true);
                body.apply_torque_impulse(
                    ang_vel * body.mass_properties().effective_angular_inertia(),
                    true,
                )
            }));

            let player_meta_handle = player_inputs.players[player_idx.0 as usize].selected_player;
            let player_meta = &*assets.get(player_meta_handle);
            ragdoll::use_ragdoll_collider(player_ent, player_meta, &mut collision_world);

            // animation.current = match killed_player.hit_from {
            //     Some(hit_from)
            //         if {
            //             let is_hit_right = transform.translation.x < hit_from.x;
            //             (player_on_right && is_hit_right) || (!player_on_right && !is_hit_right)
            //         } =>
            //     {
            //         "death_spine".into()
            //     }
            //     _ => "death_belly".into(),
            // };

            animation.current = "death_ragdoll".into();
        }

        if state.age >= 80 {
            // If only one player in match, we wont' score / transition rounds, so respawn player.
            if player_indices.bitset().bit_count() == 1 {
                commands.add(PlayerCommand::despawn(player_ent));
            }
        }
    }
}
