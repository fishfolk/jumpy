use crate::{
    networking::proto::{
        player_input::{PlayerInputFromClient, PlayerInputFromServer},
        ClientMatchInfo,
    },
    player::input::PlayerInputs,
    prelude::*,
};

use super::NetClient;

pub struct ClientPlayerInputPlugin;

impl Plugin for ClientPlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            recv_player_input_from_server.run_if_resource_exists::<NetClient>(),
        );
        app.add_system_to_stage(
            CoreStage::Last,
            send_player_input_to_server
                .run_if_resource_exists::<NetClient>()
                .run_if_resource_exists::<ClientMatchInfo>(),
        );
    }
}

fn recv_player_input_from_server(
    mut client: ResMut<Net>,
    mut player_inputs: ResMut<PlayerInputs>,
) {
    // FIXME: Unordered packets not handled correctly. We need a network tick.
    while let Some(message) = client.recv_unreliable::<PlayerInputFromServer>() {
        let player_input = &mut player_inputs.players[message.player_idx as usize];

        player_input.control = message.control;
    }
}

fn send_player_input_to_server(
    client: Res<NetClient>,
    player_inputs: Res<PlayerInputs>,
    match_info: Res<ClientMatchInfo>,
) {
    let control = &player_inputs.players[match_info.player_idx].control;
    client.send_unreliable(&PlayerInputFromClient {
        control: control.clone(),
    });
}
