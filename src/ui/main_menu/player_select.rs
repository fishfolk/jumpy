use crate::{
    loading::PlayerInputCollector,
    metadata::PlayerMeta,
    networking::{
        client::NetClient,
        proto::{
            match_setup::{MatchSetupFromClient, MatchSetupFromServer},
            ClientMatchInfo,
        },
    },
    player::input::PlayerAction,
    player::{input::PlayerInputs, MAX_PLAYERS},
};

use super::*;

#[derive(Default)]
pub struct PlayerSelectState {
    player_slots: [PlayerSlot; MAX_PLAYERS],
}

#[derive(Default)]
struct PlayerSlot {
    confirmed: bool,
}

#[derive(SystemParam)]
pub struct PlayerSelectMenu<'w, 's> {
    game: Res<'w, GameMeta>,
    menu_page: ResMut<'w, MenuPage>,
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
    player_inputs: ResMut<'w, PlayerInputs>,
    player_select_state: ResMut<'w, PlayerSelectState>,
    localization: Res<'w, Localization>,
    client: Option<ResMut<'w, NetClient>>,
    #[system_param(ignore)]
    _phantom: PhantomData<&'s ()>,
}

impl<'w, 's> WidgetSystem for PlayerSelectMenu<'w, 's> {
    type Args = ();
    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        id: WidgetId,
        _: (),
    ) {
        let mut params: PlayerSelectMenu = state.get_mut(world);

        handle_match_setup_messages(&mut params);

        // Whether or not the continue button should be enabled
        let mut ready_players = 0;
        let mut unconfirmed_players = 0;

        for (i, slot) in params.player_select_state.player_slots.iter().enumerate() {
            if slot.confirmed {
                ready_players += 1;
            } else if params.player_inputs.players[i].active {
                unconfirmed_players += 1;
            }
        }
        let may_continue =
            ready_players >= 1 && unconfirmed_players == 0 && params.client.is_none();

        ui.vertical_centered(|ui| {
            let params: PlayerSelectMenu = state.get_mut(world);

            let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
            let heading_text_style = &params.game.ui_theme.font_styles.heading;
            let normal_button_style = &params.game.ui_theme.button_styles.normal;

            ui.add_space(heading_text_style.size / 4.0);
            ui.themed_label(heading_text_style, &params.localization.get("local-game"));
            ui.themed_label(
                bigger_text_style,
                &params.localization.get("player-select-title"),
            );
            ui.add_space(normal_button_style.font.size);

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                let mut params: PlayerSelectMenu = state.get_mut(world);

                let normal_button_style = &params.game.ui_theme.button_styles.normal;

                ui.add_space(normal_button_style.font.size * 2.0);

                ui.horizontal(|ui| {
                    // Calculate button size and spacing
                    let width = ui.available_width();
                    let button_width = width / 3.0;
                    let button_min_size = egui::vec2(button_width, 0.0);
                    let button_spacing = (width - 2.0 * button_width) / 3.0;

                    ui.add_space(button_spacing);

                    // Back button
                    let back_button = BorderedButton::themed(
                        &params.game.ui_theme.button_styles.normal,
                        &params.localization.get("back"),
                    )
                    .min_size(button_min_size)
                    .show(ui)
                    .focus_by_default(ui);

                    // Go to menu when back button is clicked
                    if back_button.clicked()
                        || params.menu_input.single().just_pressed(MenuAction::Back)
                    {
                        *params.menu_page = MenuPage::Home;
                        if let Some(client) = params.client {
                            client.close();
                        }
                        ui.ctx().clear_focus();
                    }

                    ui.add_space(button_spacing);

                    // Continue button
                    let continue_button = ui
                        .scope(|ui| {
                            ui.set_enabled(may_continue);

                            BorderedButton::themed(
                                &params.game.ui_theme.button_styles.normal,
                                &params.localization.get("continue"),
                            )
                            .min_size(button_min_size)
                            .show(ui)
                        })
                        .inner;

                    if continue_button.clicked() || (params.menu_input.single().just_pressed(MenuAction::Start)) {
                        *params.menu_page = MenuPage::MapSelect { is_waiting: false };
                    }
                });

                ui.add_space(normal_button_style.font.size);

                ui.vertical_centered(|ui| {
                    let params: PlayerSelectMenu = state.get_mut(world);

                    let normal_button_style = &params.game.ui_theme.button_styles.normal;
                    ui.set_width(ui.available_width() - normal_button_style.font.size * 2.0);

                    ui.columns(MAX_PLAYERS, |columns| {
                        for (i, ui) in columns.iter_mut().enumerate() {
                            widget::<PlayerSelectPanel>(
                                world,
                                ui,
                                id.with(&format!("player_panel{i}")),
                                i,
                            );
                        }
                    });
                });
            });
        });
    }
}

