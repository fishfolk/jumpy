use super::*;

pub static ID: Lazy<Ustr> = Lazy::new(|| ustr("core::crouch"));

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
    assets: Res<AssetServer>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
) {
    for (_ent, (state, player_idx, body, transform)) in entities.iter_with((
        &mut player_states,
        &player_indexes,
        &mut bodies,
        &mut transforms,
    )) {
        let player_input = &player_inputs.players[player_idx.0 as usize];
        let meta_handle = player_input.selected_player;
        let meta = assets.get(meta_handle);

        // Reset the body size and position if we stop sliding
        if state.last == *ID && state.current != *ID {
            if let ColliderShape::Rectangle { size } = &body.shape {
                if *size != meta.body_size {
                    body.shape = ColliderShape::Rectangle {
                        size: meta.body_size,
                    };
                    let offset = (meta.body_size.y - meta.slide_body_size.y) / 2.0;
                    let direction = player_input.control.move_direction.x.signum();
                    transform.translation.x += offset * direction;
                    transform.translation.y += offset;
                }
            }
        }

        if state.current != *ID {
            continue;
        }
        let control = &player_inputs.players[player_idx.0 as usize].control;

        if !body.is_on_ground || control.move_direction.y > -0.5 {
            state.current = *idle::ID;
        }
    }
}

pub fn handle_player_state(
    entities: Res<Entities>,
    player_inputs: Res<MatchInputs>,
    player_states: Comp<PlayerState>,
    player_indexes: Comp<PlayerIdx>,
    mut animations: CompMut<AnimationBankSprite>,
    mut bodies: CompMut<KinematicBody>,
    assets: Res<AssetServer>,
    mut transforms: CompMut<Transform>,
) {
    for (_player_ent, (state, player_idx, animation, body, transform)) in entities.iter_with((
        &player_states,
        &player_indexes,
        &mut animations,
        &mut bodies,
        &mut transforms,
    )) {
        if state.current != *ID {
            continue;
        }
        let player_input = &player_inputs.players[player_idx.0 as usize];
        let meta_handle = player_input.selected_player;
        let meta = assets.get(meta_handle);

        if body.velocity.x == 0.0 {
            animation.current = "crouch".into();
            if let ColliderShape::Rectangle { size } = &body.shape {
                if *size != meta.body_size {
                    body.shape = ColliderShape::Rectangle {
                        size: meta.body_size,
                    };
                    let offset = (meta.body_size.y - meta.slide_body_size.y) / 2.0;
                    let direction = player_input.control.move_direction.x.signum();
                    transform.translation.x += offset * direction;
                    transform.translation.y += offset;
                }
            }
        } else if let ColliderShape::Rectangle { size } = &body.shape {
            animation.current = "slide".into();

            if *size != meta.slide_body_size {
                body.shape = ColliderShape::Rectangle {
                    size: meta.slide_body_size,
                };
                let offset = (meta.body_size.y - meta.slide_body_size.y) / 2.0;
                transform.translation.x -= offset;
                transform.translation.y -= offset;
            }
        }

        let control = &player_inputs.players[player_idx.0 as usize].control;

        if control.jump_just_pressed {
            body.fall_through = true;
        }
    }
}
