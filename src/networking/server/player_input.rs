use crate::{
    networking::proto::player_input::{PlayerInputFromClient, PlayerInputFromServer},
    player::input::PlayerInputs,
    prelude::*,
};

use super::NetServer;

pub struct ServerPlayerInputPlugin;

impl Plugin for ServerPlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::First, recv_player_input);
    }
}

fn recv_player_input(mut server: ResMut<NetServer>, mut player_inputs: ResMut<PlayerInputs>) {
    // FIXME: We are not ordering packets, so we may apply a stale user input, which is not good. We
    // need to track a network tick to know what the latest input really is.
    while let Some(incomming) = server.recv_unreliable::<PlayerInputFromClient>() {
        let input = &mut player_inputs.players[incomming.client_idx];

        input.control = incomming.message.control.clone();

        // FIXME: We may receive multiple updates for each user, so we may send multiple updates to
        // the client, but we don't want to do that, we should just send one update per changed client.
        server.send_unreliable_to(
            &PlayerInputFromServer {
                player_idx: incomming.client_idx.try_into().unwrap(),
                control: incomming.message.control,
            },
            super::MessageTarget::AllExcept(incomming.client_idx),
        )
    }
}
