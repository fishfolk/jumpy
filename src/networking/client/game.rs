use std::time::Duration;

use bevy_tweening::{lens::TransformPositionLens, Animator, EaseMethod, Tween, TweeningType};

use crate::{
    animation::AnimationBankSprite,
    item::{Item, ItemDropEvent, ItemGrabEvent, ItemUseEvent},
    networking::{
        proto::{
            game::{
                GameEventFromServer, PlayerEvent, PlayerEventFromServer, PlayerState,
                PlayerStateFromServer,
            },
            tick::{ClientTicks, Tick},
            ClientMatchInfo,
        },
        NetIdMap,
    },
    physics::KinematicBody,
    player::{
        PlayerDespawnCommand, PlayerDespawnEvent, PlayerIdx, PlayerKillCommand, PlayerKillEvent,
        PlayerSetInventoryCommand, PlayerUseItemCommand,
    },
    prelude::*,
    FIXED_TIMESTEP,
};

use super::NetClient;

pub struct ClientGamePlugin;

impl Plugin for ClientGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            FixedUpdateStage::Last,
            send_game_events
                .chain(send_player_state)
                .run_if_resource_exists::<NetClient>()
                .run_if_resource_exists::<ClientMatchInfo>(),
        )
        .add_system_to_stage(
            FixedUpdateStage::First,
            handle_game_events_from_server
                .chain(handle_player_state)
                .run_if_resource_exists::<NetClient>()
                .run_if_resource_exists::<ClientMatchInfo>(),
        );
    }
}

