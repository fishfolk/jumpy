use std::net::{SocketAddr, ToSocketAddrs};

use bevy::tasks::IoTaskPool;
use bones_matchmaker_proto::{MatchInfo, MatchmakerRequest, MatchmakerResponse};
use bytes::Bytes;
use futures_lite::future;
use quinn::Connection;

use crate::prelude::*;

use super::{NetworkSocket, NETWORK_ENDPOINT};

pub static ONLINE_MATCHMAKER: Lazy<OnlineMatchmaker> = Lazy::new(|| {
    let (client, server) = bi_channel();

    IoTaskPool::get().spawn(online_matchmaker(server)).detach();

    OnlineMatchmaker(client)
});

#[derive(DerefMut, Deref)]
pub struct OnlineMatchmaker(BiChannelClient<OnlineMatchmakerRequest, OnlineMatchmakerResponse>);

#[derive(Debug)]
pub enum OnlineMatchmakerRequest {
    SearchForGame { addr: String, player_count: usize },
    StopSearch,
}

#[derive(Debug)]
pub enum OnlineMatchmakerResponse {
    Searching,
    PlayerCount(usize),
    GameStarting {
        online_socket: OnlineSocket,
        player_idx: usize,
        player_count: usize,
    },
}

async fn online_matchmaker(
    matchmaker_channel: BiChannelServer<OnlineMatchmakerRequest, OnlineMatchmakerResponse>,
) {
    while let Ok(message) = matchmaker_channel.recv().await {
        match message {
            OnlineMatchmakerRequest::SearchForGame { addr, player_count } => {
                info!("Connecting to online matchmaker");
                let addr = resolve_addr_blocking(&addr).unwrap();
                let conn = NETWORK_ENDPOINT
                    .connect(addr, "matchmaker")
                    .unwrap()
                    .await
                    .unwrap();
                info!("Connected to online matchmaker");

                matchmaker_channel
                    .try_send(OnlineMatchmakerResponse::Searching)
                    .unwrap();

                // Send a match request to the server
                let (mut send, recv) = conn.open_bi().await.unwrap();

                let message = MatchmakerRequest::RequestMatch(MatchInfo {
                    client_count: player_count.try_into().unwrap(),
                    match_data: b"jumpy_default_game".to_vec(),
                });
                info!(request=?message, "Sending match request");
                let message = postcard::to_allocvec(&message).unwrap();
                send.write_all(&message).await.unwrap();
                send.finish().await.unwrap();

                let response = recv.read_to_end(256).await.unwrap();
                let message: MatchmakerResponse = postcard::from_bytes(&response).unwrap();

                if let MatchmakerResponse::Accepted = message {
                    info!("Waiting for match...");
                } else {
                    panic!("Invalid response from matchmaker");
                }

                loop {
                    let recv_ui_message = matchmaker_channel.recv();
                    let recv_online_matchmaker = conn.accept_uni();

                    let next_message = futures_lite::future::or(
                        async move { either::Left(recv_ui_message.await) },
                        async move { either::Right(recv_online_matchmaker.await) },
                    )
                    .await;

                    match next_message {
                        // UI message
                        either::Either::Left(message) => {
                            let message = message.unwrap();

                            match message {
                                OnlineMatchmakerRequest::SearchForGame { .. } => {
                                    panic!("Unexpected message from UI");
                                }
                                OnlineMatchmakerRequest::StopSearch => {
                                    info!("Canceling online search");
                                    break;
                                }
                            }
                        }

                        // Matchmaker message
                        either::Either::Right(recv) => {
                            let recv = recv.unwrap();
                            let message = recv.read_to_end(256).await.unwrap();
                            let message: MatchmakerResponse =
                                postcard::from_bytes(&message).unwrap();

                            match message {
                                MatchmakerResponse::ClientCount(count) => {
                                    info!("Online match player count: {count}");
                                    matchmaker_channel
                                        .try_send(OnlineMatchmakerResponse::PlayerCount(count as _))
                                        .unwrap();
                                }
                                MatchmakerResponse::Success {
                                    random_seed,
                                    player_idx,
                                    client_count,
                                } => {
                                    info!(%random_seed, %player_idx, player_count=%client_count, "Online match complete");
                                    let online_socket = OnlineSocket::new(
                                        player_idx as usize,
                                        client_count as usize,
                                        conn,
                                    );

                                    matchmaker_channel
                                        .try_send(OnlineMatchmakerResponse::GameStarting {
                                            online_socket,
                                            player_idx: player_idx as _,
                                            player_count: client_count as _,
                                        })
                                        .unwrap();
                                    break;
                                }
                                _ => panic!("Unexpected message from matchmaker"),
                            }
                        }
                    }
                }
            }
            OnlineMatchmakerRequest::StopSearch => (), // Not searching, don't do anything
        }
    }
}

/// Resolve a server address.
///
/// Note: This may block the thread
fn resolve_addr_blocking(addr: &str) -> anyhow::Result<SocketAddr> {
    let formatting_err =
        || anyhow::format_err!("Matchmaking server must be in the format `host:port`");

    let mut iter = addr.split(':');
    let host = iter.next().ok_or_else(formatting_err)?;
    let port = iter.next().ok_or_else(formatting_err)?;
    let port: u16 = port.parse().context("Couldn't parse port number")?;
    if iter.next().is_some() {
        return Err(formatting_err());
    }

    let addr = (host, port)
        .to_socket_addrs()
        .context("Couldn't resolve matchmaker address")?
        .find(|x| x.is_ipv4()) // For now, only support IpV4. I don't think IpV6 works right.
        .ok_or_else(|| anyhow::format_err!("Couldn't resolve matchmaker address"))?;

    Ok(addr)
}

