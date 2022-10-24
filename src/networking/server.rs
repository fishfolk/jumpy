use crate::networking::{Connection, NetServerMessage};
use bevy::{app::AppExit, tasks::IoTaskPool};
use bytes::Bytes;
use flume::{Receiver, Sender};
use futures_lite::future;
use serde::de::DeserializeOwned;

use crate::prelude::*;

use super::NetClientMessage;

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            exit_on_disconnect.run_if_resource_exists::<NetServer>(),
        )
        .add_system_to_stage(
            CoreStage::Last,
            send_messages.run_if_resource_exists::<NetServer>(),
        )
        .add_system_to_stage(
            CoreStage::Last,
            receive_messages.run_if_resource_exists::<NetServer>(),
        )
        .add_system(reply_to_messages.run_if_resource_exists::<NetServer>());
    }
}

fn reply_to_messages(server: Res<NetServer>) {
    while let Some(incomming) = server.recv_reliable::<NetClientMessage>() {
        let client_idx = incomming.client_idx;
        match incomming.message {
            NetClientMessage::Ping => {
                info!("Ping from client {client_idx}");
                server.send_reliable(&NetServerMessage::Pong, client_idx);
            }
        }
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
}

struct Outgoing {
    data: Vec<u8>,
    target: MessageTarget,
}

struct Incomming {
    data: Vec<u8>,
    client_idx: usize,
}

enum MessageTarget {
    All,
    Client(usize),
}

impl NetServer {
    pub fn new(clients: Vec<Connection>) -> Self {
        let (outgoing_reliable_sender, outgoing_reliable_receiver) = flume::unbounded();
        let (outgoing_unreliable_sender, outgoing_unreliable_receiver) = flume::unbounded();
        let (incomming_reliable_sender, incomming_reliable_receiver) = flume::unbounded();
        let (incomming_unreliable_sender, incomming_unreliable_receiver) = flume::unbounded();

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
        }
    }

    pub fn send_reliable<S: Serialize>(&self, message: &S, client_idx: usize) {
        let message = postcard::to_allocvec(message).expect("Serialize net message");
        self.outgoing_reliable_sender
            .try_send(Outgoing {
                data: message,
                target: MessageTarget::Client(client_idx),
            })
            .ok();
    }

    pub fn send_unreliable<S: Serialize>(&self, message: &S, client_idx: usize) {
        let message = postcard::to_allocvec(message).expect("Serialize net message");
        self.outgoing_unreliable_sender
            .try_send(Outgoing {
                data: message,
                target: MessageTarget::Client(client_idx),
            })
            .ok();
    }

    pub fn broadcast_reliable<S: Serialize>(&self, message: &S) {
        let message = postcard::to_allocvec(message).expect("Serialize net message");
        self.outgoing_reliable_sender
            .try_send(Outgoing {
                data: message,
                target: MessageTarget::All,
            })
            .ok();
    }

    pub fn broadcast_unreliable<S: Serialize>(&self, message: &S) {
        let message = postcard::to_allocvec(message).expect("Serialize net message");
        self.outgoing_unreliable_sender
            .try_send(Outgoing {
                data: message,
                target: MessageTarget::All,
            })
            .ok();
    }

    pub fn recv_reliable<D: DeserializeOwned>(&self) -> Option<IncommingMessage<D>> {
        self.incomming_reliable_receiver
            .try_recv()
            .map(|incomming| IncommingMessage {
                message: postcard::from_bytes(&incomming.data).expect("Deserialize net message"),
                client_idx: incomming.client_idx,
            })
            .ok()
    }

    pub fn recv_unreliable<D: DeserializeOwned>(&self) -> Option<IncommingMessage<D>> {
        self.incomming_unreliable_receiver
            .try_recv()
            .map(|incomming| IncommingMessage {
                message: postcard::from_bytes(&incomming.data).expect("Deserialize net message"),
                client_idx: incomming.client_idx,
            })
            .ok()
    }
}

pub struct IncommingMessage<T> {
    pub message: T,
    pub client_idx: usize,
}

fn send_messages(server: Res<NetServer>) {
    let io_pool = IoTaskPool::get();

    while let Ok(message) = server.outgoing_reliable_receiver.try_recv() {
        let data = Bytes::from(message.data);

        let targets = match message.target {
            MessageTarget::All => server.clients.iter().collect::<Vec<_>>(),
            MessageTarget::Client(idx) => [&server.clients[idx]].into_iter().collect::<Vec<_>>(),
        };

        // Broadcast reliable messages to clients
        for conn in targets {
            let conn_ = conn.clone();
            let message_ = data.clone();
            io_pool
                .spawn(async move {
                    let result = async move {
                        let mut sender = conn_.open_uni().await?;

                        sender.write_all(&message_).await?;
                        sender.finish().await?;

                        Ok::<(), anyhow::Error>(())
                    };

                    if let Err(e) = result.await {
                        error!("Error sending reliable message: {e:?}");
                    }
                })
                .detach();
        }
    }

    while let Ok(message) = server.outgoing_unreliable_receiver.try_recv() {
        let data = Bytes::from(message.data);

        let targets = match message.target {
            MessageTarget::All => server.clients.iter().collect::<Vec<_>>(),
            MessageTarget::Client(idx) => [&server.clients[idx]].into_iter().collect::<Vec<_>>(),
        };

        // Broadcast unreliable messages to clients
        for conn in targets {
            let conn_ = conn.clone();
            let message_ = data.clone();
            io_pool
                .spawn(async move {
                    let result = conn_.send_datagram(message_);

                    if let Err(e) = result {
                        error!("Error sending reliable message: {e:?}");
                    }
                })
                .detach();
        }
    }
}

fn receive_messages(server: Res<NetServer>) {
    let io_pool = IoTaskPool::get();

    for (client_idx, conn) in server.clients.iter().enumerate() {
        let reliable_sender = server.incomming_reliable_sender.clone();
        let unreliable_sender = server.incomming_unreliable_sender.clone();
        let conn = conn.clone();

        io_pool.spawn(async move {
            let result = async move {
                future::zip(
                    async {
                        while let Ok(recv) = conn.accept_uni().await {
                            let message = recv.read_to_end(1024 * 1024).await?;

                            reliable_sender
                                .send(Incomming {
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
                                .send(Incomming {
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

            if let Err(e) = result.await {
                error!("Error receiving net messages: {e:?}");
            }
        });
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
