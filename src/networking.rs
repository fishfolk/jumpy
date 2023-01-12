#![doc = include_str!("./networking.md")]

use ggrs::P2PSession;

use crate::{prelude::*, session::SessionManager, ui::main_menu::MenuPage};

use self::{
    client::NetClient,
    proto::{match_setup::MatchSetupMessage, ReliableGameMessageKind},
};

pub mod client;
pub mod proto;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(listen_for_map_changes.run_if_resource_exists::<P2PSession<GgrsConfig>>());
    }
}

#[derive(Debug)]
pub struct GgrsConfig;
impl ggrs::Config for GgrsConfig {
    type Input = networking::DensePlayerControl;
    type State = u8;
    /// Addresses are the same as the player handle for our custom socket.
    type Address = usize;
}

/// TODO: Map changes aren't working on network games for now, so this isn't properly used/working.
fn listen_for_map_changes(
    mut commands: Commands,
    client: Res<NetClient>,
    mut reset_manager: ResetManager,
    mut session_manager: SessionManager,
    mut menu_page: ResMut<MenuPage>,
    mut ridp: ResMut<RollbackIdProvider>,
) {
    while let Some(message) = client.recv_reliable() {
        match message.kind {
            ReliableGameMessageKind::MatchSetup(setup) => match setup {
                MatchSetupMessage::SelectMap(map_handle) => {
                    info!("Other player selected map, starting game");
                    *menu_page = MenuPage::Home;
                    reset_manager.reset_world();

                    commands.spawn((map_handle, Rollback::new(ridp.next_id())));
                    commands.insert_resource(NextState(GameState::InGame));
                    session_manager.start_session();
                }
                other => warn!("Unexpected message during match: {other:?}"),
            },
        }
    }
}
