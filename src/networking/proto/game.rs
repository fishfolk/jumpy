use crate::{animation::AnimationBankSprite, networking::NetId, prelude::*};

use super::tick::Tick;

#[derive(Serialize, Deserialize)]
pub struct PlayerEventFromServer {
    pub player_idx: u8,
    pub kind: PlayerEvent,
}

#[derive(Serialize, Deserialize)]
pub enum GameEventFromServer {
    SpawnItem {
        net_id: NetId,
        script: String,
        pos: Vec3,
    },
}

#[derive(Serialize, Deserialize)]
pub struct PlayerStateFromServer {
    pub player_idx: u8,
    pub state: PlayerState,
}

#[derive(Serialize, Deserialize)]
pub enum PlayerEvent {
    SpawnPlayer(Vec3),
    KillPlayer {
        position: Vec3,
        velocity: Vec2,
    },
    DespawnPlayer,
    GrabItem(NetId),
    DropItem { position: Vec3, velocity: Vec2 },
    UseItem { position: Vec3, item: NetId },
}

#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    pub tick: Tick,
    pub pos: Vec3,
    pub sprite: AnimationBankSprite,
}
