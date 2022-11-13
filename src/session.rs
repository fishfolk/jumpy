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
        app.init_resource::<FrameIdx>()
            .add_enter_system(GameState::InGame, setup_session)
            .extend_rollback_plugin(|plugin| {
                plugin.register_rollback_type::<FrameIdx>()
            })
            .extend_rollback_schedule(|schedule| {
                schedule.add_system_to_stage(
                    RollbackStage::Last,
                    |mut frame_idx: ResMut<FrameIdx>| {
                        frame_idx.0 = frame_idx.0.wrapping_add(1);
                        trace!("End of simulation frame {}", frame_idx.0);
                    },
                );
            });
    }
}

/// The current game logic frame, as distict from a render frame, in the presence of rollback.
///
/// Primarily used for diagnostics.
#[derive(Reflect, Component, Default)]
#[reflect(Default)]
pub struct FrameIdx(pub u32);

/// Setup the game session
fn setup_session(mut commands: Commands, client_match_info: Option<Res<ClientMatchInfo>>) {
    if let Some(_info) = client_match_info {
        todo!("Network session");
    } else {
        let mut builder = SessionBuilder::<GgrsConfig>::new();

        builder = builder
            .with_num_players(player::MAX_PLAYERS)
            .with_check_distance(2);

        for i in 0..player::MAX_PLAYERS {
            builder = builder.add_player(ggrs::PlayerType::Local, i).unwrap();
        }

        let session = builder.start_synctest_session().unwrap();
        commands.insert_resource(session);
        commands.insert_resource(SessionType::SyncTestSession);
    }
}
