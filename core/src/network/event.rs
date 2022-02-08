#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::network::Lobby;
use crate::Id;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
