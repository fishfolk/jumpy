#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::{NetworkMatchSocket, SocketTarget};
use smallvec::SmallVec;

use crate::{ui::player_image::player_image, PackMeta};

use super::*;

#[derive(Default, Clone, Debug, HasSchema)]
pub struct PlayerSelectState {
    pub slots: [PlayerSlot; MAX_PLAYERS as usize],
    /// Cache of available players from the game and packs.
    pub players: Vec<Handle<PlayerMeta>>,
    /// Cache of available hats from the game and packs.
    pub hats: Vec<Option<Handle<HatMeta>>>,
}

impl PlayerSelectState {
    pub fn any_slot_has_source(&self, source: ControlSource) -> bool {
        self.slots
            .iter()
            .any(|slot| slot.user_control_source() == Some(source))
    }

    /// Cache the hats and player assets in PlayerSelectState
    pub fn cache_player_and_hat_assets(
        &mut self,
        meta: &Root<GameMeta>,
        asset_server: Res<AssetServer>,
    ) {
        // Cache the player list
        if self.players.is_empty() {
            for player in meta.core.players.iter() {
                self.players.push(*player);
            }
            for pack in asset_server.packs() {
                let pack_meta = asset_server.get(pack.root.typed::<PackMeta>());
                for player in pack_meta.players.iter() {
                    self.players.push(*player)
                }
            }
        }

        // Cache the hat list
        if self.hats.is_empty() {
            self.hats.push(None); // No hat selected
            for hat in meta.core.player_hats.iter() {
                self.hats.push(Some(*hat));
            }
            for pack in asset_server.packs() {
                let pack_meta = asset_server.get(pack.root.typed::<PackMeta>());
                for hat in pack_meta.player_hats.iter() {
                    self.hats.push(Some(*hat));
                }
            }
        }
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub enum PlayerSlot {
    #[default]
    Empty,
    /// This is used instead of Empty for a required player slot that has not yet selected control source.
    /// (Could be 1st player in local, or the local player in online.)
    SelectingLocalControlSource,
    SelectingPlayer {
        control_source: PlayerSlotControlSource,
        current_player: Handle<PlayerMeta>,
        current_hat: Option<Handle<HatMeta>>,
    },
    SelectingHat {
        control_source: PlayerSlotControlSource,
        selected_player: Handle<PlayerMeta>,
        current_hat: Option<Handle<HatMeta>>,
    },
    Ready {
        control_source: PlayerSlotControlSource,
        selected_player: Handle<PlayerMeta>,
        selected_hat: Option<Handle<HatMeta>>,
    },
}

impl PlayerSlot {
    pub fn is_empty(&self) -> bool {
        matches!(self, Self::Empty)
    }

    pub fn is_selecting_player(&self) -> bool {
        matches!(self, Self::SelectingPlayer { .. })
    }

    pub fn is_selecting_hat(&self) -> bool {
        matches!(self, Self::SelectingHat { .. })
    }

    pub fn is_ready(&self) -> bool {
        matches!(self, Self::Ready { .. })
    }

    pub fn is_local_player(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::SelectingLocalControlSource => true,
            Self::SelectingPlayer { control_source, .. }
            | Self::SelectingHat { control_source, .. }
            | Self::Ready { control_source, .. } => control_source.is_user(),
        }
    }

    pub fn is_ai(&self) -> bool {
        match self {
            Self::Empty => false,
            Self::SelectingLocalControlSource => false,
            Self::SelectingPlayer { control_source, .. }
            | Self::SelectingHat { control_source, .. }
            | Self::Ready { control_source, .. } => control_source.is_ai(),
        }
    }

    pub fn control_source(&self) -> Option<PlayerSlotControlSource> {
        match self {
            Self::Empty => None,
            Self::SelectingLocalControlSource => None,
            Self::SelectingPlayer { control_source, .. }
            | Self::SelectingHat { control_source, .. }
            | Self::Ready { control_source, .. } => Some(*control_source),
        }
    }

    pub fn user_control_source(&self) -> Option<ControlSource> {
        match self {
            Self::Empty => None,
            Self::SelectingLocalControlSource => None,
            Self::SelectingPlayer { control_source, .. }
            | Self::SelectingHat { control_source, .. }
            | Self::Ready { control_source, .. } => control_source.user_source(),
        }
    }

    pub fn selected_player(&self) -> Option<Handle<PlayerMeta>> {
        match self {
            Self::Empty | Self::SelectingLocalControlSource => None,
            Self::SelectingPlayer { current_player, .. } => Some(*current_player),
            Self::SelectingHat {
                selected_player, ..
            } => Some(*selected_player),
            Self::Ready {
                selected_player, ..
            } => Some(*selected_player),
        }
    }

    pub fn selected_hat(&self) -> Option<Handle<HatMeta>> {
        match self {
            Self::Empty | Self::SelectingLocalControlSource => None,
            Self::SelectingPlayer { current_hat, .. } => *current_hat,
            Self::SelectingHat { current_hat, .. } => *current_hat,
            Self::Ready { selected_hat, .. } => *selected_hat,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PlayerSlotControlSource {
    User(ControlSource),
    Remote,
    Ai,
}

impl PlayerSlotControlSource {
    pub fn is_user(self) -> bool {
        matches!(self, Self::User(_))
    }

    pub fn is_ai(self) -> bool {
        matches!(self, Self::Ai)
    }

    pub fn user_source(self) -> Option<ControlSource> {
        match self {
            Self::User(src) => Some(src),
            _ => None,
        }
    }
}

#[cfg(debug_assertions)]
impl PlayerSlot {
    pub fn test_player() -> Option<String> {
        match std::env::var("TEST_PLAYER") {
            Ok(name) => Some(name),
            Err(std::env::VarError::NotUnicode(err)) => {
                warn!("Invalid TEST_PLAYER, not unicode: {err:?}");
                Some("Fishy".to_string())
            }
            Err(std::env::VarError::NotPresent) => {
                let test_vars = [
                    std::env::var_os("TEST_MAP"),
                    std::env::var_os("TEST_HAT"),
                    std::env::var_os("TEST_CONTROLLER"),
                ];
                if test_vars.iter().any(Option::is_some) {
                    Some("Fishy".to_string())
                } else {
                    None
                }
            }
        }
    }

    pub fn test_control_source() -> ControlSource {
        match std::env::var("TEST_CONTROLLER") {
            Ok(name) => match &*name {
                "Keyboard1" => ControlSource::Keyboard1,
                "Keyboard2" => ControlSource::Keyboard2,
                "Gamepad" => ControlSource::Gamepad(0),
                _ => {
                    warn!("Invalid TEST_CONTROLLER: {name}");
                    warn!(r#"Available controllers: "Keyboard1", "Keyboard2", "Gamepad""#);
                    ControlSource::Keyboard1
                }
            },
            Err(std::env::VarError::NotPresent) => ControlSource::Keyboard1,
            Err(std::env::VarError::NotUnicode(err)) => {
                warn!("Invalid TEST_CONTROLLER, not unicode: {err:?}");
                ControlSource::Keyboard1
            }
        }
    }

    pub fn test_hat(
        asset_server: &AssetServer,
        hat_handles: &[Handle<HatMeta>],
    ) -> Option<Handle<HatMeta>> {
        match std::env::var("TEST_HAT") {
            Err(std::env::VarError::NotPresent) => None,
            Err(std::env::VarError::NotUnicode(err)) => {
                warn!("Invalid TEST_HAT, not unicode: {err:?}");
                None
            }
            Ok(test_hat) => match hat_handles
                .iter()
                .copied()
                .find(|h| asset_server.get(*h).name == test_hat)
            {
                Some(hat_handle) => Some(hat_handle),
                None => {
                    warn!("TEST_HAT not found: {test_hat}");
                    let available_names =
                        handle_names_to_string(hat_handles.iter().copied(), |h| {
                            asset_server.get(h).name.as_str()
                        });
                    warn!("Available hat names: {available_names}");
                    None
                }
            },
        }
    }
}

/// Network message that may be sent during player selection.
#[derive(Serialize, Deserialize)]
pub enum PlayerSelectMessage {
    SelectPlayer(NetworkHandle<PlayerMeta>),
    SelectHat(Option<NetworkHandle<HatMeta>>),
    ConfirmSelection(bool),
}

pub fn widget(
    mut ui: In<&mut egui::Ui>,
    meta: Root<GameMeta>,
    localization: Localization<GameMeta>,
    controls: Res<GlobalPlayerControls>,
    world: &World,
    asset_server: Res<AssetServer>,
    #[cfg(not(target_arch = "wasm32"))] network_socket: Option<Res<NetworkMatchSocket>>,
) {
    let mut state = ui.ctx().get_state::<PlayerSelectState>();
    ui.ctx().set_state(EguiInputSettings {
        disable_keyboard_input: true,
        disable_gamepad_input: true,
    });

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(socket) = network_socket.as_ref() {
        handle_match_setup_messages(socket, &mut state, &asset_server);
    }

    // Set player slot 0 using the debug env vars and go to the map select menu.
    // Default the player to Jumpy if none is provided.
    #[cfg(debug_assertions)]
    'test_player: {
        use std::sync::atomic::{AtomicBool, Ordering};
        static DEBUG_DID_CHECK_ENV_VARS: AtomicBool = AtomicBool::new(false);
        if DEBUG_DID_CHECK_ENV_VARS
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            let Some(test_player) = PlayerSlot::test_player() else {
                break 'test_player;
            };

            let test_controller = PlayerSlot::test_control_source();

            let asset_server = world.resource::<AssetServer>();
            let game_meta = asset_server.root::<GameMeta>();
            let core_player_handles = &*game_meta.core.players;
            let core_hat_handles = &*game_meta.core.player_hats;

            match core_player_handles
                .iter()
                .copied()
                .find(|h| asset_server.get(*h).name == test_player)
            {
                None => {
                    warn!("TEST_PLAYER not found: {test_player}");
                    let available_names =
                        handle_names_to_string(core_player_handles.iter().copied(), |h| {
                            asset_server.get(h).name.as_str()
                        });
                    warn!("Available player names: {available_names}");
                }
                Some(player_handle) => {
                    state.slots[0] = PlayerSlot::Ready {
                        control_source: PlayerSlotControlSource::User(test_controller),
                        selected_player: player_handle,
                        selected_hat: PlayerSlot::test_hat(&asset_server, core_hat_handles),
                    };

                    ui.ctx()
                        .set_state(MenuPage::MapSelect { is_waiting: false });
                }
            }
        }
    }

    state.cache_player_and_hat_assets(&meta, asset_server);

    // Initialize state of player slots - we wait on all non-empty slots being ready before allowing
    // transition to map select. Transition slots of required players from empty to initial state.
    //
    // In Offline, we have one required player. Other slots are optional.
    #[cfg(target_arch = "wasm32")]
    {
        let first_slot = &mut state.slots[0];
        if first_slot.is_empty() {
            *first_slot = PlayerSlot::SelectingLocalControlSource;
        }
    }

    // In Online, we have one local player, and some number of remotes that must be initialized.
    #[cfg(not(target_arch = "wasm32"))]
    if let Some(socket) = network_socket.as_ref() {
        for (slot_id, slot) in state.slots.iter_mut().enumerate() {
            let is_local_player_slot = slot_id == socket.player_idx() as usize;
            let is_empty = slot.is_empty();

            if slot_id >= socket.player_count() as usize {
                // unused slots in online don't need to be initialized
                break;
            } else if is_local_player_slot && is_empty {
                *slot = PlayerSlot::SelectingLocalControlSource;
            } else if !is_local_player_slot && is_empty {
                *slot = PlayerSlot::SelectingPlayer {
                    control_source: PlayerSlotControlSource::Remote,
                    // Use default player until we get a message from them on change in selection
                    current_player: state.players[0],
                    current_hat: None,
                };
            }
        }
    }

    // Whether or not the continue button should be enabled
    let mut ready_players = 0;
    let mut unconfirmed_players = 0;

    let mut at_least_one_non_ai_ready = false;
    for slot in &state.slots {
        if slot.is_ready() {
            ready_players += 1;
            if !slot.is_ai() {
                at_least_one_non_ai_ready = true;
            }
        } else if !slot.is_empty() {
            unconfirmed_players += 1;
        }
    }
    let may_continue = ready_players >= 1 && unconfirmed_players == 0 && at_least_one_non_ai_ready;

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(socket) = network_socket.as_ref() {
        if may_continue {
            // The first player picks the map
            let is_waiting = socket.player_idx() != 0;

            ui.ctx().set_state(MenuPage::MapSelect { is_waiting });
        }
    }

    let bigger_text_style = &meta
        .theme
        .font_styles
        .bigger
        .with_color(meta.theme.panel.font_color);
    let heading_text_style = &meta
        .theme
        .font_styles
        .heading
        .with_color(meta.theme.panel.font_color);
    let normal_button_style = &meta.theme.buttons.normal;

    ui.vertical_centered(|ui| {
        ui.add_space(heading_text_style.size / 4.0);

        #[cfg(target_arch = "wasm32")]
        let is_network = false;
        #[cfg(not(target_arch = "wasm32"))]
        let is_network = network_socket.is_some();

        // Title
        if is_network {
            ui.label(heading_text_style.rich(localization.get("online-game")));
        } else {
            ui.label(heading_text_style.rich(localization.get("local-game")));
        }

        ui.label(bigger_text_style.rich(localization.get("player-select-title")));
        ui.add_space(normal_button_style.font.size);

        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
            ui.add_space(normal_button_style.font.size * 2.0);
            ui.horizontal(|ui| {
                // Calculate button size and spacing
                let width = ui.available_width();
                let button_width = width / 3.0;
                let button_min_size = vec2(button_width, 0.0);
                let button_spacing = (width - 2.0 * button_width) / 3.0;

                ui.add_space(button_spacing);

                // Back button
                let back_button =
                    BorderedButton::themed(normal_button_style, localization.get("back"))
                        .min_size(button_min_size)
                        .show(ui)
                        .focus_by_default(ui);

                if back_button.clicked()
                    || (ready_players == 0
                        && unconfirmed_players == 0
                        && controls.values().any(|x| x.menu_back_just_pressed))
                {
                    ui.ctx().set_state(MenuPage::Home);
                    ui.ctx().set_state(EguiInputSettings::default());
                    ui.ctx().set_state(PlayerSelectState::default());

                    #[cfg(not(target_arch = "wasm32"))]
                    if let Some(socket) = network_socket {
                        socket.close();
                    }
                }

                ui.add_space(button_spacing);

                // Continue button
                let continue_button = ui
                    .scope(|ui| {
                        ui.set_enabled(may_continue);

                        BorderedButton::themed(normal_button_style, localization.get("continue"))
                            .min_size(button_min_size)
                            .show(ui)
                    })
                    .inner;

                if !controls.values().any(|x| x.menu_back_just_pressed)
                    && (continue_button.clicked()
                        || (controls.values().any(|x| x.menu_start_just_pressed) && may_continue))
                {
                    ui.ctx()
                        .set_state(MenuPage::MapSelect { is_waiting: false });
                    ui.ctx().set_state(EguiInputSettings::default());
                    ui.ctx().set_state(PlayerSelectState::default());
                }
            });

            ui.add_space(normal_button_style.font.size);

            ui.vertical_centered(|ui| {
                ui.set_width(ui.available_width() - normal_button_style.font.size * 2.0);

                ui.columns(MAX_PLAYERS as usize, |columns| {
                    for (i, ui) in columns.iter_mut().enumerate() {
                        world.run_system(
                            player_select_panel,
                            (ui, u32::try_from(i).unwrap(), &mut state),
                        )
                    }
                });
            });
        });
    });

    ui.ctx().set_state(state);
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_match_setup_messages(
    network_socket: &NetworkMatchSocket,
    player_select_state: &mut PlayerSelectState,
    asset_server: &AssetServer,
) {
    let datas: Vec<(u32, Vec<u8>)> = network_socket.recv_reliable();

    for (player, data) in datas {
        match postcard::from_bytes::<PlayerSelectMessage>(&data) {
            Ok(message) => match message {
                PlayerSelectMessage::SelectPlayer(player_handle) => {
                    let slot = player_select_state.slots[player as usize];
                    let control_source = slot
                        .control_source()
                        .unwrap_or(PlayerSlotControlSource::Remote);
                    let selected_player = player_handle.into_handle(asset_server);
                    let current_hat = slot.selected_hat();
                    player_select_state.slots[player as usize] = PlayerSlot::SelectingHat {
                        control_source,
                        selected_player,
                        current_hat,
                    };
                }
                PlayerSelectMessage::ConfirmSelection(confirmed) => {
                    let slot = player_select_state.slots[player as usize];
                    let control_source = slot
                        .control_source()
                        .unwrap_or(PlayerSlotControlSource::Remote);
                    let selected_player = slot.selected_player().unwrap_or_else(|| {
                            warn!("got confirm message while in empty state, falling back to default player");
                            player_select_state.players[0]
                        });
                    let hat = slot.selected_hat();
                    player_select_state.slots[player as usize] = if confirmed {
                        PlayerSlot::Ready {
                            control_source,
                            selected_player,
                            selected_hat: hat,
                        }
                    } else {
                        PlayerSlot::SelectingHat {
                            control_source,
                            selected_player,
                            current_hat: hat,
                        }
                    };
                }
                PlayerSelectMessage::SelectHat(hat_handle) => {
                    let slot = player_select_state.slots[player as usize];
                    let control_source = slot
                        .control_source()
                        .unwrap_or(PlayerSlotControlSource::Remote);
                    let selected_player = slot.selected_player().unwrap_or_else(|| {
                            warn!("got confirm message while in empty state, falling back to default player");
                            player_select_state.players[0]
                        });
                    let current_hat = hat_handle.map(|h| h.into_handle(asset_server));
                    player_select_state.slots[player as usize] = PlayerSlot::SelectingHat {
                        control_source,
                        selected_player,
                        current_hat,
                    };
                }
            },
            Err(e) => warn!("Ignoring network message that was not understood: {e}"),
        }
    }
}

fn player_select_panel(
    mut params: In<(&mut egui::Ui, u32, &mut PlayerSelectState)>,
    meta: Root<GameMeta>,
    controls: Res<GlobalPlayerControls>,
    asset_server: Res<AssetServer>,
    localization: Localization<GameMeta>,
    mapping: Res<PlayerControlMapping>,
    world: &World,
    #[cfg(not(target_arch = "wasm32"))] network_socket: Option<Res<NetworkMatchSocket>>,
) {
    let (ui, slot_id, state) = &mut *params;
    let slot_id = *slot_id;

    #[cfg(not(target_arch = "wasm32"))]
    let network_socket = network_socket.as_deref();

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(socket) = network_socket {
        // Don't show panels for non-connected players.
        if slot_id + 1 > socket.player_count() {
            return;
        }
    }

    #[cfg(target_arch = "wasm32")]
    let is_network = false;
    #[cfg(not(target_arch = "wasm32"))]
    let is_network = network_socket.is_some();

    let is_next_open_slot = state
        .slots
        .iter()
        .enumerate()
        .any(|(i, slot)| slot.control_source().is_none() && i == slot_id as usize);

    #[cfg(target_arch = "wasm32")]
    let (network_local_player_slot, slot_allows_new_player) = (None::<u32>, is_next_open_slot);
    #[cfg(not(target_arch = "wasm32"))]
    let (network_local_player_slot, slot_allows_new_player) = match network_socket {
        Some(socket) => {
            let socket_id = socket.player_idx();
            (Some(socket_id), slot_id == socket_id)
        }
        None => (None, is_next_open_slot),
    };

    //
    // React to user inputs
    //

    #[cfg(not(target_arch = "wasm32"))]
    let net_send_player = |handle: Handle<PlayerMeta>| {
        if let Some(socket) = network_socket {
            let network_handle = handle.network_handle(&asset_server);
            let message = PlayerSelectMessage::SelectPlayer(network_handle);
            socket.send_reliable(SocketTarget::All, &postcard::to_allocvec(&message).unwrap());
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    let net_send_hat = |handle: Option<Handle<HatMeta>>| {
        if let Some(socket) = network_socket {
            let network_handle = handle.map(|h| h.network_handle(&asset_server));
            let message = PlayerSelectMessage::SelectHat(network_handle);
            socket.send_reliable(SocketTarget::All, &postcard::to_allocvec(&message).unwrap());
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    let net_send_confirm = |confirm| {
        if let Some(socket) = network_socket {
            let message = PlayerSelectMessage::ConfirmSelection(confirm);
            socket.send_reliable(SocketTarget::All, &postcard::to_allocvec(&message).unwrap());
        }
    };

    let mut next_state = None::<PlayerSlot>;

    match state.slots[slot_id as usize] {
        PlayerSlot::Empty | PlayerSlot::SelectingLocalControlSource => {
            if slot_allows_new_player {
                // Check if a new player is trying to join
                let new_player_join = controls.iter().find_map(|(source, control)| {
                    (
                        // If this control input is pressing the join button
                        control.menu_confirm_just_pressed &&
                        // And this control source is not bound to a player slot already
                        !state.any_slot_has_source(*source)
                    )
                    // Return this source
                    .then_some(*source)
                });
                next_state = new_player_join.map(|control_source| PlayerSlot::SelectingPlayer {
                    control_source: PlayerSlotControlSource::User(control_source),
                    current_player: state.players[0],
                    current_hat: None,
                });
            }
        }

        PlayerSlot::SelectingPlayer {
            control_source: control_source @ PlayerSlotControlSource::User(src),
            current_player,
            current_hat,
        } => {
            let Some(player_control) = controls.get(&src) else {
                return;
            };
            if player_control.menu_confirm_just_pressed {
                next_state = Some(PlayerSlot::SelectingHat {
                    control_source,
                    selected_player: current_player,
                    current_hat,
                });
            } else if player_control.menu_back_just_pressed && !is_network {
                next_state = Some(PlayerSlot::Empty);
            } else if player_control.just_moved {
                let current_player_handle_idx = state
                    .players
                    .iter()
                    .enumerate()
                    .find(|(_, handle)| **handle == current_player)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let next_idx = if player_control.move_direction.x > 0.0 {
                    (current_player_handle_idx + 1) % state.players.len()
                } else {
                    (current_player_handle_idx + state.players.len() - 1) % state.players.len()
                };
                let next_player = state.players[next_idx];

                #[cfg(not(target_arch = "wasm32"))]
                net_send_player(next_player);

                next_state = Some(PlayerSlot::SelectingPlayer {
                    control_source,
                    current_player: next_player,
                    current_hat,
                });
            }
        }

        PlayerSlot::SelectingHat {
            control_source: control_source @ PlayerSlotControlSource::User(src),
            selected_player,
            current_hat,
        } => {
            let Some(player_control) = controls.get(&src) else {
                return;
            };
            if player_control.menu_confirm_just_pressed {
                #[cfg(not(target_arch = "wasm32"))]
                net_send_confirm(true);
                next_state = Some(PlayerSlot::Ready {
                    control_source,
                    selected_player,
                    selected_hat: current_hat,
                });
            } else if player_control.menu_back_just_pressed {
                next_state = Some(PlayerSlot::SelectingPlayer {
                    control_source,
                    current_player: selected_player,
                    current_hat,
                });
            } else if player_control.just_moved {
                let current_hat_handle_idx = state
                    .hats
                    .iter()
                    .enumerate()
                    .find(|(_, handle)| **handle == current_hat)
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                let next_idx = if player_control.move_direction.x > 0.0 {
                    (current_hat_handle_idx + 1) % state.hats.len()
                } else {
                    (current_hat_handle_idx + state.hats.len() - 1) % state.hats.len()
                };
                let next_hat = state.hats[next_idx];

                #[cfg(not(target_arch = "wasm32"))]
                net_send_hat(next_hat);

                next_state = Some(PlayerSlot::SelectingHat {
                    control_source,
                    selected_player,
                    current_hat: next_hat,
                });
            }
        }

        PlayerSlot::Ready {
            control_source: control_source @ PlayerSlotControlSource::User(src),
            selected_player,
            selected_hat,
        } => {
            let Some(player_control) = controls.get(&src) else {
                return;
            };
            if player_control.menu_back_just_pressed {
                #[cfg(not(target_arch = "wasm32"))]
                net_send_confirm(false);
                next_state = Some(PlayerSlot::SelectingHat {
                    control_source,
                    selected_player,
                    current_hat: selected_hat,
                });
            }
        }

        _ => {}
    }

    if let Some(slot) = next_state.take() {
        state.slots[slot_id as usize] = slot;
    }

    //
    // Render panel
    //

    // Input sources that may be used to join a new player
    let available_input_sources = {
        let mut sources = SmallVec::<[_; 3]>::from_slice(&[
            ControlSource::Keyboard1,
            ControlSource::Keyboard2,
            ControlSource::Gamepad(0),
        ]);

        for slot in &state.slots {
            if matches!(
                slot.user_control_source(),
                Some(ControlSource::Keyboard1 | ControlSource::Keyboard2)
            ) {
                sources.retain(|&mut x| Some(x) != slot.user_control_source());
            }
        }
        sources
    };

    let panel = &meta.theme.panel;
    BorderedFrame::new(&panel.border)
        .padding(panel.padding)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            let normal_font = &meta.theme.font_styles.normal.with_color(panel.font_color);
            let smaller_font = &meta.theme.font_styles.smaller.with_color(panel.font_color);
            let heading_font = &meta.theme.font_styles.heading.with_color(panel.font_color);

            let slot = state.slots[slot_id as usize];

            // Marker for current player in online matches
            if is_network && slot.is_local_player() {
                ui.vertical_centered(|ui| {
                    ui.label(normal_font.rich(localization.get("you-marker")));
                });
            } else {
                ui.add_space(normal_font.size);
            }

            ui.add_space(normal_font.size);

            let display_fish =
                |ui: &mut egui::Ui,
                 player_meta_handle: Handle<PlayerMeta>,
                 hat_meta_handle: Option<Handle<HatMeta>>| {
                    let player_meta = asset_server.get(player_meta_handle);
                    let hat_meta = hat_meta_handle.map(|h| asset_server.get(h));

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.label(if slot.is_selecting_player() {
                            normal_font.rich(format!("<  {}  >", player_meta.name))
                        } else {
                            normal_font.rich(player_meta.name.as_str())
                        });

                        let hat_label = match slot {
                            PlayerSlot::Empty
                            | PlayerSlot::SelectingLocalControlSource
                            | PlayerSlot::SelectingPlayer { .. } => String::new(),
                            PlayerSlot::SelectingHat { .. } => match hat_meta.as_ref() {
                                Some(hat) => format!("< {} >", hat.name),
                                None => format!("< {} >", localization.get("no-hat")),
                            },
                            PlayerSlot::Ready { .. } => match hat_meta.as_ref() {
                                Some(hat) => hat.name.to_string(),
                                None => localization.get("no-hat").to_string(),
                            },
                        };
                        ui.label(smaller_font.rich(hat_label));

                        world.run_system(player_image, (ui, &player_meta, hat_meta.as_deref()));
                    });
                };

            if let Some(selected_player) = slot.selected_player() {
                let confirm_binding = match slot.user_control_source() {
                    Some(source) => mapping.map_control_source(source).menu_confirm.to_string(),
                    None => available_input_sources
                        .iter()
                        .map(|s| mapping.map_control_source(*s).menu_confirm.to_string())
                        .collect::<SmallVec<[_; 3]>>()
                        .join("/"),
                };

                let back_binding = match slot.user_control_source() {
                    Some(source) => mapping.map_control_source(source).menu_back.to_string(),
                    None => available_input_sources
                        .iter()
                        .map(|s| mapping.map_control_source(*s).menu_back.to_string())
                        .collect::<SmallVec<[_; 3]>>()
                        .join("/"),
                };
                ui.vertical_centered(|ui| {
                    if !slot.is_ready() {
                        if slot.is_local_player() {
                            if slot.is_selecting_hat() {
                                ui.label(normal_font.rich(localization.get("pick-a-hat")));
                            } else {
                                ui.label(normal_font.rich(localization.get("pick-a-fish")));
                            }
                        }

                        if network_local_player_slot.is_some_and(|s| s == slot_id) || !is_network {
                            let label_id = if slot.is_selecting_hat() {
                                "press-button-to-lock-in"
                            } else {
                                "press-button-to-confirm"
                            };
                            ui.label(normal_font.rich(localization.get_with(
                                label_id,
                                &fluent_args! {
                                    "button" => confirm_binding.as_str()
                                },
                            )));
                        }

                        if !is_network {
                            let label_id = if slot.is_selecting_hat() {
                                "press-button-to-go-back"
                            } else {
                                "press-button-to-remove"
                            };
                            ui.label(normal_font.rich(localization.get_with(
                                label_id,
                                &fluent_args! {
                                    "button" => back_binding.as_str()
                                },
                            )));
                        }
                    }

                    ui.vertical_centered(|ui| {
                        ui.set_height(heading_font.size * 1.5);

                        if slot.is_ready() && !slot.is_ai() {
                            ui.label(
                                heading_font
                                    .with_color(meta.theme.colors.positive)
                                    .rich(localization.get("player-select-ready")),
                            );
                            ui.add_space(normal_font.size / 2.0);

                            if slot.is_local_player() {
                                ui.label(normal_font.rich(localization.get_with(
                                    "player-select-unready",
                                    &fluent_args! {
                                        "button" => back_binding.as_str()
                                    },
                                )));
                            }
                        }
                        if !is_network && slot_id != 0 && slot.is_ai() {
                            ui.label(
                                heading_font
                                    .with_color(meta.theme.colors.positive)
                                    .rich(localization.get("ai-player")),
                            );
                            ui.add_space(normal_font.size / 2.0);
                            if BorderedButton::themed(
                                &meta.theme.buttons.normal,
                                localization.get("remove-ai-player"),
                            )
                            .show(ui)
                            .clicked()
                            {
                                next_state = Some(PlayerSlot::Empty);
                            }
                        }
                    });

                    display_fish(ui, selected_player, slot.selected_hat());
                });

            // If this slot has not selected player
            } else {
                let bindings = available_input_sources
                    .iter()
                    .map(|s| mapping.map_control_source(*s).menu_confirm.to_string())
                    .collect::<SmallVec<[_; 3]>>()
                    .join("/");

                ui.vertical_centered(|ui| {
                    if !is_network || slot.is_local_player() {
                        ui.label(normal_font.rich(localization.get_with(
                            "press-button-to-join",
                            &fluent_args! {
                                "button" => bindings
                            },
                        )));
                    }

                    if !is_network {
                        ui.add_space(meta.theme.font_styles.bigger.size);
                        if slot_id != 0
                            && BorderedButton::themed(
                                &meta.theme.buttons.normal,
                                localization.get("add-ai-player"),
                            )
                            .show(ui)
                            .clicked()
                        {
                            let player_idx =
                                THREAD_RNG.with(|rng| rng.usize(0..state.players.len()));
                            next_state = Some(PlayerSlot::Ready {
                                control_source: PlayerSlotControlSource::Ai,
                                selected_player: state.players[player_idx],
                                selected_hat: None,
                            });
                        }
                    } else {
                        // In network play, display default fish/hat for player if not yet selected.
                        let default_player_meta_handle = state.players[0];
                        display_fish(ui, default_player_meta_handle, None);
                    }
                });
            }
        });

    if let Some(slot) = next_state {
        state.slots[slot_id as usize] = slot;
    }
}
