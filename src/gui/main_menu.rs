use std::borrow::BorrowMut;

use ff_core::prelude::*;

use fishsticks::{Button, GamepadContext};

use ff_core::gui::background::draw_main_menu_background;
use ff_core::gui::{Menu, MenuEntry, MenuResult, Panel, get_gui_theme};

use crate::player::{PlayerControllerKind, PlayerParams};
use crate::{gui, GuiTheme, Map};
use ff_core::input::{is_gamepad_btn_pressed, GameInputScheme};
use ff_core::macroquad::{hash, ui};
use ff_core::macroquad::ui::{root_ui, widgets};
use ff_core::macroquad::window::next_frame;

const MENU_WIDTH: f32 = 300.0;

const HEADER_TEXTURE_ID: &str = "main_menu_header";

const LOCAL_GAME_MENU_WIDTH: f32 = 400.0;
const LOCAL_GAME_MENU_HEIGHT: f32 = 200.0;

pub enum MainMenuResult {
    LocalGame {
        map: Box<Map>,
        players: Vec<PlayerParams>,
    },
    Editor {
        is_new_map: bool,
    },
    ReloadResources,
    Credits,
    Quit,
}

#[allow(dead_code)]
enum MainMenuState {
    Root(Menu),
    LocalGame,
    Settings,
    Editor(Menu),
    Credits,
}

const ROOT_OPTION_LOCAL_GAME: usize = 0;
const ROOT_OPTION_EDITOR: usize = 1;
const ROOT_OPTION_SETTINGS: usize = 2;
const ROOT_OPTION_RELOAD_RESOURCES: usize = 3;
const ROOT_OPTION_CREDITS: usize = 4;

const LOCAL_GAME_OPTION_SUBMIT: usize = 0;

const EDITOR_OPTION_CREATE: usize = 0;
const EDITOR_OPTION_LOAD: usize = 1;

fn build_main_menu() -> Menu {
    Menu::new(
        hash!("main_menu"),
        MENU_WIDTH,
        &[
            MenuEntry {
                index: ROOT_OPTION_LOCAL_GAME,
                title: "Local Game".to_string(),
                ..Default::default()
            },
            MenuEntry {
                index: ROOT_OPTION_EDITOR,
                title: "Editor".to_string(),
                ..Default::default()
            },
            MenuEntry {
                index: ROOT_OPTION_SETTINGS,
                title: "Settings".to_string(),
                is_disabled: true,
                ..Default::default()
            },
            #[cfg(debug_assertions)]
            MenuEntry {
                index: ROOT_OPTION_RELOAD_RESOURCES,
                title: "Reload Resources".to_string(),
                ..Default::default()
            },
            MenuEntry {
                index: ROOT_OPTION_CREDITS,
                title: "Credits".to_string(),
                ..Default::default()
            },
        ],
    )
    .with_cancel_button(Some("Quit"))
}

fn build_editor_menu() -> Menu {
    Menu::new(
        hash!("main_menu", "editor"),
        MENU_WIDTH,
        &[
            MenuEntry {
                index: EDITOR_OPTION_CREATE,
                title: "Create Map".to_string(),
                ..Default::default()
            },
            MenuEntry {
                index: EDITOR_OPTION_LOAD,
                title: "Load Map".to_string(),
                ..Default::default()
            },
        ],
    )
    .with_cancel_button(Some("Cancel"))
}

