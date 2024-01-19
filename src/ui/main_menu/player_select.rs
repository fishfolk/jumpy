#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::{NetworkMatchSocket, SocketTarget};

use crate::PackMeta;

use super::*;

// const GAMEPAD_ACTION_IDX: usize = 0;
// const KEYPAD_ACTION_IDX: usize = 1;

#[derive(Default, Clone, Debug, HasSchema)]
pub struct PlayerSelectState {
    pub slots: [PlayerSlot; MAX_PLAYERS],
    /// Cache of available players from the game and packs.
    pub players: Vec<Handle<PlayerMeta>>,
    /// Cache of available hats from the game and packs.
    pub hats: Vec<Option<Handle<HatMeta>>>,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct PlayerSlot {
    pub active: bool,
    pub confirmed: bool,
    pub selected_player: Handle<PlayerMeta>,
    pub selected_hat: Option<Handle<HatMeta>>,
    pub control_source: Option<ControlSource>,
    pub is_ai: bool,
}

impl PlayerSlot {
    pub fn is_ai(&self) -> bool {
        self.is_ai
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

    #[cfg(not(target_arch = "wasm32"))] asset_server: Res<AssetServer>,
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

    // Whether or not the continue button should be enabled
    let mut ready_players = 0;
    let mut unconfirmed_players = 0;

    for slot in &state.slots {
        if slot.confirmed {
            ready_players += 1;
        } else if slot.active {
            unconfirmed_players += 1;
        }
    }
    let may_continue = ready_players >= 1 && unconfirmed_players == 0;

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

                ui.columns(MAX_PLAYERS, |columns| {
                    for (i, ui) in columns.iter_mut().enumerate() {
                        world.run_system(player_select_panel, (ui, i, &mut state))
                    }
                });
            });
        });
    });

    ui.ctx().set_state(state);
}

// }

#[cfg(not(target_arch = "wasm32"))]
fn handle_match_setup_messages(
    network_socket: &NetworkMatchSocket,
    player_select_state: &mut PlayerSelectState,
    asset_server: &AssetServer,
) {
    let datas: Vec<(usize, Vec<u8>)> = network_socket.recv_reliable();

    for (player, data) in datas {
        match postcard::from_bytes::<PlayerSelectMessage>(&data) {
            Ok(message) => match message {
                PlayerSelectMessage::SelectPlayer(player_handle) => {
                    let player_handle = player_handle.into_handle(asset_server);
                    player_select_state.slots[player].selected_player = player_handle;
                }
                PlayerSelectMessage::ConfirmSelection(confirmed) => {
                    player_select_state.slots[player].confirmed = confirmed;
                }
                PlayerSelectMessage::SelectHat(hat) => {
                    let hat = hat.map(|hat| hat.into_handle(asset_server));
                    player_select_state.slots[player].selected_hat = hat;
                }
            },
            Err(e) => warn!("Ignoring network message that was not understood: {e}"),
        }
    }
}

