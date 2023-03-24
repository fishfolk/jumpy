use std::net::Ipv4Addr;

use bevy::tasks::IoTaskPool;
use bytes::Bytes;
use futures_lite::{future, FutureExt};
use jumpy_core::input::PlayerControl;

use super::{
    proto::{DenseMoveDirection, DensePlayerControl},
    *,
};

pub static LAN_MATCHMAKER: Lazy<LanMatchmaker> = Lazy::new(|| {
    let (client, server) = bi_channel();

    IoTaskPool::get().spawn(lan_matchmaker(server)).detach();

    LanMatchmaker(client)
});

async fn lan_matchmaker(
    matchmaker_channel: BiChannelServer<LanMatchmakerRequest, LanMatchmakerResponse>,
) {
    #[derive(Serialize, Deserialize)]
    enum MatchmakerNetMsg {
        MatchReady { random_seed: u64 },
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
                        // Subtract one to count the host
                        // Tell all clients we're ready
                        for conn in &connections {
                            let mut uni = conn.open_uni().await.unwrap();
                            uni.write_all(
                                &postcard::to_vec::<_, 9>(&MatchmakerNetMsg::MatchReady {
                                    random_seed: 0, // TODO: random random seed.
                                })
                                .unwrap(),
                            )
                            .await
                            .unwrap();
                            uni.finish().await.unwrap();
                        }
                    } else if matchmaker_channel
                        .try_send(LanMatchmakerResponse::PlayerCount(current_players))
                        .is_err()
                    {
                        break;
                    }
                }
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
                let uni = conn.accept_uni().await.unwrap();
                let bytes = uni.read_to_end(20).await.unwrap();
                let message: MatchmakerNetMsg = postcard::from_bytes(&bytes).unwrap();

                match message {
                    MatchmakerNetMsg::MatchReady { random_seed } => {
                        println!("TODO: join match: {random_seed}");
                    }
                }
            }
        }
    }
}

// async fn lan_server(channel: BiChannelServer<LanServerRequest, LanServerResponse>) {}

#[derive(DerefMut, Deref)]
pub struct LanMatchmaker(BiChannelClient<LanMatchmakerRequest, LanMatchmakerResponse>);

pub enum LanMatchmakerRequest {
    StartServer { player_count: usize },
    JoinServer { ip: Ipv4Addr, port: u16 },
    StopServer,
    StopJoin,
}
pub enum LanMatchmakerResponse {
    ServerStarted,
    PlayerCount(usize),
}

// pub enum LanServerRequest {
//     Stop,
// }
// pub enum LanServerResponse {}

pub struct LanSessionRunner {
    pub core: CoreSession,
    pub session: P2PSession<GgrsConfig>,
    pub delta: f32,
    pub accumulator: f32,
}

pub struct LanSessionInfo {
    pub socket: LanSocket,
    pub player_is_local: [bool; MAX_PLAYERS],
    pub player_count: usize,
}

pub struct LanSocket {
    pub connections: [Option<quinn::Connection>; MAX_PLAYERS],
    pub message_channel: async_channel::Receiver<(usize, ggrs::Message)>,
}

