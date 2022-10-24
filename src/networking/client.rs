use std::{any::TypeId, collections::VecDeque};

use async_channel::{Receiver, RecvError, Sender};
use bevy::{tasks::IoTaskPool, utils::HashMap};
use futures_lite::future;
use serde::de::DeserializeOwned;

use crate::prelude::*;

use super::NET_MESSAGE_TYPES;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, _app: &mut App) {}
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
    incomming_reliable_queue: HashMap<TypeId, VecDeque<Vec<u8>>>,
    incomming_unreliable_queue: HashMap<TypeId, VecDeque<Vec<u8>>>,
}

impl NetClient {
    pub fn new(conn: super::Connection) -> Self {
        let (outgoing_reliable_sender, outgoing_reliable_receiver) = async_channel::unbounded();
        let (outgoing_unreliable_sender, outgoing_unreliable_receiver) = async_channel::unbounded();
        let (incomming_reliable_sender, incomming_reliable_receiver) = async_channel::unbounded();
        let (incomming_unreliable_sender, incomming_unreliable_receiver) =
            async_channel::unbounded();

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
            incomming_reliable_queue: default(),
            incomming_unreliable_queue: default(),
        }
    }

    fn update_queue(&mut self) {
        while let Ok(mut incomming) = self.incomming_reliable_receiver.try_recv() {
            let type_idx_bytes: [u8; 4] =
                incomming.split_off(incomming.len() - 4).try_into().unwrap();
            let type_idx = u32::from_le_bytes(type_idx_bytes);
            let type_id = NET_MESSAGE_TYPES[type_idx as usize];
            self.incomming_reliable_queue
                .entry(type_id)
                .or_default()
                .push_back(incomming);
        }
        while let Ok(mut incomming) = self.incomming_unreliable_receiver.try_recv() {
            let type_idx_bytes: [u8; 4] =
                incomming.split_off(incomming.len() - 4).try_into().unwrap();
            let type_idx = u32::from_le_bytes(type_idx_bytes);
            let type_id = NET_MESSAGE_TYPES[type_idx as usize];
            self.incomming_unreliable_queue
                .entry(type_id)
                .or_default()
                .push_back(incomming);
        }
    }

    pub fn send_reliable<S: 'static + Serialize>(&self, message: S) {
        let type_id = TypeId::of::<S>();
        let type_idx = NET_MESSAGE_TYPES
            .binary_search(&type_id)
            .expect("Net message not registered") as u32;
        let mut message = postcard::to_allocvec(&message).expect("Serialize net message");
        message.extend_from_slice(&(type_idx as u32).to_le_bytes());

        self.outgoing_reliable_sender.try_send(message).ok();
    }

    pub fn send_unreliable<S: 'static + Serialize>(&self, message: S) {
        let type_id = TypeId::of::<S>();
        let type_idx = NET_MESSAGE_TYPES
            .binary_search(&type_id)
            .expect("Net message not registered") as u32;
        let mut message = postcard::to_allocvec(&message).expect("Serialize net message");
        message.extend_from_slice(&(type_idx as u32).to_le_bytes());

        self.outgoing_unreliable_sender.try_send(message).ok();
    }

    pub fn recv_reliable<T: 'static + DeserializeOwned>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if !NET_MESSAGE_TYPES.contains(&type_id) {
            panic!("Attempt to receive unregistered message type");
        }
        self.update_queue();
        self.incomming_reliable_receiver
            .try_recv()
            .map(|message| postcard::from_bytes(&message).expect("Deserialize net message"))
            .ok()
    }

    pub fn recv_unreliable<T: 'static + DeserializeOwned>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if !NET_MESSAGE_TYPES.contains(&type_id) {
            panic!("Attempt to receive unregistered message type");
        }
        self.update_queue();
        self.incomming_unreliable_receiver
            .try_recv()
            .map(|message| postcard::from_bytes(&message).expect("Deserialize net message"))
            .ok()
    }

    pub fn conn(&self) -> &super::Connection {
        &self.conn
    }
}

pub fn spawn_connection_handlers(client: &NetClient) {
    spawn_message_recv_task(client);
    spawn_message_send_task(client);
}

fn spawn_message_recv_task(client: &NetClient) {
    let io_pool = IoTaskPool::get();

    let reliable_sender = client.incomming_reliable_sender.clone();
    let unreliable_sender = client.incomming_unreliable_sender.clone();
    let conn = client.conn.clone();

    io_pool
        .spawn(async move {
            'connection: loop {
                let receive_message_result = async {
                    future::zip(
                        async {
                            while let Ok(recv) = conn.accept_uni().await {
                                let message = recv.read_to_end(1024 * 1024).await?;

                                reliable_sender.try_send(message).ok();
                            }

                            Ok::<(), anyhow::Error>(())
                        },
                        async {
                            while let Ok(message) = conn.read_datagram().await {
                                unreliable_sender.try_send(message.to_vec()).ok();
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

fn spawn_message_send_task(client: &NetClient) {
    let io_pool = IoTaskPool::get();
    let conn = client.conn.clone();
    let outgoing_reliable_receiver = client.outgoing_reliable_receiver.clone();
    let outgoing_unreliable_receiver = client.outgoing_unreliable_receiver.clone();

    io_pool
        .spawn(async move {
            loop {
                let handle_reliable_message = async {
                    loop {
                        let message = outgoing_reliable_receiver.recv().await?;
                        let result = async {
                            let mut sender = conn.open_uni().await?;

                            sender.write_all(&message).await?;
                            sender.finish().await?;

                            Ok::<(), anyhow::Error>(())
                        };

                        if let Err(e) = result.await {
                            error!("Error sending reliable message: {e:?}");
                        }
                    }

                    // This is needed to annotate the return type of the block
                    #[allow(unreachable_code)]
                    Ok::<(), RecvError>(())
                };

                let handle_unreliable_message = async {
                    loop {
                        let message = outgoing_unreliable_receiver.recv().await?;
                        let result = conn.send_datagram(message.into());

                        if let Err(e) = result {
                            error!("Error sending reliable message: {e:?}");
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
