use super::*;

pub fn player_state_transition(entities: Res<Entities>, mut player_states: CompMut<PlayerState>) {
    for (_ent, state) in entities.iter_with(&mut player_states) {
        // If the current state is the default, meaningless state
        if state.current == default() {
            state.current = idle::ID;
        }
    }
}

// We don't do anything for the default state
pub fn handle_player_state() {}
