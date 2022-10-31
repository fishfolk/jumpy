use crate::{animation::AnimationBankSprite, networking::NetId, prelude::*};

use super::tick::Tick;

#[derive(Serialize, Deserialize)]
pub struct PlayerEventFromServer {
    pub player_idx: u8,
    pub kind: GameEvent,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerStateFromServer {
    pub player_idx: u8,
    pub state: PlayerState,
}

#[derive(Serialize, Deserialize)]
pub enum GameEvent {
    SpawnPlayer(Vec3),
    KillPlayer,
    GrabItem(NetId),
    DropItem(Vec3),
}

#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    pub tick: Tick,
    pub pos: Vec3,
    pub sprite: AnimationBankSprite,
}
