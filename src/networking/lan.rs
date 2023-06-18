//! LAN matchmaking and socket implementation.
//!
//! ## Matchmaking
//!
//! The LAN matchmaker works by allowing the player to start a match and wait for people to join it,
//! or to join player's started match.
//!
//! Players hosting matches are found using the MDNS protocol. Currently the MDNS logic resides in
//! the [`ui::main_menu::network_game`] module, in the menu code. This probably isn't the best place
//! for it, and it should be moved into here to be a part of the [`lan_matchmaker`] task.
//!
//! Communication happens directly between LAN peers over the QUIC protocol.

use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

use bevy::tasks::IoTaskPool;
use bytes::Bytes;
use futures_lite::{future, FutureExt};

use super::*;

/// Channel used to do matchmaking over LAN.
///
/// Spawns a task to handle the actual matchmaking.
pub static LAN_MATCHMAKER: Lazy<LanMatchmaker> = Lazy::new(|| {
    let (client, server) = bi_channel();

    IoTaskPool::get().spawn(lan_matchmaker(server)).detach();

    LanMatchmaker(client)
});

/// Implementation of the lan matchmaker task.
///
/// This is a long-running tasks that listens for messages sent through the [`LAN_MATCHMAKER`]
/// channel.
async fn lan_matchmaker(
    matchmaker_channel: BiChannelServer<LanMatchmakerRequest, LanMatchmakerResponse>,
) {
    #[derive(Serialize, Deserialize)]
    enum MatchmakerNetMsg {
        MatchReady {
            /// The peers they have for the match, with the index in the array being the player index of the peer.
            peers: [Option<SocketAddrV4>; MAX_PLAYERS],
            /// The player index of the player getting the message.
            player_idx: usize,
            player_count: usize,
        },
    }

    while let Ok(request) = matchmaker_channel.recv().await {
        match request {
            // Start server
            LanMatchmakerRequest::StartServer { mut player_count } => {
                info!("Starting LAN server");
                matchmaker_channel
                    .send(LanMatchmakerResponse::ServerStarted)
                    .await
                    .unwrap();

                let mut connections = Vec::new();

                loop {
                    let next_request = async { either::Left(matchmaker_channel.recv().await) };
                    let next_conn = async { either::Right(NETWORK_ENDPOINT.accept().await) };

                    match next_request.or(next_conn).await {
                        // Handle more matchmaker requests
                        either::Either::Left(next_request) => {
                            let Ok(next_request) = next_request else { break; };

                            match next_request {
                                LanMatchmakerRequest::StartServer {
                                    player_count: new_player_count,
                                } => {
                                    connections.clear();
                                    player_count = new_player_count;
                                }
                                LanMatchmakerRequest::StopServer => {
                                    break;
                                }
                                LanMatchmakerRequest::StopJoin => {} // Not joining, so don't do anything
                                LanMatchmakerRequest::JoinServer { .. } => {
                                    error!("Cannot join server while hosting server");
                                }
                            }
                        }

                        // Handle new connections
                        either::Either::Right(Some(new_connection)) => {
                            let Some(conn) = new_connection.await.ok() else { continue };
                            connections.push(conn);
                            let current_players = connections.len() + 1;
                            info!(%current_players, "New player connection");
                        }
                        _ => (),
                    }

                    // Discard closed connections
                    connections.retain(|conn| {
                        if conn.close_reason().is_some() {
                            info!("Player closed connection");
                            false
                        } else {
                            true
                        }
                    });

                    let current_players = connections.len();
                    let target_players = player_count;
                    info!(%current_players, %target_players);

                    // If we're ready to start a match
                    if connections.len() == player_count - 1 {
                        info!("All players joined.");

                        // Tell all clients we're ready
                        for (i, conn) in connections.iter().enumerate() {
                            let mut peers = [None; MAX_PLAYERS];
                            connections
                                .iter()
                                .enumerate()
                                .filter(|x| x.0 != i)
                                .for_each(|(i, conn)| {
                                    if let SocketAddr::V4(addr) = conn.remote_address() {
                                        peers[i + 1] = Some(addr);
                                    } else {
                                        unreachable!("IPV6 not supported in LAN matchmaking");
                                    };
                                });

                            let mut uni = conn.open_uni().await.unwrap();
                            uni.write_all(
                                &postcard::to_vec::<_, 20>(&MatchmakerNetMsg::MatchReady {
                                    player_idx: i + 1,
                                    peers,
                                    player_count,
                                })
                                .unwrap(),
                            )
                            .await
                            .unwrap();
                            uni.finish().await.unwrap();
                        }

                        // Collect the list of client connections
                        let connections = std::array::from_fn(|i| {
                            if i == 0 {
                                None
                            } else {
                                connections.get(i - 1).cloned()
                            }
                        });

                        // Send the connections to the game so that it can start the network match.
                        matchmaker_channel
                            .try_send(LanMatchmakerResponse::GameStarting {
                                lan_socket: LanSocket::new(0, connections),
                                player_idx: 0,
                                player_count,
                            })
                            .ok();
                        info!(player_idx=0, %player_count, "Matchmaking finished");

                        // Break out of the server loop
                        break;

                    // If we don't have enough players yet, send the updated player count to the game.
                    } else if matchmaker_channel
                        .try_send(LanMatchmakerResponse::PlayerCount(current_players))
                        .is_err()
                    {
                        break;
                    }
                }

                // Once we are done with server matchmaking
            }
            // Server not running or joining so do nothing
            LanMatchmakerRequest::StopServer => (),
            LanMatchmakerRequest::StopJoin => (),

            // Join a hosted match
            LanMatchmakerRequest::JoinServer { ip, port } => {
                let conn = NETWORK_ENDPOINT
                    .connect((ip, port).into(), "jumpy-host")
                    .unwrap()
                    .await
                    .expect("Could not connect to server");

                // Wait for match to start
                let mut uni = conn.accept_uni().await.unwrap();
                let bytes = uni.read_to_end(20).await.unwrap();
                let message: MatchmakerNetMsg = postcard::from_bytes(&bytes).unwrap();

                match message {
                    MatchmakerNetMsg::MatchReady {
                        peers: peer_addrs,
                        player_idx,
                        player_count,
                    } => {
                        info!(%player_count, %player_idx, ?peer_addrs, "Matchmaking finished");
                        let mut peer_connections = std::array::from_fn(|_| None);

                        // Set the connection to the matchmaker for player 0
                        peer_connections[0] = Some(conn.clone());

                        // For every peer with a player index that is higher than ours, wait for
                        // them to connect to us.
                        let range = (player_idx + 1)..player_count;
                        info!(players=?range, "Waiting for {} peer connections", range.len());
                        for _ in range {
                            // Wait for connection
                            let conn = NETWORK_ENDPOINT
                                .accept()
                                .await
                                .unwrap()
                                .await
                                .expect("Could not accept incomming connection");

                            // Receive the player index
                            let idx = {
                                let mut buf = [0; 1];
                                let mut channel = conn.accept_uni().await.unwrap();
                                channel.read_exact(&mut buf).await.unwrap();

                                buf[0] as usize
                            };
                            assert!(idx < MAX_PLAYERS, "Invalid player index");

                            peer_connections[idx] = Some(conn);
                        }

                        // For every peer with a player index lower than ours, connect to them.
                        let range = 1..player_idx;
                        info!(players=?range, "Connecting to {} peers", range.len());
                        for i in range {
                            let addr = peer_addrs[i].unwrap();
                            let conn = NETWORK_ENDPOINT
                                .connect(addr.into(), "jumpy-peer")
                                .unwrap()
                                .await
                                .expect("Could not connect to peer");

                            // Send player index
                            let mut channel = conn.open_uni().await.unwrap();
                            channel.write(&[player_idx as u8]).await.unwrap();
                            channel.finish().await.unwrap();

                            peer_connections[i] = Some(conn);
                        }

                        let lan_socket = LanSocket::new(player_idx, peer_connections);
                        info!("Connections established.");

                        matchmaker_channel
                            .try_send(LanMatchmakerResponse::GameStarting {
                                lan_socket,
                                player_idx,
                                player_count,
                            })
                            .ok();
                    }
                }
            }
        }
    }
}

