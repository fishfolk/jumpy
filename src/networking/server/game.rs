use crate::{
    networking::proto::game::{GameEventFromServer, GamePlayerEvent},
    prelude::*,
};

use super::{MessageTarget, NetServer};

pub struct ServerGamePlugin;

impl Plugin for ServerGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system(
            handle_client_messages
                .run_if_resource_exists::<NetServer>()
                .run_in_state(GameState::ServerInGame),
        );
    }
}

fn handle_client_messages(mut server: ResMut<NetServer>) {
    while let Some(incomming) = server.recv_reliable::<GamePlayerEvent>() {
        server.send_reliable_to(
            &GameEventFromServer::PlayerEvent {
                player_idx: incomming.client_idx.try_into().unwrap(),
                event: incomming.message,
            },
            MessageTarget::AllExcept(incomming.client_idx),
        )
    }
    while let Some(incomming) = server.recv_unreliable::<GamePlayerEvent>() {
        server.send_unreliable_to(
            &GameEventFromServer::PlayerEvent {
                player_idx: incomming.client_idx.try_into().unwrap(),
                event: incomming.message,
            },
            MessageTarget::AllExcept(incomming.client_idx),
        )
    }
}
