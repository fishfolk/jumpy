use serde::{Deserialize, Serialize};
use ulid::Ulid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MatchmakerRequest {
    /// Request a match ID from the server
    RequestMatch(MatchInfo),
}

/// Information about a match that is being requested
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct MatchInfo {
    pub player_count: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MatchmakerResponse {
    /// The ID of the match you've joined
    MatchId(Ulid),
}
