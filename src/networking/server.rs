use crate::networking::Connection;
use bevy::{app::AppExit, tasks::IoTaskPool};
use bytes::Bytes;
use flume::{Receiver, Sender};

use crate::prelude::*;

pub struct ServerPlugin;

pub struct NetServer {
    clients: Vec<Connection>,
    reliable_sender: Sender<Message>,
    reliable_receiver: Receiver<Message>,
    unreliable_sender: Sender<Message>,
    unreliable_receiver: Receiver<Message>,
}

struct Message {
    data: Vec<u8>,
    target: MessageTarget,
}

enum MessageTarget {
    All,
    Client(usize),
}

impl NetServer {
    pub fn new(clients: Vec<Connection>) -> Self {
        let (reliable_sender, reliable_receiver) = flume::unbounded();
        let (unreliable_sender, unreliable_receiver) = flume::unbounded();

        Self {
            clients,
            reliable_sender,
            reliable_receiver,
            unreliable_sender,
            unreliable_receiver,
        }
    }

    pub fn send_reliable<S: Serialize>(&self, message: S, client_idx: usize) {
        let message = postcard::to_allocvec(&message).expect("Serialize net message");
        self.reliable_sender
            .try_send(Message {
                data: message,
                target: MessageTarget::Client(client_idx),
            })
            .ok();
    }

    pub fn send_unreliable<S: Serialize>(&self, message: S, client_idx: usize) {
        let message = postcard::to_allocvec(&message).expect("Serialize net message");
        self.unreliable_sender
            .try_send(Message {
                data: message,
                target: MessageTarget::Client(client_idx),
            })
            .ok();
    }

    pub fn broadcast_reliable<S: Serialize>(&self, message: S) {
        let message = postcard::to_allocvec(&message).expect("Serialize net message");
        self.reliable_sender
            .try_send(Message {
                data: message,
                target: MessageTarget::All,
            })
            .ok();
    }

    pub fn broadcast_unreliable<S: Serialize>(&self, message: S) {
        let message = postcard::to_allocvec(&message).expect("Serialize net message");
        self.unreliable_sender
            .try_send(Message {
                data: message,
                target: MessageTarget::All,
            })
            .ok();
    }
}

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            exit_on_disconnect.run_if_resource_exists::<NetServer>(),
        )
        .add_system_to_stage(
            CoreStage::Last,
            send_messages.run_if_resource_exists::<NetServer>(),
        );
    }
}

fn send_messages(server: Res<NetServer>) {
    let io_pool = IoTaskPool::get();

    while let Ok(message) = server.reliable_receiver.try_recv() {
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

    while let Ok(message) = server.unreliable_receiver.try_recv() {
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
                    let result = async move {
                        conn_.send_datagram(message_)?;

                        Ok::<(), anyhow::Error>(())
                    };

                    if let Err(e) = result.await {
                        error!("Error sending reliable message: {e:?}");
                    }
                })
                .detach();
        }
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