fn handle_match_setup_messages(params: &mut PlayerSelectMenu) {
    if let Some(client) = &mut params.client {
        while let Some(message) = client.recv_reliable::<MatchSetupFromServer>() {
            match message {
                MatchSetupFromServer::ClientMessage {
                    player_idx,
                    message,
                } => match message {
                    MatchSetupFromClient::SelectPlayer(player_handle) => {
                        params.player_inputs.players[player_idx as usize].selected_player =
                            player_handle;
                    }
                    MatchSetupFromClient::ConfirmSelection(confirmed) => {
                        params.player_select_state.player_slots[player_idx as usize].confirmed =
                            confirmed;
                    }
                    message => {
                        warn!("Unexpected message in player select menu: {message:?}");
                    }
                },
                MatchSetupFromServer::SelectMap => {
                    *params.menu_page = MenuPage::MapSelect { is_waiting: false };
                }
                MatchSetupFromServer::WaitForMapSelect => {
                    *params.menu_page = MenuPage::MapSelect { is_waiting: true };
                }
            }
        }
    }
}

#[derive(SystemParam)]
struct PlayerSelectPanel<'w, 's> {
    game: Res<'w, GameMeta>,
    client_match_info: Option<Res<'w, ClientMatchInfo>>,
    player_inputs: ResMut<'w, PlayerInputs>,
    player_select_state: ResMut<'w, PlayerSelectState>,
    players: Query<
        'w,
        's,
        (
            &'static PlayerInputCollector,
            &'static ActionState<PlayerAction>,
        ),
    >,
    player_meta_assets: Res<'w, Assets<PlayerMeta>>,
    client: Option<Res<'w, NetClient>>,
    localization: Res<'w, Localization>,
    #[system_param(ignore)]
    _phantom: PhantomData<&'s ()>,
}

