use std::net::{SocketAddr, UdpSocket};
use std::sync::mpsc;

use serde::{Deserialize, Serialize};

use macroquad::experimental::scene::{Node, NodeWith, RefMut};
use macroquad::prelude::*;

use crate::data::{deserialize_bin, serialize_bin};
use crate::network::client::ClientState;
use crate::network::{
    AccountId, Api, ClientMessage, ClientMessageBody, NetworkGameResult, NetworkGameState,
    NetworkPlayerState, UDP_CHUNK_SIZE,
};
use crate::player::{PlayerControllerKind, PlayerParams};
use crate::{GameInput, Result};

type Channel = crate::Channel<ServerMessage, ClientMessage>;

const MESSAGE_BUFFER_LENGTH: usize = 4;

const WAIT_TIMEOUT: f32 = 45.0;

struct ServerMessage {
    dest: SocketAddr,
    body: ServerMessageBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessageBody {
    Waiting,
    ConnectAck,
    State(NetworkGameState),
    Terminate,
    Result(NetworkGameResult),
}

#[allow(dead_code)]
impl ServerMessage {
    pub fn waiting(dest: SocketAddr) -> Self {
        ServerMessage {
            dest,
            body: ServerMessageBody::Waiting,
        }
    }

    pub fn connect_ack(dest: SocketAddr) -> Self {
        ServerMessage {
            dest,
            body: ServerMessageBody::ConnectAck,
        }
    }

    pub fn state(dest: SocketAddr, state: NetworkGameState) -> Self {
        ServerMessage {
            dest,
            body: ServerMessageBody::State(state),
        }
    }

    pub fn terminate(dest: SocketAddr) -> Self {
        ServerMessage {
            dest,
            body: ServerMessageBody::Terminate,
        }
    }

    pub fn result(dest: SocketAddr, result: NetworkGameResult) -> Self {
        ServerMessage {
            dest,
            body: ServerMessageBody::Result(result),
        }
    }
}

#[derive(Default)]
struct InputBuffer {
    data: [Option<GameInput>; MESSAGE_BUFFER_LENGTH],
}

impl InputBuffer {
    pub fn push(&mut self, input: GameInput) {
        for i in 0..MESSAGE_BUFFER_LENGTH - 1 {
            self.data[i + 1] = self.data[i].take();
        }

        self.data[0] = Some(input);
    }

    pub fn pop(&mut self) -> Option<GameInput> {
        let res = self.data[0].take();

        for i in 0..MESSAGE_BUFFER_LENGTH - 1 {
            self.data[i] = self.data[i + 1].take();
        }

        res
    }
}

struct ClientParams {
    index: u8,
    account_id: AccountId,
    address: Option<SocketAddr>,
    input_buffer: InputBuffer,
    state: ClientState,
}

impl ClientParams {
    pub fn new(index: u8, account_id: AccountId) -> Self {
        ClientParams {
            index,
            account_id,
            address: None,
            input_buffer: Default::default(),
            state: ClientState::None,
        }
    }