fn player_select_panel(
    mut params: In<(&mut egui::Ui, usize, &mut PlayerSelectState)>,
    meta: Root<GameMeta>,
    controls: Res<GlobalPlayerControls>,
    asset_server: Res<AssetServer>,
    localization: Localization<GameMeta>,
    mapping: Res<PlayerControlMapping>,
    world: &World,
    #[cfg(not(target_arch = "wasm32"))] network_socket: Option<Res<NetworkMatchSocket>>,
) {
    let (ui, slot_id, state) = &mut *params;

    // Cache the player list
    if state.players.is_empty() {
        for player in meta.core.players.iter() {
            state.players.push(*player);
        }
        for pack in asset_server.packs() {
            let pack_meta = asset_server.get(pack.root.typed::<PackMeta>());
            for player in pack_meta.players.iter() {
                state.players.push(*player)
            }
        }
    }

    // Cache the hat list
    if state.hats.is_empty() {
        state.hats.push(None); // No hat selected
        for hat in meta.core.player_hats.iter() {
            state.hats.push(Some(*hat));
        }
        for pack in asset_server.packs() {
            let pack_meta = asset_server.get(pack.root.typed::<PackMeta>());
            for hat in pack_meta.player_hats.iter() {
                state.hats.push(Some(*hat));
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    let network_local_player_slot: Option<usize> = None;

    #[cfg(target_arch = "wasm32")]
    let is_network = false;
    #[cfg(not(target_arch = "wasm32"))]
    let is_network = network_socket.is_some();

    #[cfg(not(target_arch = "wasm32"))]
    if let Some(socket) = network_socket.as_ref() {
        // Don't show panels for non-connected players.
        if *slot_id + 1 > socket.player_count() {
            return;
        } else {
            state.slots[*slot_id].active = true;
        }
    }

    let is_next_open_slot = state
        .slots
        .iter()
        .enumerate()
        .any(|(i, slot)| (!slot.active && i == *slot_id));

    #[cfg(not(target_arch = "wasm32"))]
    let mut network_local_player_slot: Option<usize> = None;

    #[cfg(not(target_arch = "wasm32"))]
    let slot_allows_new_player = if is_network {
        {
            network_local_player_slot = Some(network_socket.as_ref().unwrap().player_idx());
            *slot_id == network_local_player_slot.unwrap()
        }
    } else {
        is_next_open_slot
    };
    #[cfg(target_arch = "wasm32")]
    let slot_allows_new_player = is_next_open_slot;

    // Check if a new player is trying to join
    let new_player_join = controls.iter().find_map(|(source, control)| {
        (
            // If this control input is pressing the join button
            control.menu_confirm_just_pressed &&
            slot_allows_new_player &&
            // And this control source is not bound to a player slot already
            !state
            .slots
            .iter()
            .any(|s| s.control_source == Some(*source))
        )
        // Return this source
        .then_some(*source)
    });

    // Input sources that may be used to join a new player
    let available_input_sources = {
        let mut sources = SmallVec::<[_; 3]>::from_slice(&[
            ControlSource::Keyboard1,
            ControlSource::Keyboard2,
            ControlSource::Gamepad(0),
        ]);

        for slot in &state.slots {
            if matches!(
                slot.control_source,
                Some(ControlSource::Keyboard1 | ControlSource::Keyboard2)
            ) {
                sources.retain(|&mut x| x != slot.control_source.unwrap());
            }
        }
        sources
    };

    let slot = &mut state.slots[*slot_id];
    let player_handle = &mut slot.selected_player;

    // If the handle is empty
    if *player_handle == default() {
        // Select the first player
        *player_handle = state.players[0];
    }

    // Handle player joining
    if let Some(control_source) = new_player_join {
        slot.active = true;
        slot.confirmed = false;
        slot.control_source = Some(control_source);
    }

    let player_control = slot
        .control_source
        .as_ref()
        .map(|s| *controls.get(s).unwrap())
        .unwrap_or_default();

    if new_player_join.is_none() {
        if player_control.menu_confirm_just_pressed {
            slot.confirmed = true;

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(socket) = network_socket.as_ref() {
                socket.send_reliable(
                    SocketTarget::All,
                    &postcard::to_allocvec(&PlayerSelectMessage::ConfirmSelection(slot.confirmed))
                        .unwrap(),
                );
            }
        } else if player_control.menu_back_just_pressed {
            if !is_network {
                if slot.confirmed {
                    slot.confirmed = false;
                } else if slot.active {
                    slot.active = false;
                    slot.control_source = None;
                }
            } else {
                slot.confirmed = false;
            }

            #[cfg(not(target_arch = "wasm32"))]
            if let Some(socket) = network_socket.as_ref() {
                socket.send_reliable(
                    SocketTarget::All,
                    &postcard::to_allocvec(&PlayerSelectMessage::ConfirmSelection(slot.confirmed))
                        .unwrap(),
                );
            }
        } else if player_control.just_moved {
            let direction = player_control.move_direction;

            // Select a hat if the player has been confirmed
            if slot.confirmed {
                let current_hat_handle_idx = state
                    .hats
                    .iter()
                    .enumerate()
                    .find(|(_, handle)| **handle == slot.selected_hat)
                    .map(|(i, _)| i)
                    .unwrap_or(0);

                let next_idx = if direction.x > 0.0 {
                    (current_hat_handle_idx + 1) % state.hats.len()
                } else {
                    let idx = current_hat_handle_idx as i32 - 1;
                    if idx == -1 {
                        state.hats.len() - 1
                    } else {
                        idx as usize
                    }
                };
                slot.selected_hat = state.hats[next_idx];

                #[cfg(not(target_arch = "wasm32"))]
                if let Some(socket) = network_socket.as_ref() {
                    socket.send_reliable(
                        SocketTarget::All,
                        &postcard::to_allocvec(&PlayerSelectMessage::SelectHat(
                            slot.selected_hat.map(|x| x.network_handle(&asset_server)),
                        ))
                        .unwrap(),
                    );
                }

                // Select player skin if the player has not be confirmed
            } else {
                let current_player_handle_idx = state
                    .players
                    .iter()
                    .enumerate()
                    .find(|(_, handle)| *handle == player_handle)
                    .map(|(i, _)| i)
                    .unwrap_or(0);
                let next_idx = if direction.x > 0.0 {
                    (current_player_handle_idx + 1) % state.players.len()
                } else {
                    let idx = current_player_handle_idx as i32 - 1;
                    if idx == -1 {
                        state.players.len() - 1
                    } else {
                        idx as usize
                    }
                };
                *player_handle = state.players[next_idx];

                #[cfg(not(target_arch = "wasm32"))]
                if let Some(socket) = network_socket.as_ref() {
                    socket.send_reliable(
                        SocketTarget::All,
                        &postcard::to_allocvec(&PlayerSelectMessage::SelectPlayer(
                            player_handle.network_handle(&asset_server),
                        ))
                        .unwrap(),
                    );
                }
            }
        }
    }

    let panel = &meta.theme.panel;
    BorderedFrame::new(&panel.border)
        .padding(panel.padding)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            let normal_font = &meta.theme.font_styles.normal.with_color(panel.font_color);
            let smaller_font = &meta.theme.font_styles.smaller.with_color(panel.font_color);
            let heading_font = &meta.theme.font_styles.heading.with_color(panel.font_color);

            // Marker for current player in online matches
            #[cfg(not(target_arch = "wasm32"))]
            if let Some(socket) = network_socket {
                if socket.player_idx() == *slot_id {
                    ui.vertical_centered(|ui| {
                        ui.label(normal_font.rich(localization.get("you-marker")));
                    });
                } else {
                    ui.add_space(normal_font.size);
                }
            } else {
                ui.add_space(normal_font.size);
            }

            ui.add_space(normal_font.size);

            if slot.active {
                let confirm_binding = match slot.control_source {
                    Some(source) => mapping.map_control_source(source).menu_confirm.to_string(),
                    None => available_input_sources
                        .iter()
                        .map(|s| mapping.map_control_source(*s).menu_confirm.to_string())
                        .collect::<SmallVec<[_; 3]>>()
                        .join("/"),
                };

                let back_binding = match slot.control_source {
                    Some(source) => mapping.map_control_source(source).menu_back.to_string(),
                    None => available_input_sources
                        .iter()
                        .map(|s| mapping.map_control_source(*s).menu_back.to_string())
                        .collect::<SmallVec<[_; 3]>>()
                        .join("/"),
                };
                ui.vertical_centered(|ui| {
                    let player_meta = asset_server.get(slot.selected_player);
                    let hat_meta = slot
                        .selected_hat
                        .as_ref()
                        .map(|handle| asset_server.get(*handle));

                    if !slot.confirmed {
                        if slot.control_source.is_some() {
                            ui.label(normal_font.rich(localization.get("pick-a-fish")));
                        }

                        if network_local_player_slot.is_some_and(|s| s == *slot_id) || !is_network {
                            ui.label(normal_font.rich(localization.get_with(
                                "press-button-to-lock-in",
                                &fluent_args! {
                                    "button" => confirm_binding.as_str()
                                },
                            )));
                        }

                        if !is_network {
                            ui.label(normal_font.rich(localization.get_with(
                                "press-button-to-remove",
                                &fluent_args! {
                                    "button" => back_binding.as_str()
                                },
                            )));
                        }
                    } else {
                        ui.label(normal_font.rich(localization.get("waiting")));
                    }

                    ui.vertical_centered(|ui| {
                        ui.set_height(heading_font.size * 1.5);

                        if slot.confirmed && !slot.is_ai() && slot.control_source.is_some() {
                            ui.label(
                                heading_font
                                    .with_color(meta.theme.colors.positive)
                                    .rich(localization.get("player-select-ready")),
                            );
                            ui.add_space(normal_font.size / 2.0);
                            ui.label(normal_font.rich(localization.get_with(
                                "player-select-unready",
                                &fluent_args! {
                                    "button" => back_binding.as_str()
                                },
                            )));
                        }
                        if !is_network && *slot_id != 0 && slot.is_ai() {
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
                                slot.confirmed = false;
                                slot.active = false;
                                slot.control_source = None;
                                slot.is_ai = false;
                            }
                        }
                    });

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        let name_with_arrows = format!("<  {}  >", player_meta.name);
                        ui.label(normal_font.rich(if slot.confirmed {
                            player_meta.name.to_string()
                        } else {
                            name_with_arrows
                        }));
                        let hat_label = if let Some(hat_meta) = &hat_meta {
                            format!("< {} >", hat_meta.name)
                        } else {
                            format!("< {} >", localization.get("no-hat"))
                        };
                        ui.label(smaller_font.rich(if slot.confirmed { &hat_label } else { "" }));

                        world.run_system(player_image, (ui, &player_meta, hat_meta.as_deref()));
                    });
                });

            // If this slot is empty
            } else {
                let bindings = available_input_sources
                    .iter()
                    .map(|s| mapping.map_control_source(*s).menu_confirm.to_string())
                    .collect::<SmallVec<[_; 3]>>()
                    .join("/");

                ui.vertical_centered(|ui| {
                    ui.label(normal_font.rich(localization.get_with(
                        "press-button-to-join",
                        &fluent_args! {
                            "button" => bindings
                        },
                    )));

                    if !is_network {
                        ui.add_space(meta.theme.font_styles.bigger.size);
                        if BorderedButton::themed(
                            &meta.theme.buttons.normal,
                            localization.get("add-ai-player"),
                        )
                        .show(ui)
                        .clicked()
                        {
                            slot.is_ai = true;
                            slot.confirmed = true;
                            slot.active = true;
                            let rand_idx = THREAD_RNG.with(|rng| rng.usize(0..state.players.len()));
                            slot.selected_player = state.players[rand_idx];
                        }
                    }
                });
            }
        });
}

