use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MatchmakerRequest {
    /// Request a match ID from the server
    RequestMatch(MatchInfo),
}

/// Information about a match that is being requested
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct MatchInfo {
    pub client_count: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MatchmakerResponse {
    Accepted,
    ClientCount(u8),
    Success,
}