impl LanSocket {
    pub fn new(connections: [Option<quinn::Connection>; MAX_PLAYERS]) -> Self {
        let (sender, receiver) = async_channel::unbounded();

        let pool = bevy::tasks::IoTaskPool::get();

        // Spawn tasks to receive network messages from each peer
        #[allow(clippy::needless_range_loop)]
        for i in 0..MAX_PLAYERS {
            if let Some(conn) = connections[i].clone() {
                let sender = sender.clone();
                pool.spawn(async move {
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

                                    if sender.send((i, message)).await.is_err() {
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
            connections,
            message_channel: receiver,
        }
    }
}

impl ggrs::NonBlockingSocket<usize> for LanSocket {
    fn send_to(&mut self, msg: &ggrs::Message, addr: &usize) {
        let conn = self.connections[*addr].as_ref().unwrap();

        let msg_bytes = postcard::to_vec::<_, 32>(msg).unwrap();
        conn.send_datagram(Bytes::copy_from_slice(&msg_bytes[..]))
            .ok();
    }

    fn receive_all_messages(&mut self) -> Vec<(usize, ggrs::Message)> {
        let mut messages = Vec::new();
        while let Ok(message) = self.message_channel.try_recv() {
            messages.push(message);
        }
        messages
    }
}

impl LanSessionRunner {
    pub fn new(core: CoreSession, info: LanSessionInfo) -> Self
    where
        Self: Sized,
    {
        let mut builder = ggrs::SessionBuilder::new()
            .with_num_players(info.player_count)
            .with_input_delay(4)
            .with_fps(jumpy_core::FPS as usize)
            .unwrap();

        for i in 0..info.player_count {
            if info.player_is_local[i] {
                builder = builder.add_player(ggrs::PlayerType::Local, i).unwrap();
            } else {
                builder = builder.add_player(ggrs::PlayerType::Remote(i), i).unwrap();
            }
        }

        let session = builder.start_p2p_session(info.socket).unwrap();

        Self {
            core,
            session,
            accumulator: default(),
            delta: default(),
        }
    }
}

impl crate::session::SessionRunner for LanSessionRunner {
    fn core_session(&mut self) -> &mut CoreSession {
        &mut self.core
    }

    fn restart(&mut self) {
        self.core.restart()
    }

    fn set_player_input(&mut self, player_idx: usize, control: PlayerControl) {
        let mut dense_control = DensePlayerControl::default();
        dense_control.set_jump_pressed(control.jump_just_pressed);
        dense_control.set_grab_pressed(control.grab_pressed);
        dense_control.set_slide_pressed(control.slide_pressed);
        dense_control.set_shoot_pressed(control.shoot_pressed);
        dense_control.set_move_direction(DenseMoveDirection(control.move_direction));
        self.session
            .add_local_input(player_idx, dense_control)
            .unwrap();
    }

    fn advance(&mut self, bevy_world: &mut World) {
        const STEP: f32 = 1.0 / jumpy_core::FPS;
        let delta = self.delta;

        self.accumulator += delta;

        loop {
            if self.accumulator >= STEP {
                self.accumulator -= STEP;

                if let Ok(requests) = self.session.advance_frame() {
                    for request in requests {
                        match request {
                            ggrs::GGRSRequest::SaveGameState { cell, frame } => {
                                cell.save(frame, Some(self.core.world.clone()), None)
                            }
                            ggrs::GGRSRequest::LoadGameState { cell, .. } => {
                                let world = cell.load().unwrap_or_default();
                                self.core.world = world;
                            }
                            ggrs::GGRSRequest::AdvanceFrame {
                                inputs: network_inputs,
                            } => {
                                self.core.update_input(|inputs| {
                                    for (player_idx, (input, _status)) in
                                        network_inputs.into_iter().enumerate()
                                    {
                                        let control = &mut inputs.players[player_idx].control;

                                        let jump_pressed = input.jump_pressed();
                                        control.jump_just_pressed =
                                            jump_pressed && !control.jump_pressed;
                                        control.jump_pressed = jump_pressed;

                                        let grab_pressed = input.grab_pressed();
                                        control.grab_just_pressed =
                                            grab_pressed && !control.grab_pressed;
                                        control.grab_pressed = grab_pressed;

                                        let shoot_pressed = input.shoot_pressed();
                                        control.shoot_just_pressed =
                                            shoot_pressed && !control.shoot_pressed;
                                        control.shoot_pressed = shoot_pressed;

                                        let was_moving = control.move_direction.length_squared()
                                            > f32::MIN_POSITIVE;
                                        control.move_direction = input.move_direction().0;
                                        let is_moving = control.move_direction.length_squared()
                                            > f32::MIN_POSITIVE;
                                        control.just_moved = !was_moving && is_moving;
                                    }
                                });
                                self.core.advance(bevy_world);
                            }
                        }
                    }
                }
            } else {
                break;
            }
        }
    }

    fn run_criteria(&mut self, time: &Time) -> bevy::ecs::schedule::ShouldRun {
        self.delta = time.delta_seconds();
        bevy::ecs::schedule::ShouldRun::Yes
    }
}
