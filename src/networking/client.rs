use bevy::tasks::IoTaskPool;
use flume::{Receiver, Sender};
use futures_lite::future;
use serde::de::DeserializeOwned;

use crate::prelude::*;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(CoreStage::Last, receive_messages);
    }
}

pub struct NetClient {
    conn: super::Connection,
    reliable_sender: Sender<Vec<u8>>,
    reliable_receiver: Receiver<Vec<u8>>,
    unreliable_sender: Sender<Vec<u8>>,
    unreliable_receiver: Receiver<Vec<u8>>,
}

impl NetClient {
    pub fn new(conn: super::Connection) -> Self {
        let (reliable_sender, reliable_receiver) = flume::unbounded();
        let (unreliable_sender, unreliable_receiver) = flume::unbounded();

        Self {
            conn,
            reliable_sender,
            reliable_receiver,
            unreliable_sender,
            unreliable_receiver,
        }
    }

    pub fn recv_reliable<D: DeserializeOwned>(&self) -> Option<D> {
        self.reliable_receiver
            .try_recv()
            .map(|message| postcard::from_bytes(&message).expect("Deserialize net message"))
            .ok()
    }

    pub fn recv_unreliable<D: DeserializeOwned>(&self) -> Option<D> {
        self.unreliable_receiver
            .try_recv()
            .map(|message| postcard::from_bytes(&message).expect("Deserialize net message"))
            .ok()
    }
}

fn receive_messages(client: Res<NetClient>) {
    // TODO: Consider a double-buffer approach that prevents new messages from comming in while a
    // frame is in progress, if that has any benefits.
    let io_pool = IoTaskPool::get();

    let reliable_sender = client.reliable_sender.clone();
    let unreliable_sender = client.unreliable_sender.clone();
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
