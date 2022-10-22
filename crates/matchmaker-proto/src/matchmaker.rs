use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MatchmakerRequest {
    /// Request a match ID from the server
    RequestMatch {
        /// Specify the number of players you want to play with for this match
        players: u8,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MatchmakerResponse {
    /// The ID of the match you've joined
    MatchId(u64)
}
