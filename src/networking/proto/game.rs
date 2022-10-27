use crate::{animation::AnimationBankSprite, prelude::*};

#[derive(Serialize, Deserialize)]
pub enum GameEventFromServer {
    PlayerEvent {
        player_idx: u8,
        event: GamePlayerEvent,
    },
}

#[derive(Serialize, Deserialize)]
pub enum GamePlayerEvent {
    UpdateState(PlayerState),
    SpawnPlayer(Vec3),
    KillPlayer,
}

#[derive(Serialize, Deserialize)]
pub struct PlayerState {
    pub pos: Vec3,
    pub sprite: AnimationBankSprite,
}
