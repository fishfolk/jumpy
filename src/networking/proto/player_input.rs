use crate::{player::input::PlayerControl, prelude::*};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerInputFromClient {
    pub control: PlayerControl,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlayerInputFromServer {
    pub player_idx: u32,
    pub control: PlayerControl,
}
