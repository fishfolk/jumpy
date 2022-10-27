use crate::prelude::*;

pub mod match_setup;
pub mod game;
// pub mod player_input;

/// A resource indicating which player this game client represents, and how many players there are
/// in the match.j
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClientMatchInfo {
    pub player_idx: usize,
    pub player_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ping;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pong;