impl<'w, 's> WidgetSystem for PlayerSelectPanel<'w, 's> {
    type Args = usize;
    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        _id: WidgetId,
        idx: usize,
    ) {
        let mut params: PlayerSelectPanel = state.get_mut(world);
        let dummy_actions = default();

        let player_actions = if let Some(match_info) = &params.client_match_info {
            if idx == match_info.player_idx {
                params
                    .players
                    .iter()
                    .find(|(player_idx, _)| player_idx.0 == 0)
                    .unwrap()
                    .1
            } else {
                &dummy_actions
            }
        } else {
            params
                .players
                .iter()
                .find(|(player_idx, _)| player_idx.0 == idx)
                .unwrap()
                .1
        };

        let player_input = &mut params.player_inputs.players[idx];
        if !player_input.active && params.client.is_some() {
            return;
        }

        let player_handle = &mut player_input.selected_player;

        let slot = &mut params.player_select_state.player_slots[idx];

        if player_actions.just_pressed(PlayerAction::Jump) {
            if params.client.is_none() {
                if player_input.active {
                    slot.confirmed = true;
                } else {
                    player_input.active = true;
                }
            } else {
                slot.confirmed = true;
            }

            if let Some(client) = params.client {
                client.send_reliable(&MatchSetupFromClient::ConfirmSelection(slot.confirmed));
            }
        } else if player_actions.just_pressed(PlayerAction::Grab) {
            if params.client.is_none() {
                if slot.confirmed {
                    slot.confirmed = false;
                } else {
                    player_input.active = false;
                }
            } else {
                slot.confirmed = false;
            }
            if let Some(client) = params.client {
                client.send_reliable(&MatchSetupFromClient::ConfirmSelection(slot.confirmed));
            }
        } else if player_actions.just_pressed(PlayerAction::Move) && !slot.confirmed {
            let direction = player_actions
                .clamped_axis_pair(PlayerAction::Move)
                .unwrap();

            let current_player_handle_idx = params
                .game
                .player_handles
                .iter()
                .enumerate()
                .find(|(_, handle)| handle.inner == player_handle.inner)
                .map(|(i, _)| i)
                .unwrap_or(0);

            if direction.x() > 0.0 {
                *player_handle = params
                    .game
                    .player_handles
                    .get(current_player_handle_idx + 1)
                    .map(|x| x.clone_weak())
                    .unwrap_or_else(|| params.game.player_handles[0].clone_weak());
            } else if direction.x() <= 0.0 {
                if current_player_handle_idx > 0 {
                    *player_handle = params
                        .game
                        .player_handles
                        .get(current_player_handle_idx - 1)
                        .map(|x| x.clone_weak())
                        .unwrap();
                } else {
                    *player_handle = params
                        .game
                        .player_handles
                        .iter()
                        .last()
                        .unwrap()
                        .clone_weak();
                }
            }

            if let Some(client) = params.client {
                client.send_reliable(&MatchSetupFromClient::SelectPlayer(
                    player_handle.clone_weak(),
                ));
            }
        }

        BorderedFrame::new(&params.game.ui_theme.panel.border)
            .padding(params.game.ui_theme.panel.padding.into())
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.set_height(ui.available_height());

                let normal_font = &params.game.ui_theme.font_styles.normal;
                let heading_font = &params.game.ui_theme.font_styles.heading;

                // Marker for current player in online matches
                if let Some(match_info) = params.client_match_info {
                    if match_info.player_idx == idx {
                        ui.vertical_centered(|ui| {
                            ui.themed_label(normal_font, &params.localization.get("you-marker"));
                        });
                    } else {
                        ui.add_space(normal_font.size);
                    }
                } else {
                    ui.add_space(normal_font.size);
                }

                if player_input.active {
                    ui.vertical_centered(|ui| {
                        let player_meta =
                            if let Some(meta) = params.player_meta_assets.get(player_handle) {
                                meta
                            } else {
                                return;
                            };

                        ui.themed_label(normal_font, &params.localization.get("pick-a-fish"));

                        ui.vertical_centered(|ui| {
                            ui.set_height(heading_font.size * 1.5);

                            if slot.confirmed {
                                ui.themed_label(
                                    &heading_font.colored(params.game.ui_theme.colors.positive),
                                    &params.localization.get("player-select-ready"),
                                );
                            }
                        });

                        ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                            let name_with_arrows = format!("<  {}  >", player_meta.name);
                            ui.themed_label(
                                normal_font,
                                if slot.confirmed {
                                    &player_meta.name
                                } else {
                                    &name_with_arrows
                                },
                            );

                            player_image(ui, player_meta);
                        });
                    });
                } else {
                    ui.vertical_centered(|ui| {
                        ui.themed_label(
                            normal_font,
                            &params.localization.get("press-jump-to-join"),
                        );
                    });
                }
            });
    }
}

fn player_image(ui: &mut egui::Ui, player_meta: &PlayerMeta) {
    let time = ui.ctx().input().time;
    let spritesheet = &player_meta.spritesheet;
    let tile_size = spritesheet.tile_size.as_vec2();
    let spritesheet_size =
        tile_size * Vec2::new(spritesheet.columns as f32, spritesheet.rows as f32);
    let sprite_aspect = tile_size.y / tile_size.x;
    let width = ui.available_width();
    let height = sprite_aspect * width;
    let available_height = ui.available_height();
    let y_offset = -(available_height - height) / 2.0;
    let (rect, _) = ui.allocate_exact_size(egui::vec2(width, height), egui::Sense::hover());

    let fps = spritesheet.animation_fps as f64;
    let anim_clip = spritesheet
        .animations
        .get("idle")
        .expect("Missing `idle` animation");
    let frame_in_time_idx = (time * fps).round() as usize;
    let frame_in_clip_idx = frame_in_time_idx % anim_clip.frames.len();
    let frame_in_sheet_idx = anim_clip.frames.clone().nth(frame_in_clip_idx).unwrap();
    let x_in_sheet_idx = frame_in_sheet_idx % spritesheet.columns;
    let y_in_sheet_idx = frame_in_sheet_idx / spritesheet.columns;

    let uv = egui::Rect::from_min_size(
        egui::pos2(
            x_in_sheet_idx as f32 * tile_size.x / spritesheet_size.x,
            y_in_sheet_idx as f32 * tile_size.y / spritesheet_size.y,
        ),
        egui::vec2(
            1.0 / spritesheet.columns as f32,
            1.0 / spritesheet.rows as f32,
        ),
    );

    let mut mesh = egui::Mesh {
        texture_id: spritesheet.egui_texture_id,
        ..default()
    };

    mesh.add_rect_with_uv(rect, uv, egui::Color32::WHITE);

    mesh.translate(egui::vec2(0.0, y_offset));

    ui.painter().add(mesh);
}
