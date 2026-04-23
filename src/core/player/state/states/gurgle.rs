use super::*;

pub static ID: Lazy<Ustr> = Lazy::new(|| ustr("core::gurgle"));

const GURGLE_DURATION: u64 = 60;

pub fn install(session: &mut SessionBuilder) {
    PlayerState::add_player_state_transition_system(session, player_state_transition);
    PlayerState::add_player_state_update_system(session, handle_player_state);
}

pub fn player_state_transition(
    entities: Res<Entities>,
    player_inputs: Res<MatchInputs>,
    player_indexes: Comp<PlayerIdx>,
    mut player_states: CompMut<PlayerState>,
) {
    for (_ent, (player_idx, state)) in entities.iter_with((&player_indexes, &mut player_states)) {
        let control = &player_inputs.players[player_idx.0 as usize].control;

        if control.gurgle_just_pressed && state.current != *ID {
            state.current = *ID;
        }

        if state.current == *ID && state.age >= GURGLE_DURATION {
            state.current = *idle::ID;
        }
    }
}

pub fn handle_player_state(
    entities: Res<Entities>,
    player_states: Comp<PlayerState>,
    mut animations: CompMut<AnimationBankSprite>,
    game_meta: Root<GameMeta>,
    mut audio_center: ResMut<AudioCenter>,
) {
    for (_ent, (state, animation)) in entities.iter_with((&player_states, &mut animations)) {
        if state.current != *ID {
            continue;
        }

        if state.age == 0 {
            animation.current = "idle".into();
            audio_center.play_sound(game_meta.music.gurgle, 1.0);
        }
    }
}
