use crate::{animation::AnimationBankSprite, prelude::*};

use super::tick::Tick;

#[derive(Serialize, Deserialize)]
pub struct PlayerEventFromServer {
    pub player_idx: u8,
    pub kind: PlayerEvent,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerStateFromServer {
    pub player_idx: u8,
    pub state: PlayerState,
}

#[derive(Serialize, Deserialize)]
pub enum PlayerEvent {
    SpawnPlayer(Vec3),
    KillPlayer,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    pub tick: Tick,
    pub pos: Vec3,
    pub sprite: AnimationBankSprite,
}
