use serde::{Deserialize, Serialize};

use crate::input::PlayerInput;

use super::PlayerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMessage {
    UpdatePlayerInput {
        player_id: PlayerId,
        input: PlayerInput,
    },
}
