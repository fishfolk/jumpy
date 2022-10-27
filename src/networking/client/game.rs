use crate::{
    animation::AnimationBankSprite,
    networking::proto::{
        game::{GameEventFromServer, GamePlayerEvent, PlayerState},
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
            FixedUpdateStage::First,
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
            client.send_unreliable(&GamePlayerEvent::UpdateState(PlayerState {
                pos: transform.translation,
                sprite: sprite.clone(),
            }));
        }
    }
}

fn handle_server_events(
    mut commands: Commands,
    mut client: ResMut<NetClient>,
    mut players: Query<(Entity, &PlayerIdx, &mut Transform, &mut AnimationBankSprite)>,
) {
    while let Some(event) = client.recv_reliable::<GameEventFromServer>() {
        match event {
            GameEventFromServer::PlayerEvent { player_idx, event } => match event {
                GamePlayerEvent::SpawnPlayer(pos) => {
                    commands
                        .spawn()
                        .insert(PlayerIdx(player_idx as usize))
                        .insert(Transform::from_translation(pos));
                }
                GamePlayerEvent::KillPlayer => {
                    for (entity, idx, _, _) in &mut players {
                        if idx.0 == player_idx as usize {
                            commands.entity(entity).despawn_recursive();
                            break;
                        }
                    }
                }
                _ => (),
            },
        }
    }
    while let Some(event) = client.recv_unreliable::<GameEventFromServer>() {
        match event {
            GameEventFromServer::PlayerEvent { player_idx, event } => {
                if let GamePlayerEvent::UpdateState(state) = event {
                    for (_, idx, mut transform, mut animation_bank_sprite) in &mut players {
                        if idx.0 == player_idx as usize {
                            transform.translation = state.pos;
                            *animation_bank_sprite = state.sprite;
                            break;
                        }
                    }
                }
            }
        }
    }
}
