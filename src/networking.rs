#![doc = include_str!("./networking.md")]

use bevy_ggrs::ggrs::P2PSession;

use crate::{
    prelude::*, session::SessionManager, ui::main_menu::MenuPage, utils::ResetManager, GgrsConfig,
};

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

// TODO: Map changes aren't working on network games for now.
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

                    commands
                        .spawn()
                        .insert(map_handle)
                        .insert(Rollback::new(ridp.next_id()));
                    commands.insert_resource(NextState(GameState::InGame));
                    session_manager.start_session();
                }
                other => warn!("Unexpected message during match: {other:?}"),
            },
        }
    }
}
