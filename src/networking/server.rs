use std::{any::TypeId, collections::VecDeque, time::Instant};

use crate::{networking::proto, player::input::PlayerInputs};
use async_channel::{Receiver, RecvError, Sender};
use bevy::{app::AppExit, tasks::IoTaskPool, utils::HashMap};
use bytes::Bytes;
use futures_lite::future;
use quinn::Connection;
use serde::de::DeserializeOwned;

use crate::prelude::*;

use super::NET_MESSAGE_TYPES;

mod game;
pub mod match_setup;
// pub mod player_input;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(match_setup::ServerPlayerSelectPlugin)
            .add_plugin(game::ServerGamePlugin)
            // .add_plugin(player_input::ServerPlayerInputPlugin)
            .add_startup_system(spawn_message_recv_tasks)
            .add_startup_system(spawn_message_send_task)
            .add_system_to_stage(CoreStage::First, exit_on_disconnect)
            .add_system(reply_to_ping);

        app.world.resource_scope(|world, server: Mut<NetServer>| {
            let mut player_inputs = world.resource_mut::<PlayerInputs>();

            for i in 0..server.client_count() {
                player_inputs.players[i].active = true;
            }
        });
    }
}

fn reply_to_ping(mut server: ResMut<NetServer>) {
    while let Some(incomming) = server.recv_reliable::<proto::Ping>() {
        let client_idx = incomming.client_idx;
        info!("Ping from client {client_idx}");
        server.send_reliable(&proto::Pong, client_idx);
    }
}

#[derive(Debug, Clone)]
pub struct NetServer {
    clients: Vec<Connection>,
    outgoing_reliable_sender: Sender<Outgoing>,
    outgoing_reliable_receiver: Receiver<Outgoing>,
    outgoing_unreliable_sender: Sender<Outgoing>,
    outgoing_unreliable_receiver: Receiver<Outgoing>,
    incomming_reliable_sender: Sender<Incomming>,
    incomming_reliable_receiver: Receiver<Incomming>,
    incomming_unreliable_sender: Sender<Incomming>,
    incomming_unreliable_receiver: Receiver<Incomming>,
    incomming_reliable_queue: HashMap<TypeId, VecDeque<Incomming>>,
    incomming_unreliable_queue: HashMap<TypeId, VecDeque<Incomming>>,
}

#[derive(Debug)]
struct Outgoing {
    data: Vec<u8>,
    target: MessageTarget,
}

#[derive(Debug, Clone)]
struct Incomming {
    data: Vec<u8>,
    client_idx: usize,
}

#[derive(Debug, Clone)]
pub enum MessageTarget {
    All,
    AllExcept(usize),
    Client(usize),
}

impl NetServer {
    pub fn new(clients: Vec<Connection>) -> Self {
        let (outgoing_reliable_sender, outgoing_reliable_receiver) = async_channel::unbounded();
        let (outgoing_unreliable_sender, outgoing_unreliable_receiver) = async_channel::unbounded();
        let (incomming_reliable_sender, incomming_reliable_receiver) = async_channel::unbounded();
        let (incomming_unreliable_sender, incomming_unreliable_receiver) =
            async_channel::unbounded();

        Self {
            clients,
            outgoing_reliable_sender,
            outgoing_reliable_receiver,
            outgoing_unreliable_sender,
            outgoing_unreliable_receiver,
            incomming_reliable_sender,
            incomming_reliable_receiver,
            incomming_unreliable_sender,
            incomming_unreliable_receiver,
            incomming_reliable_queue: default(),
            incomming_unreliable_queue: default(),
        }
    }

    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Update the incomming message queue
    fn update_queue(&mut self) {
        while let Ok(mut incomming) = self.incomming_reliable_receiver.try_recv() {
            let type_idx_bytes: [u8; 4] = incomming
                .data
                .split_off(incomming.data.len() - 4)
                .try_into()
                .unwrap();
            let type_idx = u32::from_le_bytes(type_idx_bytes);
            let type_id = NET_MESSAGE_TYPES[type_idx as usize];
            self.incomming_reliable_queue
                .entry(type_id)
                .or_default()
                .push_back(incomming);
        }
        while let Ok(mut incomming) = self.incomming_unreliable_receiver.try_recv() {
            let type_idx_bytes: [u8; 4] = incomming
                .data
                .split_off(incomming.data.len() - 4)
                .try_into()
                .unwrap();
            let type_idx = u32::from_le_bytes(type_idx_bytes);
            let type_id = NET_MESSAGE_TYPES[type_idx as usize];
            self.incomming_unreliable_queue
                .entry(type_id)
                .or_default()
                .push_back(incomming);
        }
    }

