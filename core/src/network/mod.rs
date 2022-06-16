mod api;
mod event;
mod message;
mod status;

pub use api::{Api, ApiBackend, ApiBackendConstructor};
pub use event::NetworkEvent;
pub use message::NetworkMessage;
pub use status::RequestStatus;

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

pub type LobbyId = String;
pub type PlayerId = String;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Server {
    pub http: SocketAddr,
    pub udp: SocketAddr,
    pub tcp: SocketAddr,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LobbyPrivacy {
    Public,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lobby {
    pub id: LobbyId,
    pub name: String,
    pub creator_player_id: PlayerId,
    pub admin_player_id: PlayerId,
    pub player_count: i32,
    pub capacity: i32,
    pub server: Option<Server>,
    pub privacy: LobbyPrivacy,
    pub state: LobbyState,
    pub players: Vec<Player>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum LobbyState {
    NotStarted,
    Ready,
    Starting,
    Running,
    Ending,
    Ended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub id: PlayerId,
    pub username: String,
    pub state: ClientState,
}

impl Player {
    pub fn new(id: &PlayerId, username: &str) -> Self {
        Player {
            id: id.clone(),
            username: username.to_string(),
            state: ClientState::Unknown,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum ClientState {
    Unknown,
    Joined,
    Ready,
    Playing,
    Left,
    Done,
}
