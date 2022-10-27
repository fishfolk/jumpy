use crate::{
    animation::AnimationBankSprite,
    networking::proto::{
        game::{PlayerEvent, PlayerEventFromServer, PlayerState, PlayerStateFromServer},
        tick::{ClientTicks, Tick},
        ClientMatchInfo,
    },
    player::PlayerIdx,
    prelude::*,
};

use super::NetClient;

pub struct ClientGamePlugin;

impl Plugin for ClientGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            FixedUpdateStage::Last,
            send_player_state
                .run_if_resource_exists::<NetClient>()
                .run_if_resource_exists::<ClientMatchInfo>(),
        )
        .add_system_to_stage(
            FixedUpdateStage::Last,
            handle_server_events
                .run_if_resource_exists::<NetClient>()
                .run_if_resource_exists::<ClientMatchInfo>(),
        );
    }
}

fn send_player_state(
    client: Res<NetClient>,
    players: Query<(&PlayerIdx, &Transform, &AnimationBankSprite)>,
    match_info: Res<ClientMatchInfo>,
) {
    for (player_idx, transform, sprite) in &players {
        if player_idx.0 == match_info.player_idx {
            client.send_unreliable(&PlayerState {
                tick: Tick::next(),
                pos: transform.translation,
                sprite: sprite.clone(),
            });
        }
    }
}

fn handle_server_events(
    mut client_ticks: Local<ClientTicks>,
    mut commands: Commands,
    mut client: ResMut<NetClient>,
    mut players: Query<(Entity, &PlayerIdx, &mut Transform, &mut AnimationBankSprite)>,
) {
    while let Some(event) = client.recv_reliable::<PlayerEventFromServer>() {
        match event.kind {
            PlayerEvent::SpawnPlayer(pos) => {
                commands
                    .spawn()
                    .insert(PlayerIdx(event.player_idx as usize))
                    .insert(Transform::from_translation(pos));
            }
            PlayerEvent::KillPlayer => {
                for (entity, idx, _, _) in &mut players {
                    if idx.0 == event.player_idx as usize {
                        commands.entity(entity).despawn_recursive();
                        break;
                    }
                }
            }
        }
    }
    while let Some(message) = client.recv_unreliable::<PlayerStateFromServer>() {
        if client_ticks.is_latest(message.player_idx as usize, message.state.tick) {
            for (_, idx, mut transform, mut sprite) in &mut players {
                if idx.0 == message.player_idx as usize {
                    transform.translation = message.state.pos;
                    *sprite = message.state.sprite;
                    break;
                }
            }
        }
    }
}
