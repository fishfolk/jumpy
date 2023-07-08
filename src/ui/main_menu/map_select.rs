use crate::{editor::UserMapStorage, ui::pause_menu::PauseMenuPage};

#[cfg(not(target_arch = "wasm32"))]
use crate::networking::{GgrsSessionRunnerInfo, NetworkMatchSocket, SocketTarget};

use super::*;

/// Network message that may be sent when selecting a map.
#[derive(Serialize, Deserialize)]
pub enum MapSelectMessage {
    SelectMap(bones::Handle<MapMeta>),
}

#[derive(SystemParam)]
pub struct MapSelectMenu<'w, 's> {
    menu_input: Query<'w, 's, &'static mut ActionState<MenuAction>>,
    menu_page: ResMut<'w, MenuPage>,
    session_manager: SessionManager<'w, 's>,
    game: Res<'w, GameMeta>,
    core: Res<'w, CoreMetaArc>,
    player_select_state: Res<'w, super::player_select::PlayerSelectState>,
    game_state: Res<'w, State<EngineState>>,
    pause_page: ResMut<'w, PauseMenuPage>,
    commands: Commands<'w, 's>,
    localization: Res<'w, Localization>,
    map_assets: Res<'w, Assets<MapMeta>>,
    storage: ResMut<'w, Storage>,
    #[cfg(not(target_arch = "wasm32"))]
    network_socket: Option<Res<'w, NetworkMatchSocket>>,
}

impl<'w, 's> WidgetSystem for MapSelectMenu<'w, 's> {
    type Args = bool;

