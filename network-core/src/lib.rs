mod backend;
mod error;

use std::net::SocketAddr;

pub use backend::{Backend, MockBackend};
pub use error::{Error, Result};

pub const DEFAULT_PORT: u16 = 9000;

#[derive(Debug, Clone, PartialEq)]
pub struct Server {
    pub http: SocketAddr,
    pub udp: SocketAddr,
    pub tcp: SocketAddr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LobbyId(String);

impl LobbyId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for LobbyId {
    fn from(s: &str) -> Self {
        LobbyId(s.to_string())
    }
}

impl From<String> for LobbyId {
    fn from(s: String) -> Self {
        LobbyId(s)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum LobbyPrivacy {
    Public,
    Private,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LobbyState {
    NotStarted,
    ChannelReady,
    Starting,
    Running,
    Ending,
    Ended,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlayerId(String);

impl PlayerId {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<&str> for PlayerId {
    fn from(s: &str) -> Self {
        PlayerId(s.to_string())
    }
}

impl From<String> for PlayerId {
    fn from(s: String) -> Self {
        PlayerId(s)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlayerState {
    Joined,
    Ready,
    Playing,
    Left,
    Done,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub username: String,
    pub state: Vec<PlayerState>,
}

impl Player {
    pub fn new(id: &PlayerId, username: &str) -> Self {
        Player {
            id: id.clone(),
            username: username.to_string(),
            state: Vec::new(),
        }
    }
}