    pub fn try_from(params: &PlayerParams) -> Option<Self> {
        if let PlayerControllerKind::Network(account_id) = params.controller {
            Some(ClientParams::new(params.index, account_id))
        } else {
            None
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ServerState {
    Waiting,
    Ready,
    Finished,
    Terminating,
}

pub struct Server {
    clients: Vec<ClientParams>,
    channel: Channel,
    state: ServerState,
    wait_timeout_timer: f32,
}

impl Server {
    pub fn new(port: u16, players: &[PlayerParams]) -> Result<Self> {
        let ip = Api::get_instance().get_own_address()?;
        let listen_address = SocketAddr::new(ip, port);

        let clients = players.iter().filter_map(ClientParams::try_from).collect();

        let socket = UdpSocket::bind(listen_address)?;

        let (tx_1, rx_1) = mpsc::channel::<ServerMessage>();
        let (tx_2, rx_2) = mpsc::channel::<ClientMessage>();

        {
            let socket = socket.try_clone()?;
            std::thread::spawn(move || loop {
                let mut data = [0; UDP_CHUNK_SIZE];
                match socket.recv_from(&mut data) {
                    Err(..) => {}
                    Ok((count, src)) => {
                        assert!(count < UDP_CHUNK_SIZE);
                        match deserialize_bin(&data[0..count]) {
                            Ok(body) => {
                                #[cfg(debug_assertions)]
                                println!("Server < {}: {:?}", src, &body);

                                tx_2.send(ClientMessage::new(src, body)).unwrap();
                            }
                            Err(err) => {
                                #[cfg(debug_assertions)]
                                println!("Server < {}: {}", src, err)
                            }
                        }
                    }
                }
            });
        }

        {
            let socket = socket.try_clone()?;
            std::thread::spawn(move || loop {
                if let Ok(message) = rx_1.recv() {
                    #[cfg(debug_assertions)]
                    println!("Server > {}: {:?}", message.dest, message.body);

                    let data = serialize_bin(&message.body).unwrap();
                    socket.send_to(&data, message.dest).unwrap();
                }
            });
        }

        let channel = Channel::new(tx_1, rx_2);

        let res = Server {
            clients,
            channel,
            state: ServerState::Waiting,
            wait_timeout_timer: 0.0,
        };

        Ok(res)
    }

    fn get_game_state(&self) -> NetworkGameState {
        let players = scene::find_nodes_by_type::<OldPlayer>()
            .map(|node| NetworkPlayerState {
                index: node.index,
                position: node.body.position,
                velocity: node.body.velocity,
                is_facing_right: node.body.is_facing_right,
                is_upside_down: node.body.is_upside_down,
                is_on_ground: node.body.is_on_ground,
                is_crouched: node.is_crouched,
            })
            .collect();

        NetworkGameState { players }
    }

    fn get_game_result(&self) -> Option<NetworkGameResult> {
        None
    }
}

impl Node for Server {
    fn update(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        if node.state == ServerState::Ready {
            let mut game = scene::find_node_by_type::<NetworkGame>().unwrap();
            for client in &mut node.clients {
                if client.state == ClientState::Ready {
                    if let Some(input) = client.input_buffer.pop() {
                        game.apply_player_input(client.index, input);
                    }
                }
            }
        }
    }

    fn fixed_update(mut node: RefMut<Self>)
    where
        Self: Sized,
    {
        if node.state == ServerState::Waiting {
            let are_clients_ready = node
                .clients
                .iter()
                .filter(|&client| client.state != ClientState::Ready)
                .count()
                == 0;

            if are_clients_ready {
                node.state = ServerState::Ready;
            } else {
                node.wait_timeout_timer += get_frame_time();
                if node.wait_timeout_timer >= WAIT_TIMEOUT {
                    node.clients
                        .retain(|client| client.state == ClientState::Ready);
                    node.state = ServerState::Ready;
                }
            }
        }

        while let Ok(message) = node.channel.rx.try_recv() {
            if node.state == ServerState::Waiting {
                if let ClientMessageBody::Connect(account_id) = message.body {
                    let res = node
                        .clients
                        .iter_mut()
                        .find(|client| client.account_id == account_id);

                    if let Some(mut client) = res {
                        client.address = Some(message.src);
                        client.state = ClientState::Ready;

                        let message = ServerMessage::connect_ack(message.src);
                        node.channel.tx.send(message).unwrap();
                    }
                }
            } else {
                let res = node.clients.iter_mut().find(|client| {
                    client.address.is_some() && client.address.unwrap() == message.src
                });

                if let Some(mut client) = res {
                    match message.body {
                        ClientMessageBody::Input(input) => {
                            client.input_buffer.push(input);
                        }
                        ClientMessageBody::Disconnect => {
                            client.state = ClientState::Disconnected;
                        }
                        _ => {}
                    }
                }
            }
        }

        node.clients
            .retain(|client| client.state != ClientState::Disconnected);

        if node.state == ServerState::Ready {
            if let Some(result) = node.get_game_result() {
                for client in &node.clients {
                    let address = client.address.unwrap();
                    let message = ServerMessage::result(address, result.clone());
                    node.channel.tx.send(message).unwrap();
                }
            } else {
                let state = node.get_game_state();
                for client in &node.clients {
                    let address = client.address.unwrap();
                    let message = ServerMessage::state(address, state.clone());
                    node.channel.tx.send(message).unwrap();
                }
            }
        }

        if node.state == ServerState::Ready {
            for NodeWith { node, capability } in scene::find_nodes_with::<NetworkReplicate>() {
                (capability.network_update)(node);
            }
        }
    }
}