    fn system(
        world: &mut World,
        state: &mut SystemState<Self>,
        ui: &mut egui::Ui,
        _id: WidgetId,
        is_waiting: Self::Args,
    ) {
        let mut params: MapSelectMenu = state.get_mut(world);

        #[cfg(not(target_arch = "wasm32"))]
        handle_match_setup_messages(&mut params);

        let in_game = params.game_state.0 == EngineState::InGame;

        if params.menu_input.single().just_pressed(MenuAction::Back) {
            // If we are on the main menu
            if params.game_state.0 == EngineState::MainMenu {
                *params.menu_page = MenuPage::PlayerSelect;

            // If we're on a map selection in game, we must be in the pause menu
            } else if in_game {
                *params.pause_page = PauseMenuPage::Default;
            }
        }

        ui.vertical_centered_justified(|ui| {
            let bigger_text_style = &params.game.ui_theme.font_styles.bigger;
            let heading_text_style = &params.game.ui_theme.font_styles.heading;
            let small_button_style = &params.game.ui_theme.button_styles.small;

            if !in_game {
                ui.add_space(heading_text_style.size / 4.0);
                ui.themed_label(heading_text_style, &params.localization.get("local-game"));
                ui.themed_label(
                    bigger_text_style,
                    &params.localization.get("map-select-title"),
                );
                ui.add_space(small_button_style.font.size);
            }

            let available_size = ui.available_size();
            let menu_width = params.game.main_menu.menu_width;
            let x_margin = (available_size.x - menu_width) / 2.0;
            let outer_margin = egui::style::Margin::symmetric(x_margin, heading_text_style.size);

            if is_waiting {
                ui.themed_label(
                    bigger_text_style,
                    &params.localization.get("waiting-for-map"),
                );
            } else {
                BorderedFrame::new(&params.game.ui_theme.panel.border)
                    .margin(outer_margin)
                    .padding(params.game.ui_theme.panel.padding.into())
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        let mut first_button = true;

                        egui::ScrollArea::vertical().show(ui, |ui| {
                            for (section_title, map_handles) in [
                                (
                                    &params.localization.get("default-maps"),
                                    &params.core.stable_maps,
                                ),
                                (
                                    &params.localization.get("experimental-maps"),
                                    &params.core.experimental_maps,
                                ),
                            ] {
                                if map_handles.is_empty() {
                                    continue;
                                }
                                ui.add_space(bigger_text_style.size / 2.0);
                                ui.themed_label(bigger_text_style, section_title);

                                // Clippy lint is a false alarm, necessary to avoid borrowing params
                                #[allow(clippy::unnecessary_to_owned)]
                                for map_handle in map_handles.to_vec().into_iter() {
                                    let map_meta = params
                                        .map_assets
                                        .get(&map_handle.get_bevy_handle())
                                        .expect("Error loading map");
                                    ui.add_space(ui.spacing().item_spacing.y);

                                    let mut button =
                                        BorderedButton::themed(small_button_style, &map_meta.name)
                                            .show(ui);

                                    if first_button {
                                        first_button = false;
                                        button = button.focus_by_default(ui);
                                    }

                                    if button.clicked() {
                                        *params.pause_page = PauseMenuPage::Default;
                                        *params.menu_page = MenuPage::Home;

                                        // TODO: This code to start a game is duplicated 3 or 4
                                        // times throughout this file, which isn't good. We should
                                        // try to abstract it into a function or something.
                                        let mut player_info = <[Option<GameSessionPlayerInfo>;
                                            MAX_PLAYERS]>::default(
                                        );
                                        (0..MAX_PLAYERS).for_each(|i| {
                                            let slot = &params.player_select_state.slots[i];
                                            if slot.active {
                                                player_info[i] = Some(GameSessionPlayerInfo {
                                                    player: slot.selected_player.clone(),
                                                    hat: slot.selected_hat.clone(),
                                                    is_ai: slot.is_ai,
                                                });
                                            }
                                        });
                                        let core_info = CoreSessionInfo {
                                            meta: params.core.0.clone(),
                                            map_meta: map_meta.clone(),
                                            player_info,
                                        };
                                        #[cfg(not(target_arch = "wasm32"))]
                                        if let Some(socket) = &params.network_socket {
                                            info!("Selected map, starting network game");
                                            params.session_manager.start_network(
                                                core_info,
                                                GgrsSessionRunnerInfo {
                                                    socket: socket.ggrs_socket(),
                                                    player_is_local: socket.player_is_local(),
                                                    player_count: socket.player_count(),
                                                },
                                            );
                                        } else {
                                            info!("Selected map, starting game");
                                            params.session_manager.start_local(core_info);
                                        }
                                        #[cfg(target_arch = "wasm32")]
                                        {
                                            info!("Selected map, starting game");
                                            params.session_manager.start_local(core_info);
                                        }

                                        params
                                            .commands
                                            .insert_resource(NextState(Some(EngineState::InGame)));
                                        params
                                            .commands
                                            .insert_resource(NextState(Some(InGameState::Playing)));

                                        #[cfg(not(target_arch = "wasm32"))]
                                        if let Some(socket) = &params.network_socket {
                                            socket.send_reliable(
                                                SocketTarget::All,
                                                &postcard::to_allocvec(
                                                    &MapSelectMessage::SelectMap(map_handle),
                                                )
                                                .unwrap(),
                                            );
                                        }
                                    }
                                }

                                let user_maps: Option<UserMapStorage> =
                                    params.storage.get(UserMapStorage::STORAGE_KEY);
                                if let Some(user_maps) = user_maps {
                                    #[cfg(not(target_arch = "wasm32"))]
                                    let is_network = params.network_socket.is_some();
                                    #[cfg(target_arch = "wasm32")]
                                    let is_network = false;

                                    // For now, network games can only play core maps.
                                    ui.set_enabled(!is_network);
                                    ui.add_space(bigger_text_style.size / 2.0);
                                    ui.themed_label(
                                        bigger_text_style,
                                        &params.localization.get("user-maps"),
                                    );

                                    let mut maps =
                                        user_maps.0.clone().into_iter().collect::<Vec<_>>();
                                    maps.sort_by(|a, b| a.0.cmp(&b.0));

                                    for (name, map_meta) in maps {
                                        ui.add_space(ui.spacing().item_spacing.y);
                                        let button =
                                            BorderedButton::themed(small_button_style, &name)
                                                .show(ui);
                                        if button.clicked() {
                                            *params.pause_page = PauseMenuPage::Default;
                                            *params.menu_page = MenuPage::Home;

                                            let mut player_info = <[Option<GameSessionPlayerInfo>;
                                                MAX_PLAYERS]>::default(
                                            );
                                            (0..MAX_PLAYERS).for_each(|i| {
                                                let slot = &params.player_select_state.slots[i];
                                                if slot.active {
                                                    player_info[i] = Some(GameSessionPlayerInfo {
                                                        player: slot.selected_player.clone(),
                                                        hat: slot.selected_hat.clone(),
                                                        is_ai: slot.is_ai,
                                                    });
                                                }
                                            });
                                            params.session_manager.start_local(CoreSessionInfo {
                                                meta: params.core.0.clone(),
                                                map_meta,
                                                player_info,
                                            });
                                            params.commands.insert_resource(NextState(Some(
                                                EngineState::InGame,
                                            )));
                                            params.commands.insert_resource(NextState(Some(
                                                InGameState::Playing,
                                            )));
                                        };
                                    }
                                }
                            }
                        });
                    });
            }
        });
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn handle_match_setup_messages(params: &mut MapSelectMenu) {
    if let Some(socket) = &params.network_socket {
        let datas: Vec<(usize, Vec<u8>)> = socket.recv_reliable();

        for (player, data) in datas {
            match postcard::from_bytes::<MapSelectMessage>(&data) {
                Ok(message) => match message {
                    MapSelectMessage::SelectMap(map_handle) => {
                        assert_eq!(player, 0, "Only player 0 may select the map.");
                        info!("Other player selected map, starting game");
                        *params.pause_page = PauseMenuPage::Default;
                        *params.menu_page = MenuPage::Home;

                        let map_meta = params
                            .map_assets
                            .get(&map_handle.get_bevy_handle())
                            .unwrap()
                            .clone();

                        let mut player_info =
                            <[Option<GameSessionPlayerInfo>; MAX_PLAYERS]>::default();
                        (0..MAX_PLAYERS).for_each(|i| {
                            let slot = &params.player_select_state.slots[i];
                            if slot.active {
                                player_info[i] = Some(GameSessionPlayerInfo {
                                    player: slot.selected_player.clone(),
                                    hat: slot.selected_hat.clone(),
                                    is_ai: slot.is_ai,
                                });
                            }
                        });
                        params.session_manager.start_network(
                            CoreSessionInfo {
                                meta: params.core.0.clone(),
                                map_meta,
                                player_info,
                            },
                            GgrsSessionRunnerInfo {
                                socket: socket.ggrs_socket(),
                                player_is_local: socket.player_is_local(),
                                player_count: socket.player_count(),
                            },
                        );
                        params
                            .commands
                            .insert_resource(NextState(Some(EngineState::InGame)));
                        params
                            .commands
                            .insert_resource(NextState(Some(InGameState::Playing)));
                    }
                },
                Err(e) => {
                    // TODO: The second player in an online match is having this triggered by
                    // picking up a `ConfirmSelection` message, that might have been sent to
                    // _itself_.
                    warn!("Ignoring network message that was not understood: {e}");
                }
            }
        }
    }
}
