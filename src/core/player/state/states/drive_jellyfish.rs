use super::*;

pub static ID: Lazy<Ustr> = Lazy::new(|| ustr("core::drive_jellyfish"));

pub fn install(session: &mut Session) {
    PlayerState::add_player_state_transition_system(session, player_state_transition);
}

pub fn player_state_transition(
    entities: Res<Entities>,
    player_driving: Comp<PlayerDrivingJellyfish>,
    mut player_states: CompMut<PlayerState>,
    killed_players: Comp<PlayerKilled>,
) {
    for (player_ent, player_state) in entities.iter_with(&mut player_states) {
        if player_driving.contains(player_ent) {
            if killed_players.contains(player_ent) {
                continue;
            }
            if player_state.current != *ID {
                player_state.current = *ID;
            }
        } else if player_state.current == *ID {
            player_state.current = *idle::ID;
        }
    }
}