#[derive(Debug, Clone)]
pub struct OnlineSocket {
    pub conn: Connection,
    pub ggrs_receiver: async_channel::Receiver<(usize, ggrs::Message)>,
    pub reliable_receiver: async_channel::Receiver<(usize, Vec<u8>)>,
    pub player_idx: usize,
    pub player_count: usize,
}

impl OnlineSocket {
    pub fn new(player_idx: usize, player_count: usize, conn: Connection) -> Self {
        let (ggrs_sender, ggrs_receiver) = async_channel::unbounded();
        let (reliable_sender, reliable_receiver) = async_channel::unbounded();

        let task_pool = IoTaskPool::get();

        let conn_ = conn.clone();
        task_pool
            .spawn(async move {
                let conn = conn_;
                loop {
                    let event = future::or(async { either::Left(conn.closed().await) }, async {
                        either::Right(conn.read_datagram().await)
                    })
                    .await;

                    match event {
                        either::Either::Left(closed) => {
                            warn!("Connection error: {closed}");
                            break;
                        }
                        either::Either::Right(datagram_result) => match datagram_result {
                            Ok(data) => {
                                let message: bones_matchmaker_proto::RecvProxyMessage =
                                    postcard::from_bytes(&data)
                                        .expect("Could not deserialize net message");
                                let player = message.from_client;
                                let message = postcard::from_bytes(&message.message).unwrap();

                                if ggrs_sender.send((player as _, message)).await.is_err() {
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Connection error: {e}");
                            }
                        },
                    }
                }
            })
            .detach();

        let conn_ = conn.clone();
        task_pool
            .spawn(async move {
                let conn = conn_;
                loop {
                    let event = future::or(async { either::Left(conn.closed().await) }, async {
                        either::Right(conn.accept_uni().await)
                    })
                    .await;

                    match event {
                        either::Either::Left(closed) => {
                            warn!("Connection error: {closed}");
                            break;
                        }
                        either::Either::Right(result) => match result {
                            Ok(stream) => {
                                let data =
                                    stream.read_to_end(4096).await.expect("Network read error");
                                let message: bones_matchmaker_proto::RecvProxyMessage =
                                    postcard::from_bytes(&data).unwrap();

                                if reliable_sender
                                    .send((message.from_client as usize, message.message))
                                    .await
                                    .is_err()
                                {
                                    break;
                                }
                            }
                            Err(e) => {
                                warn!("Connection error: {e}");
                            }
                        },
                    }
                }
            })
            .detach();

        Self {
            conn,
            ggrs_receiver,
            reliable_receiver,
            player_idx,
            player_count,
        }
    }
}

impl NetworkSocket for OnlineSocket {
    fn ggrs_socket(&self) -> networking::BoxedNonBlockingSocket {
        networking::BoxedNonBlockingSocket(Box::new(self.clone()))
    }

    fn send_reliable(&self, target: networking::SocketTarget, message: &[u8]) {
        let task_pool = IoTaskPool::get();
        let target_client = match target {
            networking::SocketTarget::Player(player) => {
                bones_matchmaker_proto::TargetClient::One(player as _)
            }
            networking::SocketTarget::All => bones_matchmaker_proto::TargetClient::All,
        };
        let message = bones_matchmaker_proto::SendProxyMessage {
            target_client,
            message: message.into(),
        };

        let conn = self.conn.clone();
        task_pool
            .spawn(async move {
                let mut send = conn.open_uni().await.unwrap();

                send.write_all(&postcard::to_allocvec(&message).unwrap())
                    .await
                    .unwrap();
                send.finish().await.unwrap();
            })
            .detach();
    }

    fn recv_reliable(&self) -> Vec<(usize, Vec<u8>)> {
        let mut messages = Vec::new();
        while let Ok(message) = self.reliable_receiver.try_recv() {
            messages.push(message);
        }
        messages
    }

    fn close(&self) {
        self.conn.close(0u8.into(), &[]);
    }

    fn player_idx(&self) -> usize {
        self.player_idx
    }

    fn player_is_local(&self) -> [bool; MAX_PLAYERS] {
        std::array::from_fn(|i| i == self.player_idx)
    }

    fn player_count(&self) -> usize {
        self.player_count
    }
}

impl ggrs::NonBlockingSocket<usize> for OnlineSocket {
    fn send_to(&mut self, msg: &ggrs::Message, addr: &usize) {
        let message = bones_matchmaker_proto::SendProxyMessage {
            target_client: bones_matchmaker_proto::TargetClient::One(*addr as u8),
            message: postcard::to_allocvec(msg).unwrap(),
        };
        let msg_bytes = postcard::to_allocvec(&message).unwrap();
        self.conn
            .send_datagram(Bytes::copy_from_slice(&msg_bytes[..]))
            .ok();
    }

    fn receive_all_messages(&mut self) -> Vec<(usize, ggrs::Message)> {
        let mut messages = Vec::new();
        while let Ok(message) = self.ggrs_receiver.try_recv() {
            messages.push(message);
        }
        messages
    }
}
