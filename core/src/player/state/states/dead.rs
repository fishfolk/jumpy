use super::*;

pub const ID: Key = key!("core::dead");

pub fn player_state_transition(
    entities: Res<Entities>,
    killed_players: Comp<PlayerKilled>,
    mut player_states: CompMut<PlayerState>,
) {
    for (_ent, (state, _killed)) in entities.iter_with((&mut player_states, &killed_players)) {
        state.current = ID;
    }
}

pub fn handle_player_state(
    entities: Res<Entities>,
    player_states: Comp<PlayerState>,
    mut animations: CompMut<AnimationBankSprite>,
    mut player_events: ResMut<PlayerEvents>,
) {
    for (player_ent, (state, animation)) in entities.iter_with((&player_states, &mut animations)) {
        if state.current != ID {
            continue;
        };

        if state.age == 0 {
            animation.current = key!("death_1");
        }

        if state.age >= 80 {
            player_events.despawn(player_ent);
        }
    }
}