    pub fn send_reliable_to<S: 'static + Serialize>(&self, message: &S, target: MessageTarget) {
        let type_id = TypeId::of::<S>();
        let type_idx = NET_MESSAGE_TYPES
            .iter()
            .position(|x| x == &type_id)
            .expect("Net message not registered") as u32;
        let mut message = postcard::to_allocvec(message).expect("Serialize net message");
        message.extend_from_slice(&(type_idx as u32).to_le_bytes());
        self.outgoing_reliable_sender
            .try_send(Outgoing {
                data: message,
                target,
            })
            .ok();
    }

    pub fn send_unreliable_to<S: 'static + Serialize>(&self, message: &S, target: MessageTarget) {
        let type_id = TypeId::of::<S>();
        let type_idx = NET_MESSAGE_TYPES
            .iter()
            .position(|x| x == &type_id)
            .expect("Net message not registered") as u32;
        let mut message = postcard::to_allocvec(message).expect("Serialize net message");
        message.extend_from_slice(&(type_idx as u32).to_le_bytes());

        self.outgoing_unreliable_sender
            .try_send(Outgoing {
                data: message,
                target,
            })
            .ok();
    }

    pub fn send_reliable<S: 'static + Serialize>(&self, message: &S, client_idx: usize) {
        self.send_reliable_to(message, MessageTarget::Client(client_idx));
    }

    pub fn send_unreliable<S: 'static + Serialize>(&self, message: &S, client_idx: usize) {
        self.send_unreliable_to(message, MessageTarget::Client(client_idx));
    }

    pub fn broadcast_reliable<S: 'static + Serialize>(&self, message: &S) {
        self.send_reliable_to(message, MessageTarget::All);
    }

    pub fn broadcast_unreliable<S: 'static + Serialize>(&self, message: &S) {
        self.send_unreliable_to(message, MessageTarget::All);
    }

    pub fn recv_reliable<T: 'static + DeserializeOwned>(&mut self) -> Option<IncommingMessage<T>> {
        let type_id = TypeId::of::<T>();
        if !NET_MESSAGE_TYPES.contains(&type_id) {
            panic!("Attempt to receive unregistered message type");
        }
        self.update_queue();
        self.incomming_reliable_queue
            .get_mut(&type_id)
            .and_then(|queue| queue.pop_front())
            .map(|incomming| IncommingMessage {
                message: postcard::from_bytes(&incomming.data).expect("Deserialize net message"),
                client_idx: incomming.client_idx,
            })
    }

    pub fn recv_unreliable<T: 'static + DeserializeOwned>(
        &mut self,
    ) -> Option<IncommingMessage<T>> {
        let type_id = TypeId::of::<T>();
        if !NET_MESSAGE_TYPES.contains(&type_id) {
            panic!("Attempt to receive unregistered message type");
        }
        self.update_queue();
        self.incomming_unreliable_queue
            .get_mut(&type_id)
            .and_then(|queue| queue.pop_front())
            .map(|incomming| IncommingMessage {
                message: postcard::from_bytes(&incomming.data).expect("Deserialize net message"),
                client_idx: incomming.client_idx,
            })
    }
}

pub struct IncommingMessage<T> {
    pub message: T,
    pub client_idx: usize,
}

