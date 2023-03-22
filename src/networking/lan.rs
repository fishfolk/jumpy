use bytes::Bytes;
use futures_lite::future;
use jumpy_core::input::PlayerControl;

use super::{
    proto::{DenseMoveDirection, DensePlayerControl},
    *,
};

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
