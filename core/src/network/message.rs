use serde::{Deserialize, Serialize};

use crate::input::GameInput;
use crate::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMessage {
    UpdatePlayerController { player_id: Id, input: GameInput },
}
