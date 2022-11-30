//! Game session handling.
//!
//! A "session" in this context means either a local or networked game session, which for network
//! games will be synced with peers.

use bevy::ecs::system::SystemParam;
use bevy_ggrs::{
    ggrs::{self, NonBlockingSocket, P2PSession, SessionBuilder, SyncTestSession},
    ResetGGRSSession, SessionType,
};
use jumpy_matchmaker_proto::TargetClient;

use crate::{
    config::ENGINE_CONFIG,
    networking::{client::NetClient, proto::ClientMatchInfo},
    player,
    prelude::*,
    GgrsConfig,
};

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<FrameIdx>()
            .extend_rollback_plugin(|plugin| plugin.register_rollback_type::<FrameIdx>())
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

#[derive(SystemParam)]
pub struct SessionManager<'w, 's> {
    commands: Commands<'w, 's>,
    client_match_info: Option<Res<'w, ClientMatchInfo>>,
    client: Option<Res<'w, NetClient>>,
}

impl NonBlockingSocket<usize> for NetClient {
    fn send_to(&mut self, msg: &ggrs::Message, addr: &usize) {
        self.send_unreliable(msg.clone(), TargetClient::One(*addr as u8));
    }

    fn receive_all_messages(&mut self) -> Vec<(usize, ggrs::Message)> {
        let mut messages = Vec::new();
        while let Some(message) = self.recv_unreliable() {
            match message.kind {
                crate::networking::proto::UnreliableGameMessageKind::Ggrs(msg) => {
                    messages.push((message.from_player_idx, msg));
                }
            }
        }
        messages
    }
}

impl<'w, 's> SessionManager<'w, 's> {
    pub fn drop_session(&mut self) {
        self.commands.insert_resource(ResetGGRSSession);
        self.commands.remove_resource::<SessionType>();
        self.commands.remove_resource::<P2PSession<GgrsConfig>>();
        self.commands
            .remove_resource::<SyncTestSession<GgrsConfig>>();
    }

    /// Setup the game session
    pub fn start_session(&mut self) {
        const INPUT_DELAY: usize = 1;

        if let Some((info, client)) = self.client_match_info.as_ref().zip(self.client.as_ref()) {
            let client = (*client).clone();

            let mut builder = SessionBuilder::<GgrsConfig>::new();
            builder = builder
                .with_input_delay(INPUT_DELAY)
                .with_num_players(info.player_count);

            for i in 0..info.player_count {
                builder = builder
                    .add_player(
                        if i == info.player_idx {
                            ggrs::PlayerType::Local
                        } else {
                            ggrs::PlayerType::Remote(i)
                        },
                        i,
                    )
                    .expect("Invalid player handle");
            }

            let session = builder.start_p2p_session(client).unwrap();
            self.commands.insert_resource(session);
            self.commands.insert_resource(SessionType::P2PSession);
            info!("Started P2P session");
        } else {
            let mut builder = SessionBuilder::<GgrsConfig>::new();

            builder = builder
                .with_input_delay(INPUT_DELAY)
                .with_num_players(player::MAX_PLAYERS)
                .with_check_distance(ENGINE_CONFIG.sync_test_check_distance);

            for i in 0..player::MAX_PLAYERS {
                builder = builder.add_player(ggrs::PlayerType::Local, i).unwrap();
            }

            let session = builder.start_synctest_session().unwrap();
            self.commands.insert_resource(session);
            self.commands.insert_resource(SessionType::SyncTestSession);
            info!("Started Local session");
        }
    }
}