fn send_game_events(
    mut player_kill_events: EventReader<PlayerKillEvent>,
    mut player_despawn_events: EventReader<PlayerDespawnEvent>,
    mut item_grab_events: EventReader<ItemGrabEvent>,
    mut item_drop_events: EventReader<ItemDropEvent>,
    mut item_use_events: EventReader<ItemUseEvent>,
    players: Query<(&PlayerIdx, &Transform, &KinematicBody)>,
    client: Res<NetClient>,
    client_info: Res<ClientMatchInfo>,
    net_ids: Res<NetIdMap>,
) {
    for event in item_grab_events.iter() {
        if let Ok((player_idx, ..)) = players.get(event.player) {
            // As the client, we're only allowed to drop and pick up items for our own player.
            if client_info.player_idx == player_idx.0 {
                let net_id = net_ids
                    .get_net_id(event.item)
                    .expect("Item in network game without NetId");
                client.send_reliable(&PlayerEvent::GrabItem(net_id));
            }
        }
    }

    for event in item_drop_events.iter() {
        if let Ok((player_idx, player_transform, body)) = players.get(event.player) {
            // As the client, we're only allowed to drop and pick up items for our own player.
            if client_info.player_idx == player_idx.0 {
                client.send_reliable(&PlayerEvent::DropItem {
                    position: player_transform.translation,
                    velocity: body.velocity,
                });
            }
        }
    }

    for event in item_use_events.iter() {
        if let Ok((player_idx, ..)) = players.get(event.player) {
            // As the client, we're only allowed to drop and pick up items for our own player.
            if client_info.player_idx == player_idx.0 {
                let item_id = net_ids
                    .get_net_id(event.item)
                    .expect("Item in network game without NetId");
                client.send_reliable(&PlayerEvent::UseItem {
                    position: event.position,
                    item: item_id,
                });
            }
        }
    }

    for event in player_kill_events.iter() {
        if let Ok((player_idx, ..)) = players.get(event.player) {
            // As the client, we're only allowed to kill our own player
            if client_info.player_idx == player_idx.0 {
                client.send_reliable(&PlayerEvent::KillPlayer {
                    position: event.position,
                    velocity: event.velocity,
                });
            }
        } else {
            warn!("Received kill event for player that isn't found");
        }
    }

    for event in player_despawn_events.iter() {
        // As the client, we're only able to despawn our own player
        if client_info.player_idx == event.player_idx {
            client.send_reliable(&PlayerEvent::DespawnPlayer);
        }
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

fn handle_game_events_from_server(
    mut commands: Commands,
    mut client: ResMut<NetClient>,
    players: Query<(Entity, &PlayerIdx)>,
    mut net_ids: ResMut<NetIdMap>,
) {
    while let Some(event) = client.recv_reliable::<PlayerEventFromServer>() {
        let player_ent = players
            .iter()
            .find(|x| x.1 .0 == event.player_idx as usize)
            .map(|x| x.0);

        match event.kind {
            PlayerEvent::SpawnPlayer(pos) => {
                commands
                    .spawn()
                    .insert(PlayerIdx(event.player_idx as usize))
                    .insert(Transform::from_translation(pos));
            }
            PlayerEvent::KillPlayer { position, velocity } => {
                if let Some(player_ent) = player_ent {
                    commands.add(PlayerKillCommand {
                        player: player_ent,
                        position: Some(position),
                        velocity: Some(velocity),
                    });
                } else {
                    warn!(?event.player_idx, "Net event to kill player that doesn't exist locally");
                }
            }
            PlayerEvent::DespawnPlayer => {
                if let Some(player_ent) = player_ent {
                    commands.add(PlayerDespawnCommand::new(player_ent));
                } else {
                    warn!(?event.player_idx, "Net event to despawn player that doesn't exist locally");
                }
            }
            PlayerEvent::GrabItem(net_id) => {
                if let Some(player_ent) = player_ent {
                    if let Some(item_ent) = net_ids.get_entity(net_id) {
                        commands.add(PlayerSetInventoryCommand {
                            player: player_ent,
                            item: Some(item_ent),
                            position: None,
                            velocity: None,
                        });
                    } else {
                        warn!(
                            "Trying to grab item but could not find local item with given net ID"
                        );
                    }
                } else {
                    warn!(?event.player_idx, "Net event to kill player that doesn't exist locally");
                }
            }
            PlayerEvent::DropItem { position, velocity } => {
                if let Some(player_ent) = player_ent {
                    commands.add(PlayerSetInventoryCommand {
                        player: player_ent,
                        item: None,
                        position: Some(position),
                        velocity: Some(velocity),
                    });
                } else {
                    warn!(?event.player_idx, "Net event to kill player that doesn't exist locally");
                }
            }
            PlayerEvent::UseItem { position, item } => {
                if let Some(player_ent) = player_ent {
                    if let Some(item_ent) = net_ids.get_entity(item) {
                        commands.add(PlayerUseItemCommand {
                            player: player_ent,
                            position: Some(position),
                            item: Some(item_ent),
                        });
                    } else {
                        warn!(
                            "Trying to use item but could not find entity for item with given ID"
                        );
                    }
                } else {
                    warn!(?event.player_idx, "Net event to kill player that doesn't exist locally");
                }
            }
        }
    }
    while let Some(event) = client.recv_reliable::<GameEventFromServer>() {
        match event {
            GameEventFromServer::SpawnItem {
                net_id,
                script,
                pos,
            } => {
                let mut item = commands.spawn();
                net_ids.insert(item.id(), net_id);
                item.insert(Transform::from_translation(pos))
                    .insert(GlobalTransform::default())
                    .insert(Item { script })
                    .insert_bundle(VisibilityBundle::default());
            }
        }
    }
}

fn handle_player_state(
    mut client_ticks: Local<ClientTicks>,
    mut client: ResMut<NetClient>,
    mut players: Query<(
        Entity,
        &PlayerIdx,
        &Transform,
        &mut Animator<Transform>,
        &mut AnimationBankSprite,
    )>,
) {
    while let Some(message) = client.recv_unreliable::<PlayerStateFromServer>() {
        if client_ticks.is_latest(message.player_idx as usize, message.state.tick) {
            for (_, idx, transform, mut animator, mut sprite) in &mut players {
                if idx.0 == message.player_idx as usize {
                    animator.set_tweenable(Tween::new(
                        EaseMethod::Linear,
                        TweeningType::Once,
                        Duration::from_secs_f64(FIXED_TIMESTEP * 2.0),
                        TransformPositionLens {
                            start: transform.translation,
                            end: message.state.pos,
                        },
                    ));
                    *sprite = message.state.sprite;
                    break;
                }
            }
        }
    }
}