pub async fn show_main_menu() -> MainMenuResult {
    let mut menu_state = MainMenuState::Root(build_main_menu());

    let mut player_input = Vec::new();

    loop {
        update_gamepad_context().unwrap();

        draw_main_menu_background(true);

        {
            let texture_entry = get_texture(HEADER_TEXTURE_ID);

            let size = texture_entry.texture.size();

            let viewport = get_viewport();
            let position = vec2((viewport.width - size.width) / 2.0, 35.0);

            widgets::Texture::new(texture_entry.texture.into())
                .position(position)
                .size(size.width, size.height)
                .ui(&mut *root_ui());
        }

        match menu_state.borrow_mut() {
            MainMenuState::Root(menu_instance) => {
                if let Some(res) = menu_instance.ui(&mut *root_ui()) {
                    match res.into_usize() {
                        ROOT_OPTION_LOCAL_GAME => {
                            menu_state = MainMenuState::LocalGame;
                        }
                        ROOT_OPTION_EDITOR => {
                            menu_state = MainMenuState::Editor(build_editor_menu());
                        }
                        ROOT_OPTION_RELOAD_RESOURCES => {
                            return MainMenuResult::ReloadResources;
                        }
                        ROOT_OPTION_CREDITS => {
                            menu_state = MainMenuState::Credits;
                        }
                        Menu::CANCEL_INDEX => {
                            return MainMenuResult::Quit;
                        }
                        _ => {}
                    }
                }
            }
            MainMenuState::LocalGame => {
                let res = local_game_ui(&mut *root_ui(), &mut player_input);
                if let Some(res) = res {
                    match res.into_usize() {
                        LOCAL_GAME_OPTION_SUBMIT => {
                            let player_cnt = player_input.len();

                            assert_eq!(
                                player_cnt, 2,
                                "Local Game: There should be two player input schemes for this game mode"
                            );

                            let player_characters =
                                gui::show_select_characters_menu(&player_input).await;

                            let map_resource = gui::show_select_map_menu().await;

                            let mut players = Vec::new();

                            for (i, &input_scheme) in player_input.iter().enumerate() {
                                let character = player_characters.get(i).cloned().unwrap();

                                let controller = PlayerControllerKind::LocalInput(input_scheme);

                                let params = PlayerParams {
                                    index: i as u8,
                                    controller,
                                    character,
                                };

                                players.push(params);
                            }

                            return MainMenuResult::LocalGame {
                                map: Box::new(map_resource.map),
                                players,
                            };
                        }
                        Menu::CANCEL_INDEX => {
                            menu_state = MainMenuState::Root(build_main_menu());
                        }
                        _ => {}
                    }
                }
            }
            MainMenuState::Editor(menu_instance) => {
                if let Some(res) = menu_instance.ui(&mut *root_ui()) {
                    match res.into_usize() {
                        EDITOR_OPTION_CREATE => {
                            return MainMenuResult::Editor {
                                is_new_map: true,
                            }
                        }
                        EDITOR_OPTION_LOAD => {
                            return MainMenuResult::Editor {
                                is_new_map: false,
                            }
                        }
                        Menu::CANCEL_INDEX => {
                            menu_state = MainMenuState::Root(build_main_menu());
                        }
                        _ => {}
                    }
                }
            }
            MainMenuState::Settings => {
                unreachable!("Settings is not implemented yet");
            }
            MainMenuState::Credits => {
                return MainMenuResult::Credits;
            }
        }

        next_frame().await;
    }
}

fn local_game_ui(ui: &mut ui::Ui, player_input: &mut Vec<GameInputScheme>) -> Option<MenuResult> {
    if player_input.len() == 2 {
        return Some(LOCAL_GAME_OPTION_SUBMIT.into());
    } else {
        if is_key_pressed(KeyCode::Escape)
            || is_gamepad_btn_pressed(Button::East)
        {
            return Some(Menu::CANCEL_INDEX.into());
        }
    }

    if player_input.len() < 2 {
        if is_key_pressed(KeyCode::Enter) {
            if !player_input.contains(&GameInputScheme::KeyboardLeft) {
                player_input.push(GameInputScheme::KeyboardLeft);
            } else {
                player_input.push(GameInputScheme::KeyboardRight);
            }
        }

        let gamepad_ctx = get_gamepad_context();
        for (ix, gamepad) in gamepad_ctx.gamepads() {
            if gamepad.digital_inputs.activated(fishsticks::Button::Start)
                && !player_input.contains(&GameInputScheme::Gamepad(ix))
            {
                player_input.push(GameInputScheme::Gamepad(ix));
            }
        }
    }

    let size = vec2(LOCAL_GAME_MENU_WIDTH, LOCAL_GAME_MENU_HEIGHT);

    let viewport = get_viewport();

    let position = (vec2(viewport.width, viewport.height) - size) / 2.0;

    Panel::new(hash!(), size, position).ui(ui, |ui, _| {
        {
            let gui_theme = get_gui_theme();
            ui.push_skin(&gui_theme.menu);
        }

        {
            let position = vec2(12.0, 12.0);

            if !player_input.is_empty() {
                ui.label(position, "Player 1: READY");
            } else {
                ui.label(position, "Player 1: press START or ENTER");
            }
        }

        {
            let position = vec2(12.0, 44.0);

            if player_input.len() > 1 {
                ui.label(position, "Player 2: READY");
            } else {
                ui.label(position, "Player 2: press START or ENTER");
            }
        }

        {
            let position = vec2(12.0, 108.0);

            ui.label(position, "Press B or ESC to cancel");
        }

        ui.pop_skin();
    });

    None
}
