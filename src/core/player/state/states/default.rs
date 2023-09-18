use super::*;

pub fn install(session: &mut Session) {
    PlayerState::add_player_state_transition_system(session, player_state_transition);
}

pub fn player_state_transition(entities: Res<Entities>, mut player_states: CompMut<PlayerState>) {
    for (_ent, state) in entities.iter_with(&mut player_states) {
        // If the current state is the default, meaningless state
        if state.current.is_empty() {
            state.current = *idle::ID;
        }
    }
}