/// The type of the [`LAN_MATCHMAKER`] channel.
#[derive(DerefMut, Deref)]
pub struct LanMatchmaker(BiChannelClient<LanMatchmakerRequest, LanMatchmakerResponse>);

/// A request that may be sent to the [`LAN_MATCHMAKER`].
pub enum LanMatchmakerRequest {
    StartServer { player_count: usize },
    JoinServer { ip: Ipv4Addr, port: u16 },
    StopServer,
    StopJoin,
}

/// A response that may come from the [`LAN_MATCHMAKER`].
pub enum LanMatchmakerResponse {
    ServerStarted,
    PlayerCount(usize),
    GameStarting {
        lan_socket: LanSocket,
        player_idx: usize,
        player_count: usize,
    },
}

/// The LAN [`NetworkSocket`] implementation.
#[derive(Debug, Clone)]
pub struct LanSocket {
    pub connections: [Option<quinn::Connection>; MAX_PLAYERS],
    pub ggrs_receiver: async_channel::Receiver<(usize, ggrs::Message)>,
    pub reliable_receiver: async_channel::Receiver<(usize, Vec<u8>)>,
    pub player_idx: usize,
    pub player_count: usize,
}

impl LanSocket {
    pub fn new(player_idx: usize, connections: [Option<quinn::Connection>; MAX_PLAYERS]) -> Self {
        let (ggrs_sender, ggrs_receiver) = async_channel::unbounded();
        let (reliable_sender, reliable_receiver) = async_channel::unbounded();

        let pool = bevy::tasks::IoTaskPool::get();

        // Spawn tasks to receive network messages from each peer
        #[allow(clippy::needless_range_loop)]
        for i in 0..MAX_PLAYERS {
            if let Some(conn) = connections[i].clone() {
                let ggrs_sender = ggrs_sender.clone();

                // Unreliable message receiver
                let conn_ = conn.clone();
                pool.spawn(async move {
                    let conn = conn_;

                    #[cfg(feature = "debug-network-slowdown")]
                    use turborand::prelude::*;
                    #[cfg(feature = "debug-network-slowdown")]
                    let rng = AtomicRng::new();

                    loop {
                        let event =
                            future::or(async { either::Left(conn.closed().await) }, async {
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
                                    let message: ggrs::Message = postcard::from_bytes(&data)
                                        .expect("Could not deserialize net message");

                                    // Debugging code to introduce artificial latency
                                    #[cfg(feature = "debug-network-slowdown")]
                                    {
                                        use async_timer::Oneshot;
                                        async_timer::oneshot::Timer::new(
                                            std::time::Duration::from_millis(
                                                (rng.f32_normalized() * 100.0) as u64 + 1,
                                            ),
                                        )
                                        .await;
                                    }
                                    if ggrs_sender.send((i, message)).await.is_err() {
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

                // Reliable message receiver
                let reliable_sender = reliable_sender.clone();
                pool.spawn(async move {
                    #[cfg(feature = "debug-network-slowdown")]
                    use turborand::prelude::*;
                    #[cfg(feature = "debug-network-slowdown")]
                    let rng = AtomicRng::new();

                    loop {
                        let event =
                            future::or(async { either::Left(conn.closed().await) }, async {
                                either::Right(conn.accept_uni().await)
                            })
                            .await;

                        match event {
                            either::Either::Left(closed) => {
                                warn!("Connection error: {closed}");
                                break;
                            }
                            either::Either::Right(result) => match result {
                                Ok(mut stream) => {
                                    let data =
                                        stream.read_to_end(4096).await.expect("Network read error");

                                    // Debugging code to introduce artificial latency
                                    #[cfg(feature = "debug-network-slowdown")]
                                    {
                                        use async_timer::Oneshot;
                                        async_timer::oneshot::Timer::new(
                                            std::time::Duration::from_millis(
                                                (rng.f32_normalized() * 100.0) as u64 + 1,
                                            ),
                                        )
                                        .await;
                                    }
                                    if reliable_sender.send((i, data)).await.is_err() {
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
            }
        }

        Self {
            player_idx,
            player_count: connections.iter().flatten().count() + 1,
            connections,
            ggrs_receiver,
            reliable_receiver,
        }
    }
}

impl ggrs::NonBlockingSocket<usize> for LanSocket {
    fn send_to(&mut self, msg: &ggrs::Message, addr: &usize) {
        let conn = self.connections[*addr].as_ref().unwrap();

        // TODO: determine a reasonable size for this buffer.
        let msg_bytes = postcard::to_allocvec(msg).unwrap();
        conn.send_datagram(Bytes::copy_from_slice(&msg_bytes[..]))
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

impl NetworkSocket for LanSocket {
    fn send_reliable(&self, target: SocketTarget, message: &[u8]) {
        let task_pool = IoTaskPool::get();
        let message = Bytes::copy_from_slice(message);

        match target {
            SocketTarget::Player(i) => {
                let conn = self.connections[i].as_ref().unwrap().clone();

                task_pool
                    .spawn(async move {
                        let mut stream = conn.open_uni().await.unwrap();
                        stream.write_chunk(message).await.unwrap();
                        stream.finish().await.unwrap();
                    })
                    .detach();
            }
            SocketTarget::All => {
                for conn in &self.connections {
                    if let Some(conn) = conn.clone() {
                        let message = message.clone();
                        task_pool
                            .spawn(async move {
                                let mut stream = conn.open_uni().await.unwrap();
                                stream.write_chunk(message).await.unwrap();
                                stream.finish().await.unwrap();
                            })
                            .detach();
                    }
                }
            }
        }
    }

    fn recv_reliable(&self) -> Vec<(usize, Vec<u8>)> {
        let mut messages = Vec::new();
        while let Ok(message) = self.reliable_receiver.try_recv() {
            messages.push(message);
        }
        messages
    }

    fn ggrs_socket(&self) -> BoxedNonBlockingSocket {
        BoxedNonBlockingSocket(Box::new(self.clone()))
    }

    fn close(&self) {
        for conn in self.connections.iter().flatten() {
            conn.close(0u8.into(), &[]);
        }
    }

    fn player_idx(&self) -> usize {
        self.player_idx
    }

    fn player_count(&self) -> usize {
        self.player_count
    }

    fn player_is_local(&self) -> [bool; MAX_PLAYERS] {
        std::array::from_fn(|i| self.connections[i].is_none() && i < self.player_count)
    }
}
