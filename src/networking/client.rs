//! The Bevy network client implementation.
//!
//! This plugin provides bevy the [`NetClient`] resource which is used to send and receive messages
//! over the the network.
//!
//! Note that, because we use a P2P rollback networking model, Bevy only ever acts as a client and
//! is never a "server". Messages are sent to other peers by means of the matchmaking server.

use std::{net::SocketAddr, sync::Arc};

use async_channel::{Receiver, RecvError, Sender};
use bevy::tasks::IoTaskPool;
use bones_matchmaker_proto::{RecvProxyMessage, SendProxyMessage, TargetClient};
use futures_lite::future;
use quinn::{ClientConfig, Connection, Endpoint, EndpointConfig};
use quinn_runtime_bevy::BevyIoTaskPoolExecutor;

use crate::prelude::*;

use super::proto::{
    ClientMatchInfo, RecvReliableGameMessage, RecvUnreliableGameMessage, ReliableGameMessageKind,
    UnreliableGameMessageKind,
};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_to_stage(
            CoreStage::First,
            remove_closed_client.run_if_resource_exists::<NetClient>(),
        )
        .add_exit_system(
            GameState::InGame,
            close_connection_when_leaving_game.run_if_resource_exists::<NetClient>(),
        );
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
    )?;

    let conn = endpoint
        .connect_with(client_config, server_addr.into(), "server")?
        .await?;

    Ok((endpoint, conn))
}

/// Networking client. Can be cloned to get another handle to the same network client.
#[derive(Clone, Resource)]
pub struct NetClient(Arc<NetClientInner>);

struct NetClientInner {
    endpoint: Endpoint,
    conn: Connection,
    outgoing_reliable_sender: Sender<SendProxyMessage>,
    outgoing_reliable_receiver: Receiver<SendProxyMessage>,
    outgoing_unreliable_sender: Sender<SendProxyMessage>,
    outgoing_unreliable_receiver: Receiver<SendProxyMessage>,
    incomming_reliable_sender: Sender<RecvProxyMessage>,
    incomming_reliable_receiver: Receiver<RecvProxyMessage>,
    incomming_unreliable_sender: Sender<RecvProxyMessage>,
    incomming_unreliable_receiver: Receiver<RecvProxyMessage>,
}

impl NetClient {
    pub fn new(endpoint: Endpoint, conn: Connection) -> Self {
        let (outgoing_reliable_sender, outgoing_reliable_receiver) = async_channel::unbounded();
        let (outgoing_unreliable_sender, outgoing_unreliable_receiver) = async_channel::unbounded();
        let (incomming_reliable_sender, incomming_reliable_receiver) = async_channel::unbounded();
        let (incomming_unreliable_sender, incomming_unreliable_receiver) =
            async_channel::unbounded();

        let client = Self(Arc::new(NetClientInner {
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
        }));

        spawn_message_recv_task(&client);
        spawn_message_send_task(&client);

        client
    }

    pub fn send_reliable<M: Into<ReliableGameMessageKind>>(
        &self,
        message: M,
        target_client: TargetClient,
    ) {
        let message = message.into();
        let message = postcard::to_allocvec(&message).expect("Serialize net message");
        let proxy_message = SendProxyMessage {
            target_client,
            message,
        };
        self.0.outgoing_reliable_sender.try_send(proxy_message).ok();
    }

    pub fn send_unreliable<M: Into<UnreliableGameMessageKind>>(
        &self,
        message: M,
        target_client: TargetClient,
    ) {
        let message = message.into();
        let message = postcard::to_allocvec(&message).expect("Serialize net message");
        let proxy_message = SendProxyMessage {
            target_client,
            message,
        };
        self.0
            .outgoing_unreliable_sender
            .try_send(proxy_message)
            .ok();
    }

    pub fn recv_reliable(&self) -> Option<RecvReliableGameMessage> {
        self.0
            .incomming_reliable_receiver
            .try_recv()
            .map(|message| RecvReliableGameMessage {
                from_player_idx: message.from_client as usize,
                kind: postcard::from_bytes(&message.message)
                    .expect("TODO: Handle error: Net deserialize error"),
            })
            .ok()
    }

    pub fn recv_unreliable(&self) -> Option<RecvUnreliableGameMessage> {
        self.0
            .incomming_unreliable_receiver
            .try_recv()
            .map(|message| RecvUnreliableGameMessage {
                from_player_idx: message.from_client as usize,
                kind: postcard::from_bytes(&message.message)
                    .expect("TODO: Handle error: Net deserialize error"),
            })
            .ok()
    }

    pub fn conn(&self) -> &Connection {
        &self.0.conn
    }
    pub fn endpoint(&self) -> &Endpoint {
        &self.0.endpoint
    }

    pub fn close(&self) {
        self.0.conn.close(0u8.into(), b"NetClient::close()");
    }

    pub fn is_closed(&self) -> bool {
        self.0.conn.close_reason().is_some()
    }
}

fn spawn_message_recv_task(client: &NetClient) {
    let io_pool = IoTaskPool::get();

    let reliable_sender = client.0.incomming_reliable_sender.clone();
    let unreliable_sender = client.0.incomming_unreliable_sender.clone();
    let conn = client.0.conn.clone();

    io_pool
        .spawn(async move {
            'connection: loop {
                let receive_message_result = async {
                    future::zip(
                        async {
                            while let Ok(recv) = conn.accept_uni().await {
                                let message = recv.read_to_end(1024 * 1024).await?;
                                let message = postcard::from_bytes::<RecvProxyMessage>(&message)?;

                                reliable_sender.try_send(message).ok();
                            }

                            Ok::<(), anyhow::Error>(())
                        },
                        async {
                            while let Ok(message) = conn.read_datagram().await {
                                let message = postcard::from_bytes::<RecvProxyMessage>(&message)
                                    .expect("TODO: Handle error: deserialize net message.");

                                unreliable_sender.try_send(message).ok();
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
    let conn = client.0.conn.clone();
    let outgoing_reliable_receiver = client.0.outgoing_reliable_receiver.clone();
    let outgoing_unreliable_receiver = client.0.outgoing_unreliable_receiver.clone();

    io_pool
        .spawn(async move {
            loop {
                let handle_reliable_message = async {
                    loop {
                        let message = outgoing_reliable_receiver.recv().await?;
                        let message =
                            postcard::to_allocvec(&message).expect("Serialize net message");

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
                        let message =
                            postcard::to_allocvec(&message).expect("Serialize net message");

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
