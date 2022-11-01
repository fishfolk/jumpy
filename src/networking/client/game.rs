use std::time::Duration;

use bevy_tweening::{lens::TransformPositionLens, Animator, EaseMethod, Tween, TweeningType};

use crate::{
    animation::AnimationBankSprite,
    item::{Item, ItemDropEvent, ItemGrabEvent},
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
    player::PlayerIdx,
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
            handle_game_events
                .chain(handle_player_state)
                .run_if_resource_exists::<NetClient>()
                .run_if_resource_exists::<ClientMatchInfo>(),
        );
    }
}

fn send_game_events(
    mut grab_events: EventReader<ItemGrabEvent>,
    mut drop_events: EventReader<ItemDropEvent>,
    players: Query<(&PlayerIdx, &Transform)>,
    client: Res<NetClient>,
    client_info: Res<ClientMatchInfo>,
    net_ids: Res<NetIdMap>,
) {
    for event in grab_events.iter() {
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

    for event in drop_events.iter() {
        if let Ok((player_idx, player_transform)) = players.get(event.player) {
            // As the client, we're only allowed to drop and pick up items for our own player.
            if client_info.player_idx == player_idx.0 {
                client.send_reliable(&PlayerEvent::DropItem(player_transform.translation));
            }
        }
    }
}

fn send_player_state(
    client: Res<NetClient>,
    players: Query<(&PlayerIdx, &Transform, &AnimationBankSprite, &KinematicBody)>,
    match_info: Res<ClientMatchInfo>,
) {
    for (player_idx, transform, sprite, body) in &players {
        if player_idx.0 == match_info.player_idx {
            client.send_unreliable(&PlayerState {
                tick: Tick::next(),
                pos: transform.translation,
                velocity: body.velocity,
                sprite: sprite.clone(),
            });
        }
    }
}

fn handle_game_events(
    mut commands: Commands,
    mut client: ResMut<NetClient>,
    mut players: Query<(Entity, &PlayerIdx, &Transform, Option<&Children>), Without<Item>>,
    mut items: Query<&mut Transform, With<Item>>,
    mut net_ids: ResMut<NetIdMap>,
    mut item_grab_events: EventWriter<ItemGrabEvent>,
    mut item_drop_events: EventWriter<ItemDropEvent>,
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
                for (entity, idx, ..) in &mut players {
                    if idx.0 == event.player_idx as usize {
                        commands.entity(entity).despawn_recursive();
                        break;
                    }
                }
            }
            PlayerEvent::GrabItem(net_id) => {
                if let Some(item_ent) = net_ids.get_entity(net_id) {
                    if let Some((player_ent, _idx, transform, ..)) = players
                        .iter()
                        .find(|(_, player_idx, ..)| player_idx.0 == event.player_idx as usize)
                    {
                        item_grab_events.send(ItemGrabEvent {
                            player: player_ent,
                            item: item_ent,
                            position: transform.translation,
                        });
                        commands.entity(player_ent).push_children(&[item_ent]);
                    } else {
                        warn!("Dead player grabbed item??");
                    }
                } else {
                    warn!("No entity found for Net ID");
                }
            }
            PlayerEvent::DropItem(pos) => {
                if let Some((player_ent, _idx, _trans, children)) = players
                    .iter()
                    .find(|(_, player_idx, ..)| player_idx.0 == event.player_idx as usize)
                {
                    if let Some(children) = children {
                        for child in children {
                            if let Ok(mut item_transform) = items.get_mut(*child) {
                                item_transform.translation = pos;
                                item_drop_events.send(ItemDropEvent {
                                    player: player_ent,
                                    item: *child,
                                    position: pos,
                                });
                                commands.entity(player_ent).remove_children(&[*child]);
                            }
                        }
                    } else {
                        warn!("Dropping item for player not carrying any");
                    }
                } else {
                    warn!(?event.player_idx, "Trying to drop item for dead player??");
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
        &mut KinematicBody,
    )>,
) {
    while let Some(message) = client.recv_unreliable::<PlayerStateFromServer>() {
        if client_ticks.is_latest(message.player_idx as usize, message.state.tick) {
            for (_, idx, transform, mut animator, mut sprite, mut body) in &mut players {
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
                    body.velocity = message.state.velocity;
                    break;
                }
            }
        }
    }
}