fn player_image(
    mut params: In<(&mut egui::Ui, &PlayerMeta, Option<&HatMeta>)>,
    egui_textures: Res<EguiTextures>,
    asset_server: Res<AssetServer>,
) {
    let (ui, player_meta, hat_meta) = &mut *params;
    let time = ui.ctx().input(|i| i.time as f32);
    let width = ui.available_width();
    let available_height = ui.available_width();

    let body_rect;
    let body_scale;
    let body_offset;
    let y_offset;
    // Render the body sprite
    {
        let atlas_handle = &player_meta.layers.body.atlas;
        let atlas = asset_server.get(*atlas_handle);
        let anim_clip = player_meta
            .layers
            .body
            .animations
            .frames
            .get(&ustr("idle"))
            .unwrap();
        let fps = anim_clip.fps;
        let frame_in_time_idx = (time * fps).round() as usize;
        let frame_in_clip_idx = frame_in_time_idx % anim_clip.frames.len();
        let frame_in_sheet_idx = anim_clip.frames[frame_in_clip_idx];
        let sprite_pos = atlas.tile_pos(frame_in_sheet_idx);
        body_offset =
            player_meta.layers.body.animations.offsets[&ustr("idle")][frame_in_clip_idx].body;

        let sprite_aspect = atlas.tile_size.y / atlas.tile_size.x;
        let height = sprite_aspect * width;
        y_offset = -(available_height - height) / 2.0;
        let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

        let uv_min = sprite_pos / atlas.size();
        let uv_max = (sprite_pos + atlas.tile_size) / atlas.size();
        let uv = egui::Rect {
            min: egui::pos2(uv_min.x, uv_min.y),
            max: egui::pos2(uv_max.x, uv_max.y),
        };

        let mut mesh = egui::Mesh {
            texture_id: *egui_textures.0.get(&atlas.image).unwrap(),
            ..default()
        };

        mesh.add_rect_with_uv(rect, uv, egui::Color32::WHITE);
        mesh.translate(egui::vec2(0.0, y_offset));
        ui.painter().add(mesh);

        body_rect = rect;
        body_scale = width / atlas.tile_size.x;
    }

    // Render the fin & face animation
    for layer in [&player_meta.layers.fin, &player_meta.layers.face] {
        let atlas_handle = &layer.atlas;
        let atlas = asset_server.get(*atlas_handle);
        let anim_clip = layer.animations.get(&ustr("idle")).unwrap();
        let fps = anim_clip.fps;
        let frame_in_time_idx = (time * fps).round() as usize;
        let frame_in_clip_idx = frame_in_time_idx % anim_clip.frames.len();
        let frame_in_sheet_idx = anim_clip.frames[frame_in_clip_idx];
        let sprite_pos = atlas.tile_pos(frame_in_sheet_idx);

        let uv_min = sprite_pos / atlas.size();
        let uv_max = (sprite_pos + atlas.tile_size) / atlas.size();
        let uv = egui::Rect {
            min: egui::pos2(uv_min.x, uv_min.y),
            max: egui::pos2(uv_max.x, uv_max.y),
        };

        let mut mesh = egui::Mesh {
            texture_id: *egui_textures.0.get(&atlas.image).unwrap(),
            ..default()
        };

        let sprite_size = atlas.tile_size * body_scale;
        let offset = (layer.offset + body_offset) * body_scale;
        let rect = egui::Rect::from_center_size(
            body_rect.center() + egui::vec2(offset.x, -offset.y + y_offset),
            egui::vec2(sprite_size.x, sprite_size.y),
        );

        mesh.add_rect_with_uv(rect, uv, egui::Color32::WHITE);
        ui.painter().add(mesh);
    }

    // Render the player hat
    if let Some(hat_meta) = hat_meta {
        let atlas_handle = &hat_meta.atlas;
        let atlas = asset_server.get(*atlas_handle);
        let sprite_pos = Vec2::ZERO;

        let uv_min = sprite_pos / atlas.size();
        let uv_max = (sprite_pos + atlas.tile_size) / atlas.size();
        let uv = egui::Rect {
            min: egui::pos2(uv_min.x, uv_min.y),
            max: egui::pos2(uv_max.x, uv_max.y),
        };

        let mut mesh = egui::Mesh {
            texture_id: *egui_textures.0.get(&atlas.image).unwrap(),
            ..default()
        };

        let sprite_size = atlas.tile_size * body_scale;
        let offset = (hat_meta.offset + body_offset) * body_scale;
        let rect = egui::Rect::from_center_size(
            body_rect.center() + egui::vec2(offset.x, -offset.y + y_offset),
            egui::vec2(sprite_size.x, sprite_size.y),
        );

        mesh.add_rect_with_uv(rect, uv, egui::Color32::WHITE);
        ui.painter().add(mesh);
    }
}
