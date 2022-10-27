use std::{any::TypeId, collections::VecDeque, net::SocketAddr, sync::Arc};

use async_channel::{Receiver, RecvError, Sender};
use bevy::{tasks::IoTaskPool, utils::HashMap};
use futures_lite::future;
use quinn::{ClientConfig, Connection, Endpoint, EndpointConfig};
use quinn_bevy::BevyIoTaskPoolExecutor;
use serde::de::DeserializeOwned;

use crate::{metadata::GameMeta, player::input::PlayerInputs, prelude::*};

use super::{proto::ClientMatchInfo, NET_MESSAGE_TYPES};

// pub mod player_input;
pub mod game;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(game::ClientGamePlugin)
            .add_system_to_stage(
                CoreStage::First,
                remove_closed_client.run_if_resource_exists::<NetClient>(),
            )
            .add_system_to_stage(
                CoreStage::First,
                recv_client_match_info
                    .run_if_resource_exists::<NetClient>()
                    .run_unless_resource_exists::<ClientMatchInfo>(),
            )
            .add_exit_system(
                GameState::InGame,
                close_connection_when_leaving_game.run_if_resource_exists::<NetClient>(),
            );
    }
}

fn recv_client_match_info(
    mut commands: Commands,
    mut client: ResMut<NetClient>,
    mut player_inputs: ResMut<PlayerInputs>,
    game: Res<GameMeta>,
) {
    if let Some(match_info) = client.recv_reliable::<ClientMatchInfo>() {
        info!("Got match info: {:?}", match_info);

        for (i, player) in player_inputs.players.iter_mut().enumerate() {
            player.active = i < match_info.player_count;
            player.selected_player = game
                .player_handles
                .get(0)
                .expect("No players in .game.yaml")
                .clone_weak();
        }

        commands.insert_resource(match_info);
    }
}

fn remove_closed_client(client: Res<NetClient>, mut commands: Commands) {
    if client.is_closed() {
        commands.remove_resource::<NetClient>();
        commands.remove_resource::<ClientMatchInfo>();
    }
}

fn close_connection_when_leaving_game(client: Res<NetClient>) {
    client.close();
}

mod certs {
    use std::sync::Arc;

    // Implementation of `ServerCertVerifier` that verifies everything as trustworthy.
    pub struct SkipServerVerification;

    impl SkipServerVerification {
        pub fn new() -> Arc<Self> {
            Arc::new(Self)
        }
    }

    impl rustls::client::ServerCertVerifier for SkipServerVerification {
        fn verify_server_cert(
            &self,
            _end_entity: &rustls::Certificate,
            _intermediates: &[rustls::Certificate],
            _server_name: &rustls::ServerName,
            _scts: &mut dyn Iterator<Item = &[u8]>,
            _ocsp_response: &[u8],
            _now: std::time::SystemTime,
        ) -> Result<rustls::client::ServerCertVerified, rustls::Error> {
            Ok(rustls::client::ServerCertVerified::assertion())
        }
    }
}

fn configure_quic_client() -> ClientConfig {
    let crypto = rustls::ClientConfig::builder()
        .with_safe_defaults()
        .with_custom_certificate_verifier(certs::SkipServerVerification::new())
        .with_no_client_auth();

    ClientConfig::new(Arc::new(crypto))
}

pub async fn open_connection(
    server_addr: impl Into<SocketAddr>,
) -> anyhow::Result<(Endpoint, Connection)> {
    let client_config = configure_quic_client();
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    // Bind this endpoint to a UDP socket on the given client address.
    let endpoint = Endpoint::new(
        EndpointConfig::default(),
        None,
        socket,
        BevyIoTaskPoolExecutor,
    )?
    .0;

    let conn = endpoint
        .connect_with(client_config, server_addr.into(), "server")?
        .await?;

    Ok((endpoint, conn))
}

pub struct NetClient {
    endpoint: Endpoint,
    conn: Connection,
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
    pub fn new(endpoint: Endpoint, conn: Connection) -> Self {
        let (outgoing_reliable_sender, outgoing_reliable_receiver) = async_channel::unbounded();
        let (outgoing_unreliable_sender, outgoing_unreliable_receiver) = async_channel::unbounded();
        let (incomming_reliable_sender, incomming_reliable_receiver) = async_channel::unbounded();
        let (incomming_unreliable_sender, incomming_unreliable_receiver) =
            async_channel::unbounded();

        let client = Self {
            endpoint,
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
        };

        spawn_message_recv_task(&client);
        spawn_message_send_task(&client);

        client
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

    pub fn send_reliable<S: 'static + Serialize>(&self, message: &S) {
        let type_id = TypeId::of::<S>();
        let type_idx = NET_MESSAGE_TYPES
            .iter()
            .position(|x| x == &type_id)
            .expect("Net message type not registered") as u32;
        let mut message = postcard::to_allocvec(message).expect("Serialize net message");
        message.extend_from_slice(&(type_idx as u32).to_le_bytes());

        self.outgoing_reliable_sender.try_send(message).ok();
    }

    pub fn send_unreliable<S: 'static + Serialize>(&self, message: &S) {
        let type_id = TypeId::of::<S>();
        let type_idx = NET_MESSAGE_TYPES
            .iter()
            .position(|x| x == &type_id)
            .expect("Net message not registered") as u32;
        let mut message = postcard::to_allocvec(message).expect("Serialize net message");
        message.extend_from_slice(&(type_idx as u32).to_le_bytes());

        self.outgoing_unreliable_sender.try_send(message).ok();
    }

    pub fn recv_reliable<T: 'static + DeserializeOwned>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if !NET_MESSAGE_TYPES.contains(&type_id) {
            panic!("Attempt to receive unregistered message type");
        }
        self.update_queue();
        self.incomming_reliable_queue
            .get_mut(&type_id)
            .and_then(|queue| queue.pop_front())
            .map(|message| postcard::from_bytes(&message).expect("Deserialize net message"))
    }

    pub fn recv_unreliable<T: 'static + DeserializeOwned>(&mut self) -> Option<T> {
        let type_id = TypeId::of::<T>();
        if !NET_MESSAGE_TYPES.contains(&type_id) {
            panic!("Attempt to receive unregistered message type");
        }
        self.update_queue();
        self.incomming_unreliable_queue
            .get_mut(&type_id)
            .and_then(|queue| queue.pop_front())
            .map(|message| postcard::from_bytes(&message).expect("Deserialize net message"))
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
    pub fn endpoint(&self) -> &Endpoint {
        &self.endpoint
    }

    pub fn close(&self) {
        self.conn.close(0u8.into(), b"NetClient::close()");
    }

    pub fn is_closed(&self) -> bool {
        self.conn.close_reason().is_some()
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
