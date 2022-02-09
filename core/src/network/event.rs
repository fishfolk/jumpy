use serde::{Deserialize, Serialize};

use crate::network::Lobby;
use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkEvent {
    LobbyCreated {
        lobby_id: Id,
    },
    LobbyChanged {
        lobby: Lobby,
    },
    PlayerMarkedReady {
        player_id: Id,
    },
    PlayerMarkedNotReady {
        player_id: Id,
    },
    PlayerJoined {
        player_id: Id,
        username: String,
        port: u16,
    },
    PlayerLeft {
        player_id: Id,
    },
    PlayerReconnecting {
        player_id: Id,
    },
    GameStarted {
        lobby_id: Id,
    },
    GameEnded {
        lobby_id: Id,
    },
}
