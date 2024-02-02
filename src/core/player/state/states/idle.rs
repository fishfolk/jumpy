use super::*;

pub static ID: Lazy<Ustr> = Lazy::new(|| ustr("core::idle"));

pub fn install(session: &mut Session) {
    PlayerState::add_player_state_transition_system(session, player_state_transition);
    PlayerState::add_player_state_update_system(session, handle_player_state);
    PlayerState::add_player_state_update_system(session, use_drop_or_grab_items_system(*ID));
}

pub fn player_state_transition(
    entities: Res<Entities>,
    player_inputs: Res<MatchInputs>,
    player_indexes: Comp<PlayerIdx>,
    mut player_states: CompMut<PlayerState>,
    bodies: Comp<KinematicBody>,
) {
    for (_ent, (player_idx, player_state, body)) in
        entities.iter_with((&player_indexes, &mut player_states, &bodies))
    {
        if player_state.current != *ID {
            continue;
        }

        let control = &player_inputs.players[player_idx.0 as usize].control;

        if !body.is_on_ground {
            player_state.current = *midair::ID;
        } else if control.move_direction.y < -0.5 {
            player_state.current = *crouch::ID;
        } else if control.move_direction.x != 0.0 {
            player_state.current = *walk::ID;
        }
    }
}

pub fn handle_player_state(
    entities: Res<Entities>,
    player_inputs: Res<MatchInputs>,
    player_indexes: Comp<PlayerIdx>,
    player_states: Comp<PlayerState>,
    assets: Res<AssetServer>,
    mut sprites: CompMut<AnimationBankSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut audio_center: ResMut<AudioCenter>,
    collision_world: CollisionWorld,
    slippery: CompMut<Slippery>,
) {
    let players = entities.iter_with((&player_states, &player_indexes, &mut sprites, &mut bodies));
    for (player_ent, (player_state, player_idx, animation, body)) in players {
        if player_state.current != *ID {
            continue;
        }
        let meta_handle = player_inputs.players[player_idx.0 as usize].selected_player;
        let meta = assets.get(meta_handle);

        // If this is the first frame of this state
        if player_state.age == 0 {
            // set our animation to idle
            animation.current = "idle".into();
        }

        let control = &player_inputs.players[player_idx.0 as usize].control;

        // If we are jumping
        if control.jump_just_pressed {
            // Play jump sound
            audio_center.play_sound(meta.sounds.jump, meta.sounds.jump_volume);

            // Move up
            body.velocity.y = meta.stats.jump_speed;
        }

        let mut slide_factor = 1.;
        for (slippery_ent, slippery_meta) in entities.iter_with(&slippery) {
            if collision_world
                .actor_collisions(player_ent)
                .contains(&slippery_ent)
            {
                slide_factor = 1. / slippery_meta.player_slide;
            }
        }

        // Since we are idling, slide
        if body.velocity.x != 0.0 {
            if body.velocity.x.is_sign_positive() {
                body.velocity.x = (body.velocity.x - meta.stats.slowdown * slide_factor).max(0.0);
            } else {
                body.velocity.x = (body.velocity.x + meta.stats.slowdown * slide_factor).min(0.0);
            }
        }
    }
}
