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
) {
    for (_ent, (state, player_idx, animation, body)) in entities.iter_with((
        &player_states,
        &player_indexes,
        &mut animations,
        &mut bodies,
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
    }
}
