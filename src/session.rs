//! Game session handling.
//!
//! A "session" in this context means either a local or networked game session, which for network
//! games will be synced with peers.

use bevy_ggrs::{
    ggrs::{self, SessionBuilder},
    SessionType,
};

use crate::{networking::proto::ClientMatchInfo, player, prelude::*, GgrsConfig};

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_enter_system(GameState::InGame, setup_session);
    }
}

/// Setup the game session
fn setup_session(mut commands: Commands, client_match_info: Option<Res<ClientMatchInfo>>) {
    if let Some(_info) = client_match_info {
        todo!("Network session");
    } else {
        let mut builder = SessionBuilder::<GgrsConfig>::new();

        builder = builder
            .with_num_players(player::MAX_PLAYERS)
            .with_check_distance(0);

        for i in 0..player::MAX_PLAYERS {
            builder = builder.add_player(ggrs::PlayerType::Local, i).unwrap();
        }

        let session = builder.start_synctest_session().unwrap();
        commands.insert_resource(session);
        commands.insert_resource(SessionType::SyncTestSession);
    }
}
