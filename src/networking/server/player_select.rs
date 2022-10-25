use crate::{
    networking::proto::player_select::{PlayerSelectFromClient, PlayerSelectFromServer},
    player::{input::PlayerInputs, MAX_PLAYERS},
    prelude::*,
};

use super::NetServer;

pub struct ServerPlayerSelectPlugin;

impl Plugin for ServerPlayerSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_client_messages.run_if_resource_exists::<NetServer>());
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
    mut confirmations: Local<PlayerConfirmations>,
    mut commands: Commands,
    mut server: ResMut<NetServer>,
    mut player_inputs: ResMut<PlayerInputs>,
) {
    while let Some(incomming) = server.recv_reliable::<PlayerSelectFromClient>() {
        match &incomming.message {
            PlayerSelectFromClient::SelectPlayer(handle) => {
                player_inputs.players[incomming.client_idx].selected_player = handle.clone_weak();
            }
            PlayerSelectFromClient::ConfirmSelection(confirmed) => {
                confirmations[incomming.client_idx] = *confirmed;
            }
        }

        if confirmations.count() == server.client_count() {
            server.broadcast_reliable(&PlayerSelectFromServer::StartGame);
            // TODO: Spawn map
            commands.insert_resource(());
        } else {
            let message = PlayerSelectFromServer::PlayerSelection {
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
