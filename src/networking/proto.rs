use crate::prelude::*;

pub mod match_setup;

/// A resource indicating which player this game client represents, and how many players there are
/// in the match.j
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct NetClientMatchInfo {
    pub player_idx: usize,
    pub player_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Ping;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pong;
