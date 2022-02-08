mod api;
mod event;
mod status;

pub use api::{Api, ApiBackend, MockApiBackend};
pub use event::NetworkEvent;
pub use status::RequestStatus;

use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::Id;

pub const DEFAULT_PORT: u16 = 9000;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Server {
    pub http: SocketAddr,
    pub udp: SocketAddr,
    pub tcp: SocketAddr,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_json", serde(rename_all = "snake_case"))]
pub enum LobbyPrivacy {
    Public,
    Private,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Lobby {
    pub id: Id,
    pub name: String,
    pub creator_player_id: Id,
    pub admin_player_id: Id,
    pub player_count: i32,
    pub capacity: i32,
    pub server: Option<Server>,
    pub privacy: LobbyPrivacy,
    pub state: LobbyState,
    pub players: Vec<Player>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LobbyState {
    NotStarted,
    LobbyReady,
    Starting,
    Running,
    Ending,
    Ended,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Player {
    pub id: Id,
    pub username: String,
    pub state: ClientState,
}

impl Player {
    pub fn new(id: &Id, username: &str) -> Self {
        Player {
            id: id.clone(),
            username: username.to_string(),
            state: ClientState::None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ClientState {
    None,
    Joined,
    Ready,
    Playing,
    Left,
    Done,
}
