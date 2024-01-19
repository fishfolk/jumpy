use crate::core::{JumpyDefaultMatchRunner, MatchPlugin};
use crate::prelude::*;

use crate::ui::map_select::{map_select_menu, MapSelectAction};

use super::player_select::PlayerSelectState;
use super::MenuPage;

#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::{GgrsSessionRunner, GgrsSessionRunnerInfo, NetworkMatchSocket};

/// Network message that may be sent when selecting a map.
#[derive(Serialize, Deserialize)]
pub enum MapSelectMessage {
    SelectMap(NetworkHandle<MapMeta>),
}

pub fn widget(
    ui: In<&mut egui::Ui>,
    world: &World,
    meta: Root<GameMeta>,
    mut sessions: ResMut<Sessions>,
    mut session_options: ResMut<SessionOptions>,
    assets: Res<AssetServer>,

    #[cfg(not(target_arch = "wasm32"))] network_socket: Option<Res<NetworkMatchSocket>>,
) {
    let mut select_action = MapSelectAction::None;

    // Get map select action from network
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(MapSelectAction::SelectMap(map_meta)) =
        handle_match_setup_messages(&network_socket, &assets)
    {
        select_action = MapSelectAction::SelectMap(map_meta);
    }

    // If no network action - update action from UI
    if matches!(select_action, MapSelectAction::None) {
        select_action = world.run_system(map_select_menu, ());

        #[cfg(not(target_arch = "wasm32"))]
        // Replicate local action
        replicate_map_select_action(&select_action, &network_socket, &assets);
    }

    match select_action {
        MapSelectAction::None => (),
        MapSelectAction::SelectMap(map_handle) => {
            session_options.delete = true;
            ui.ctx().set_state(MenuPage::Home);

            #[cfg(not(target_arch = "wasm32"))]
            let session_runner: Box<dyn SessionRunner> = match network_socket {
                Some(socket) => Box::new(GgrsSessionRunner::<NetworkInputConfig>::new(
                    FPS,
                    GgrsSessionRunnerInfo::from(socket.as_ref()),
                )),
                None => Box::<JumpyDefaultMatchRunner>::default(),
            };
            #[cfg(target_arch = "wasm32")]
            let session_runner = Box::<JumpyDefaultMatchRunner>::default();

            let map_meta = assets.get(map_handle).clone();

            let player_select_state = ui.ctx().get_state::<PlayerSelectState>();
            sessions.start_game(MatchPlugin {
                map: map_meta,
                player_info: std::array::from_fn(|i| {
                    let slot = player_select_state.slots[i];

                    PlayerInput {
                        active: slot.active,
                        selected_player: slot.selected_player,
                        selected_hat: slot.selected_hat,
                        control_source: slot.control_source,
                        editor_input: default(),
                        control: default(),
                        is_ai: slot.is_ai,
                    }
                }),
                plugins: meta.get_plugins(&assets),
                session_runner,
            });
            ui.ctx().set_state(PlayerSelectState::default());
        }
        MapSelectAction::GoBack => ui.ctx().set_state(MenuPage::PlayerSelect),
    }
}

/// Send a MapSelectMessage over network if local player has selected a map.
#[cfg(not(target_arch = "wasm32"))]
fn replicate_map_select_action(
    action: &MapSelectAction,
    socket: &Option<Res<NetworkMatchSocket>>,
    asset_server: &AssetServer,
) {
    use bones_framework::networking::SocketTarget;
    if let Some(socket) = socket {
        if let MapSelectAction::SelectMap(map) = action {
            info!("Sending network SelectMap message.");
            socket.send_reliable(
                SocketTarget::All,
                &postcard::to_allocvec(&MapSelectMessage::SelectMap(
                    map.network_handle(asset_server),
                ))
                .unwrap(),
            );
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_match_setup_messages(
    socket: &Option<Res<NetworkMatchSocket>>,
    asset_server: &AssetServer,
) -> Option<MapSelectAction> {
    if let Some(socket) = socket {
        let datas: Vec<(usize, Vec<u8>)> = socket.recv_reliable();

        for (_player, data) in datas {
            match postcard::from_bytes::<MapSelectMessage>(&data) {
                Ok(message) => match message {
                    MapSelectMessage::SelectMap(map_handle) => {
                        info!("Map select message received, starting game");

                        let handle = map_handle.into_handle(asset_server);
                        return Some(MapSelectAction::SelectMap(handle));
                    }
                },
                Err(e) => {
                    // TODO: The second player in an online match is having this triggered by
                    // picking up a `ConfirmSelection` message, that might have been sent to
                    // _itself_.
                    warn!("Ignoring network message that was not understood: {e} data: {data:?}");
                }
            }
        }
    }

    None
}
