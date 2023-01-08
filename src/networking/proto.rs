//! Serializable data types for network messages used by the game.

use crate::prelude::*;

pub mod match_setup;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ReliableGameMessageKind {
    MatchSetup(match_setup::MatchSetupMessage),
}

impl From<match_setup::MatchSetupMessage> for ReliableGameMessageKind {
    fn from(x: match_setup::MatchSetupMessage) -> Self {
        Self::MatchSetup(x)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecvReliableGameMessage {
    pub from_player_idx: usize,
    pub kind: ReliableGameMessageKind,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum UnreliableGameMessageKind {
    Ggrs(bevy_ggrs::ggrs::Message),
}

impl From<bevy_ggrs::ggrs::Message> for UnreliableGameMessageKind {
    fn from(m: bevy_ggrs::ggrs::Message) -> Self {
        Self::Ggrs(m)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RecvUnreliableGameMessage {
    pub from_player_idx: usize,
    pub kind: UnreliableGameMessageKind,
}

/// A resource indicating which player this game client represents, and how many players there are
/// in the match.j
#[derive(Serialize, Deserialize, Debug, Clone, Resource)]
pub struct ClientMatchInfo {
    pub player_idx: usize,
    pub player_count: usize,
}