fn spawn_message_send_task(server: Res<NetServer>) {
    let io_pool = IoTaskPool::get();

    let clients = server.clients.clone();
    let outgoing_reliable_receiver = server.outgoing_reliable_receiver.clone();
    let outgoing_unreliable_receiver = server.outgoing_unreliable_receiver.clone();
    io_pool
        .spawn(async move {
            loop {
                let handle_reliable_message = async {
                    loop {
                        let message = outgoing_reliable_receiver.recv().await?;
                        let data = Bytes::from(message.data);

                        let targets = match message.target {
                            MessageTarget::All => clients.iter().collect::<Vec<_>>(),
                            MessageTarget::AllExcept(idx) => clients
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| i != &idx)
                                .map(|(_, x)| x)
                                .collect::<Vec<_>>(),
                            MessageTarget::Client(idx) => {
                                [&clients[idx]].into_iter().collect::<Vec<_>>()
                            }
                        };

                        // Broadcast reliable messages to clients
                        for conn in targets {
                            let message_ = data.clone();

                            let result = async {
                                let mut sender = conn.open_uni().await?;

                                sender.write_all(&message_).await?;
                                sender.finish().await?;

                                Ok::<(), anyhow::Error>(())
                            };

                            if let Err(e) = result.await {
                                error!("Error sending reliable message: {e:?}");
                            }
                        }
                    }

                    // This is needed to annotate the return type of the block
                    #[allow(unreachable_code)]
                    Ok::<(), RecvError>(())
                };

                let handle_unreliable_message = async {
                    loop {
                        let message = outgoing_unreliable_receiver.recv().await?;
                        let data = Bytes::from(message.data);

                        let targets = match message.target {
                            MessageTarget::All => clients.iter().collect::<Vec<_>>(),
                            MessageTarget::AllExcept(idx) => clients
                                .iter()
                                .enumerate()
                                .filter(|(i, _)| i != &idx)
                                .map(|(_, x)| x)
                                .collect::<Vec<_>>(),
                            MessageTarget::Client(idx) => {
                                [&clients[idx]].into_iter().collect::<Vec<_>>()
                            }
                        };

                        // Broadcast unreliable messages to clients
                        for conn in targets {
                            let message_ = data.clone();
                            let result = conn.send_datagram(message_);

                            if let Err(e) = result {
                                error!("Error sending unreliable message: {e:?}");
                            }
                        }
                    }

                    // This is needed to annotate the return type of the block
                    #[allow(unreachable_code)]
                    Ok::<(), RecvError>(())
                };

                if future::race(handle_reliable_message, handle_unreliable_message)
                    .await
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
}

fn spawn_message_recv_tasks(server: Res<NetServer>) {
    let io_pool = IoTaskPool::get();

    for (client_idx, conn) in server.clients.iter().enumerate() {
        let reliable_sender = server.incomming_reliable_sender.clone();
        let unreliable_sender = server.incomming_unreliable_sender.clone();
        let conn = conn.clone();

        io_pool
            .spawn(async move {
                'connection: loop {
                    let receive_message_result = async {
                        future::zip(
                            async {
                                while let Ok(recv) = conn.accept_uni().await {
                                    let start = Instant::now();
                                    let message = recv.read_to_end(usize::MAX).await?;
                                    let time = Instant::now() - start;
                                    let mib = message.len() as f32 / 1024.0 / 1024.0;
                                    trace!(
                                        "Got message of {} MiB: {} MiB/s",
                                        mib,
                                        mib as f32 / time.as_secs_f32()
                                    );
                                    reliable_sender
                                        .try_send(Incomming {
                                            data: message,
                                            client_idx,
                                        })
                                        .ok();
                                }

                                Ok::<(), anyhow::Error>(())
                            },
                            async {
                                while let Ok(message) = conn.read_datagram().await {
                                    unreliable_sender
                                        .try_send(Incomming {
                                            data: message.to_vec(),
                                            client_idx,
                                        })
                                        .ok();
                                }
                            },
                        )
                        .await
                        .0?;

                        Ok::<(), anyhow::Error>(())
                    };

                    let connection_closed = conn.closed();

                    let event = future::or(
                        async move { either::Left(connection_closed.await) },
                        async move { either::Right(receive_message_result.await) },
                    )
                    .await;

                    match event {
                        either::Either::Left(closed) => {
                            debug!("Client connection closed: {closed:?}");
                            break 'connection;
                        }
                        either::Either::Right(message_result) => {
                            if let Err(e) = message_result {
                                error!("Error receiving net messages: {e:?}");
                            }
                        }
                    }
                }
            })
            .detach();
    }
}

fn exit_on_disconnect(mut server: ResMut<NetServer>, mut exit_sender: EventWriter<AppExit>) {
    // Remove disconnected clients
    server.clients.retain(|conn| conn.close_reason().is_none());

    // If all clients have disconnected, exit the app
    if server.clients.is_empty() {
        info!("All clients disconnected from match");
        exit_sender.send_default();
    }
}
