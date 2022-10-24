use bevy::tasks::IoTaskPool;
use flume::{Receiver, Sender};
use futures_lite::future;
use serde::de::DeserializeOwned;

use crate::prelude::*;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::Last,
            receive_messages.run_if_resource_exists::<NetClient>(),
        )
        .add_system_to_stage(
            CoreStage::Last,
            send_messages.run_if_resource_exists::<NetClient>(),
        );
    }
}

pub struct NetClient {
    conn: super::Connection,
    outgoing_reliable_sender: Sender<Vec<u8>>,
    outgoing_reliable_receiver: Receiver<Vec<u8>>,
    outgoing_unreliable_sender: Sender<Vec<u8>>,
    outgoing_unreliable_receiver: Receiver<Vec<u8>>,
    incomming_reliable_sender: Sender<Vec<u8>>,
    incomming_reliable_receiver: Receiver<Vec<u8>>,
    incomming_unreliable_sender: Sender<Vec<u8>>,
    incomming_unreliable_receiver: Receiver<Vec<u8>>,
}

impl NetClient {
    pub fn new(conn: super::Connection) -> Self {
        let (outgoing_reliable_sender, outgoing_reliable_receiver) = flume::unbounded();
        let (outgoing_unreliable_sender, outgoing_unreliable_receiver) = flume::unbounded();
        let (incomming_reliable_sender, incomming_reliable_receiver) = flume::unbounded();
        let (incomming_unreliable_sender, incomming_unreliable_receiver) = flume::unbounded();

        Self {
            conn,
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

    pub fn send_reliable<S: Serialize>(&self, message: S) {
        let message = postcard::to_allocvec(&message).expect("Serialize net message");
        self.outgoing_reliable_sender.try_send(message).ok();
    }

    pub fn send_unreliable<S: Serialize>(&self, message: S) {
        let message = postcard::to_allocvec(&message).expect("Serialize net message");
        self.outgoing_unreliable_sender.try_send(message).ok();
    }

    pub fn recv_reliable<D: DeserializeOwned>(&self) -> Option<D> {
        self.incomming_reliable_receiver
            .try_recv()
            .map(|message| postcard::from_bytes(&message).expect("Deserialize net message"))
            .ok()
    }

    pub fn recv_unreliable<D: DeserializeOwned>(&self) -> Option<D> {
        self.incomming_unreliable_receiver
            .try_recv()
            .map(|message| postcard::from_bytes(&message).expect("Deserialize net message"))
            .ok()
    }

    pub fn conn(&self) -> &super::Connection {
        &self.conn
    }
}

fn receive_messages(client: Res<NetClient>) {
    // TODO: Consider a double-buffer approach that prevents new messages from comming in while a
    // frame is in progress, if that has any benefits.
    let io_pool = IoTaskPool::get();

    let reliable_sender = client.incomming_reliable_sender.clone();
    let unreliable_sender = client.incomming_unreliable_sender.clone();
    let conn = client.conn.clone();

    io_pool.spawn(async move {
        let result = async move {
            future::zip(
                async {
                    while let Ok(recv) = conn.accept_uni().await {
                        let message = recv.read_to_end(1024 * 1024).await?;

                        reliable_sender.send(message).ok();
                    }

                    Ok::<(), anyhow::Error>(())
                },
                async {
                    while let Ok(message) = conn.read_datagram().await {
                        unreliable_sender.send(message.to_vec()).ok();
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

fn send_messages(client: Res<NetClient>) {
    let io_pool = IoTaskPool::get();

    while let Ok(message) = client.outgoing_reliable_receiver.try_recv() {
        let conn = client.conn.clone();
        io_pool
            .spawn(async move {
                let result = async move {
                    let mut sender = conn.open_uni().await?;

                    sender.write_all(&message).await?;
                    sender.finish().await?;

                    Ok::<(), anyhow::Error>(())
                };

                if let Err(e) = result.await {
                    error!("Error sending reliable message: {e:?}");
                }
            })
            .detach();
    }

    while let Ok(message) = client.outgoing_unreliable_receiver.try_recv() {
        let conn = client.conn.clone();
        io_pool
            .spawn(async move {
                let result = conn.send_datagram(message.into());

                if let Err(e) = result {
                    error!("Error sending reliable message: {e:?}");
                }
            })
            .detach();
    }
}
