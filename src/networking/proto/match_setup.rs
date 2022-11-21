use crate::{
    metadata::{MapMeta, PlayerMeta},
    prelude::*,
};

/// Network message sent by client to select a player
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum MatchSetupMessage {
    SelectPlayer(AssetHandle<PlayerMeta>),
    ConfirmSelection(bool),
    SelectMap(AssetHandle<MapMeta>),
}
