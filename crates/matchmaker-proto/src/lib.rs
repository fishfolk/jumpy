use serde::{Deserialize, Serialize};

//
// === Matchmaking Mode ===
//
// These are messages sent when first establishing a connecting to the matchmaker and waiting for a
// match.
//

/// Requests that may be made in matchmaking mode
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MatchmakerRequest {
    /// Request a match ID from the server
    RequestMatch(MatchInfo),
}

/// Information about a match that is being requested
#[derive(Serialize, Deserialize, Debug, Clone, Hash, Eq, PartialEq)]
pub struct MatchInfo {
    /// The number of clients to have in a match
    pub client_count: u8,
    /// This is an arbitrary set of bytes that must match exactly for clients to end up in the same
    /// match.
    ///
    /// This allows us to support matchmaking for different games/modes with the same matchmaking
    /// server.
    pub match_data: Vec<u8>,
}

/// Responses that may be returned in matchmaking mode
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MatchmakerResponse {
    Accepted,
    ClientCount(u8),
    Success,
}

//
// === Proxy mode ===
//
// These are messages sent after the match has been made and the clients are sending messages to
// each-other.
//

/// The format of a message sent by a client to the proxy, so the proxy can send it to another
/// client.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SendProxyMessage {
    /// The client that the message should go to.
    pub target_client: u8,
    /// The message data.
    pub message: Vec<u8>,
}

/// The format of a message forwarded by the proxy to a client.
pub struct RecvProxyMessage {
    /// The client that the message came from.
    pub from_client: u8,
    /// The message data.
    pub message: Vec<u8>,
}
