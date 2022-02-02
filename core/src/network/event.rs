#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::network::{Lobby, Protocol};
use crate::Id;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum NetworkEvent {
    LobbyCreated {
        lobby_id: Id,
    },
    LobbyChanged {
        lobby: Lobby,
    },
    PlayerJoined {
        player_id: Id,
        username: String,
        port: u16,
        protocol: Protocol,
    },
    PlayerLeft {
        player_id: Id,
    },
    PlayerReconnecting {
        player_id: Id,
        protocol: Protocol,
    },
    GameStarted {
        lobby_id: Id,
    },
    GameEnded {
        lobby_id: Id,
    },
}
