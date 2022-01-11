use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;

use serde::{Deserialize, Serialize};

use macroquad::experimental::scene::{Node, NodeWith, RefMut};
use macroquad::prelude::*;

use crate::data::{deserialize_bin, serialize_bin};
use crate::network::{
    AccountId, Api, NetworkGameState, ServerMessageBody, DEFAULT_CLIENT_PORT, DEFAULT_SERVER_PORT,
    UDP_CHUNK_SIZE,
};
use crate::{exit_to_main_menu, GameInput, Result};

type Channel = crate::Channel<ClientMessageBody, ServerMessageBody>;

pub struct ClientMessage {
    pub src: SocketAddr,
    pub body: ClientMessageBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessageBody {
    Connect(AccountId),
    Input(GameInput),
    Disconnect,
}

impl ClientMessage {
    pub fn new(src: SocketAddr, body: ClientMessageBody) -> Self {
        ClientMessage { src, body }
    }
}

const CONNECTION_RETRY_INTERVAL: f32 = 5.0;
const CONNECTION_TIMEOUT: f32 = 100.0;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ClientState {
    None,
    Ready,
    Disconnected,
}

pub struct Client {
    channel: Channel,
    is_ready: bool,
    retry_timer: f32,
    timeout_timer: f32,
    retry_cnt: u16,
    #[allow(dead_code)]
    state: ClientState,
}

impl Client {
    pub fn new(_host_id: AccountId) -> Result<Self> {
        let ip = Api::get_instance().get_own_address()?;
        let listen_address = SocketAddr::new(ip, DEFAULT_CLIENT_PORT);
        let server_address = SocketAddr::new(ip, DEFAULT_SERVER_PORT);

        let socket = UdpSocket::bind(listen_address)?;
        socket.connect(server_address)?;

        let (tx_1, rx_1) = mpsc::channel::<ClientMessageBody>();
        let (tx_2, rx_2) = mpsc::channel::<ServerMessageBody>();

        {
            let socket = socket.try_clone()?;
            std::thread::spawn(move || loop {
                let mut data = [0; UDP_CHUNK_SIZE];
                match socket.recv_from(&mut data) {
                    Err(..) => {}
                    Ok((count, src)) => {
                        if src == server_address {
                            assert!(count < UDP_CHUNK_SIZE);
                            match deserialize_bin(&data[0..count]) {
                                Ok(body) => {
                                    #[cfg(debug_assertions)]
                                    println!("Client < {}: {:?}", src, body);

                                    tx_2.send(body).unwrap();
                                }
                                Err(err) => {
                                    #[cfg(debug_assertions)]
                                    println!("Client < {}: {}", src, err);
                                }
                            }
                        } else {
                            #[cfg(debug_assertions)]
                            println!("Client < {}: Unknown source {:?}", src, &data);
                        }
                    }
                }
            });
        }

        {
            let socket = socket.try_clone()?;
            std::thread::spawn(move || loop {
                if let Ok(body) = rx_1.recv() {
                    #[cfg(debug_assertions)]
                    println!("Client > {}: {:?}", server_address, body);

                    let data = serialize_bin(&body).unwrap();
                    socket.send(&data).unwrap();
                }
            });
        }

        let channel = Channel::new(tx_1, rx_2);

        let res = Client {
            channel,
            is_ready: false,
            retry_timer: CONNECTION_RETRY_INTERVAL,
            timeout_timer: 0.0,
            retry_cnt: 0,
            state: ClientState::None,
        };

        Ok(res)
    }

    fn apply_state_update(&self, state: NetworkGameState) {
        /*
        for mut node in scene::find_nodes_by_type::<OldPlayer>() {
            let res = state.players.iter().find(|state| state.index == node.index);

            if let Some(player_state) = res {
                node.body.position = player_state.position;
                node.body.velocity = player_state.velocity;
                node.body.is_facing_right = player_state.is_facing_right;
                node.body.is_upside_down = player_state.is_upside_down;
                node.body.is_on_ground = player_state.is_on_ground;
                node.is_crouched = player_state.is_crouched;
            } else {
                node.delete();
            }
        }
         */
    }
}

impl Node for Client {
    fn fixed_update(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        if !node.is_ready {
            let dt = get_frame_time();
            node.timeout_timer += dt;
            if node.timeout_timer >= CONNECTION_TIMEOUT {
                #[cfg(debug_assertions)]
                println!("Client: Connection failed (timeout)");

                exit_to_main_menu();
            } else {
                node.retry_timer += dt;
                if node.retry_timer >= CONNECTION_RETRY_INTERVAL {
                    node.retry_timer = 0.0;
                    node.retry_cnt += 1;

                    #[cfg(debug_assertions)]
                    println!("Client: Connecting ({})", node.retry_cnt);

                    let account = Api::get_instance().get_own_account().unwrap();
                    let body = ClientMessageBody::Connect(account.id);
                    node.channel.tx.send(body).unwrap();
                }
            }
        }

        let mut should_terminate = false;
        let mut last_state = None;

        while let Ok(body) = node.channel.rx.try_recv() {
            if node.is_ready {
                match body {
                    ServerMessageBody::State(state) => {
                        last_state = Some(state);
                    }
                    ServerMessageBody::Result(result) => {
                        #[cfg(debug_assertions)]
                        println!("Client: Game result {:?}", result);

                        exit_to_main_menu();
                    }
                    ServerMessageBody::Terminate => {
                        should_terminate = true;
                        break;
                    }
                    _ => {}
                }
            } else {
                match body {
                    ServerMessageBody::ConnectAck => {
                        node.is_ready = true;
                    }
                    ServerMessageBody::Terminate => {
                        should_terminate = true;
                        break;
                    }
                    _ => {}
                }
            }
        }

        if should_terminate {
            #[cfg(debug_assertions)]
            println!("Client: Connection terminated");

            exit_to_main_menu();
        }

        if let Some(state) = last_state {
            node.apply_state_update(state);
        }

        if node.is_ready {
        }
    }
}
