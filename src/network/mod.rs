mod api;

use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

pub use api::{Account, AccountId, Api, Lobby, LobbyId};

use crate::json;

pub const UDP_CHUNK_SIZE: usize = 512;

pub const DEFAULT_CLIENT_PORT: u16 = 6002;
pub const DEFAULT_SERVER_PORT: u16 = 6001;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPlayerState {
    pub index: u8,
    #[serde(with = "json::vec2_def")]
    pub position: Vec2,
    #[serde(with = "json::vec2_def")]
    pub velocity: Vec2,
    pub is_facing_right: bool,
    pub is_upside_down: bool,
    pub is_on_ground: bool,
    pub is_crouched: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkGameState {
    pub players: Vec<NetworkPlayerState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkGameResult {}
