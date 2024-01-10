use super::*;

pub static ID: Lazy<Ustr> = Lazy::new(|| ustr("core::drive_jellyfish"));

pub fn install(session: &mut Session) {
    PlayerState::add_player_state_transition_system(session, player_state_transition);
    PlayerState::add_player_state_update_system(session, handle_player_state);
}

pub fn player_state_transition(
    entities: Res<Entities>,
    jellyfishes: Comp<Jellyfish>,
    driving_jellyfishes: Comp<DrivingJellyfish>,
    mut player_states: CompMut<PlayerState>,
    killed_players: Comp<PlayerKilled>,
    player_inventories: PlayerInventories,
) {
    for (jellyfish_ent, _) in entities.iter_with(&jellyfishes) {
        if let Some(driving) = driving_jellyfishes.get(jellyfish_ent) {
            let Some(player_state) = player_states.get_mut(driving.owner) else {
                continue;
            };
            if killed_players.contains(driving.owner) {
                continue;
            }
            if player_state.current != *ID {
                player_state.current = *ID;
            }
        } else {
            let Some(player_state) = player_inventories
                .iter()
                .find_map(|inv| inv.filter(|i| i.inventory == jellyfish_ent))
                .and_then(|inventory| player_states.get_mut(inventory.player))
            else {
                continue;
            };
            if player_state.current == *ID {
                player_state.current = *idle::ID;
            }
        }
    }
}

pub fn handle_player_state(
    entities: Res<Entities>,
    player_indexes: Comp<PlayerIdx>,
    player_states: Comp<PlayerState>,
    inventories: Comp<Inventory>,
    player_inputs: Res<MatchInputs>,
    mut commands: Commands,
) {
    for (player_ent, (player_idx, player_state, inventory)) in
        entities.iter_with((&player_indexes, &player_states, &inventories))
    {
        if player_state.current != *ID {
            continue;
        }

        let control = &player_inputs.players[player_idx.0 as usize].control;

        if control.shoot_just_pressed && inventory.is_some() {
            commands.add(PlayerCommand::use_item(player_ent));
        }
    }
}
