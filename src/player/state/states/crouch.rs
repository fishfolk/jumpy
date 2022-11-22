use super::*;

pub const ID: &str = "core:crouch";

pub fn player_state_transition(
    player_inputs: Res<PlayerInputs>,
    mut players: Query<(&mut PlayerState, &PlayerIdx, &KinematicBody)>,
) {
    for (mut player_state, player_idx, body) in &mut players {
        if player_state.id != ID {
            continue;
        }

        let control = &player_inputs.players[player_idx.0].control;

        if !body.is_on_ground || control.move_direction.y > -0.5 {
            player_state.id = idle::ID.into();
        }
    }
}

pub fn handle_player_state(
    player_inputs: Res<PlayerInputs>,
    mut players: Query<(
        &PlayerState,
        &PlayerIdx,
        &mut AnimationBankSprite,
        &mut KinematicBody,
    )>,
) {
    for (player_state, player_idx, mut sprite, mut body) in &mut players {
        if player_state.id != ID {
            continue;
        }

        // Set animation
        if player_state.age == 0 {
            sprite.current_animation = "crouch".into();
        }

        let control = &player_inputs.players[player_idx.0].control;

        if control.jump_just_pressed {
            body.fall_through = true;
        }
    }
}
