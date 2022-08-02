use serde::{Deserialize, Serialize};

use super::PlayerId;
use crate::network::Lobby;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "snake_case")]
pub enum NetworkEvent {
    LobbyCreated {
        lobby_id: PlayerId,
    },
    LobbyChanged {
        lobby: Lobby,
    },
    PlayerMarkedReady {
        player_id: PlayerId,
    },
    PlayerMarkedNotReady {
        player_id: PlayerId,
    },
    PlayerJoined {
        player_id: PlayerId,
        username: String,
    },
    PlayerLeft {
        player_id: PlayerId,
    },
    PlayerReconnecting {
        player_id: PlayerId,
    },
    GameStarted {
        lobby_id: PlayerId,
    },
    GameEnded {
        lobby_id: PlayerId,
    },
}
