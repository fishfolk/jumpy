use rand::{thread_rng, Rng};

use crate::{
    networking::proto::match_setup::{MatchSetupFromClient, MatchSetupFromServer},
    player::{input::PlayerInputs, MAX_PLAYERS},
    prelude::*,
};

use super::NetServer;

pub struct ServerPlayerSelectPlugin;

impl Plugin for ServerPlayerSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            handle_client_messages
                .run_if_resource_exists::<NetServer>()
                .run_in_state(GameState::ServerPlayerSelect),
        );
    }
}

#[derive(Default, Deref, DerefMut)]
struct PlayerConfirmations([bool; MAX_PLAYERS]);

impl PlayerConfirmations {
    fn count(&self) -> usize {
        let mut count = 0;
        for confirmed in &self.0 {
            if *confirmed {
                count += 1;
            }
        }

        count
    }
}

fn handle_client_messages(
    mut players_selected: Local<PlayerConfirmations>,
    mut player_selecting_map: Local<Option<usize>>,
    mut commands: Commands,
    mut server: ResMut<NetServer>,
    mut player_inputs: ResMut<PlayerInputs>,
) {
    while let Some(incomming) = server.recv_reliable::<MatchSetupFromClient>() {
        match &incomming.message {
            MatchSetupFromClient::SelectPlayer(handle) => {
                player_inputs.players[incomming.client_idx].selected_player = handle.clone_weak();
            }
            MatchSetupFromClient::ConfirmSelection(confirmed) => {
                players_selected[incomming.client_idx] = *confirmed;
            }
            MatchSetupFromClient::SelectMap(map_handle) => {
                if let Some(player_selecting_map) = &*player_selecting_map {
                    if *player_selecting_map == incomming.client_idx {
                        // Spawn the map
                        commands.spawn().insert(map_handle.clone_weak());

                        // Start the game
                        commands.insert_resource(NextState(GameState::InGame));
                        commands.insert_resource(NextState(InGameState::Playing));
                    }
                }
            }
        }

        // If the players have finished selecting their fish
        if players_selected.count() == server.client_count() && player_selecting_map.is_none() {
            // Select a random player to pick the map
            let idx = thread_rng().gen_range(0..server.client_count());
            *player_selecting_map = Some(idx);

            server.send_reliable_to(
                &MatchSetupFromServer::WaitForMapSelect,
                super::MessageTarget::AllExcept(idx),
            );
            server.send_reliable(&MatchSetupFromServer::SelectMap, idx);

        // If we are still waiting for players to select fish, map, or get ready
        } else {
            // Forward client message to other clients
            let message = MatchSetupFromServer::ClientMessage {
                player_idx: incomming.client_idx.try_into().unwrap(),
                message: incomming.message,
            };
            server.send_reliable_to(
                &message,
                super::MessageTarget::AllExcept(incomming.client_idx),
            );
        }
    }
}
