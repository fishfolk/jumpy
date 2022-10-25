use crate::{metadata::PlayerMeta, prelude::*};

pub struct PlayerSelectPlugin;

impl Plugin for PlayerSelectPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(handle_player_select.run_in_state(GameState::ServerPlayerSelect));
    }
}

/// Network message sent by client to select a player
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PlayerSelectFromClient {
    SelectPlayer(AssetHandle<PlayerMeta>),
    ConfirmSelection(bool),
}

/// Network message sent by server to notify clients of selected players
#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum PlayerSelectFromServer {
    PlayerSelection {
        player_idx: u8,
        message: PlayerSelectFromClient,
    },
    StartGame,
}

fn handle_player_select() {}
