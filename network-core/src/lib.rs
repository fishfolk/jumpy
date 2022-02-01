mod api;
mod error;
mod status;

use std::net::SocketAddr;

pub use async_trait::async_trait;

pub use api::{Api, ApiBackend, MockApiBackend};
pub use error::{Error, ErrorKind, Result};
pub use status::RequestStatus;

#[cfg(feature = "serde")]
pub use serde;

#[cfg(feature = "serde_json")]
pub use serde_json;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub const DEFAULT_PORT: u16 = 9000;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Id(String);

impl Id {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Id(s.to_string())
    }
}

impl From<String> for Id {
    fn from(s: String) -> Self {
        Id(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Server {
    pub http: SocketAddr,
    pub udp: SocketAddr,
    pub tcp: SocketAddr,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
#[cfg_attr(feature = "serde_json", serde(rename_all = "snake_case"))]
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
    pub state: Vec<PlayerState>,
}

impl Player {
    pub fn new(id: &Id, username: &str) -> Self {
        Player {
            id: id.clone(),
            username: username.to_string(),
            state: Vec::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_json", serde(rename_all = "snake_case"))]
pub enum PlayerState {
    Joined,
    Ready,
    Playing,
    Left,
    Done,
}
