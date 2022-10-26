use crate::{
    metadata::{MapMeta, PlayerMeta},
    prelude::*,
};

/// Network message sent by client to select a player
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MatchSetupFromClient {
    SelectPlayer(AssetHandle<PlayerMeta>),
    ConfirmSelection(bool),
    SelectMap(AssetHandle<MapMeta>),
    ReadyToStart,
}

/// Network message sent by server to notify clients of selected players
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MatchSetupFromServer {
    ClientMessage {
        player_idx: u8,
        message: MatchSetupFromClient,
    },
    SelectMap,
    WaitForMapSelect,
}
