use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{self, hash, root_ui, widgets},
};
use std::borrow::BorrowMut;

use fishsticks::{Button, GamepadContext};

use super::{draw_main_menu_background, GuiResources, Menu, MenuEntry, MenuResult, Panel};

use crate::input::update_gamepad_context;
use crate::{is_gamepad_btn_pressed, EditorInputScheme, GameInputScheme, Resources};

const MENU_WIDTH: f32 = 300.0;

const HEADER_TEXTURE_ID: &str = "main_menu_header";

const LOCAL_GAME_MENU_WIDTH: f32 = 400.0;
const LOCAL_GAME_MENU_HEIGHT: f32 = 200.0;

pub enum MainMenuResult {
    LocalGame(Vec<GameInputScheme>),
    Editor {
        input_scheme: EditorInputScheme,
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
    NetworkGame,
    Settings,
    Editor(Menu),
    Credits,
}

const ROOT_OPTION_LOCAL_GAME: usize = 0;
const ROOT_OPTION_NETWORK_GAME: usize = 1;
const ROOT_OPTION_EDITOR: usize = 2;
const ROOT_OPTION_SETTINGS: usize = 3;
const ROOT_OPTION_RELOAD_RESOURCES: usize = 4;
const ROOT_OPTION_CREDITS: usize = 5;

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
                index: ROOT_OPTION_NETWORK_GAME,
                title: "Network Game".to_string(),
                is_disabled: true,
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
        update_gamepad_context(None).unwrap();

        draw_main_menu_background(true);

        {
            let resources = storage::get::<Resources>();
            let texture_entry = resources.textures.get(HEADER_TEXTURE_ID).unwrap();

            let size = vec2(
                texture_entry.texture.width(),
                texture_entry.texture.height(),
            );

            let position = vec2((screen_width() - size.x) / 2.0, 35.0);

            widgets::Texture::new(texture_entry.texture)
                .position(position)
                .size(size.x, size.y)
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
                if let Some(res) = local_game_ui(&mut *root_ui(), &mut player_input) {
                    match res.into_usize() {
                        LOCAL_GAME_OPTION_SUBMIT => {
                            return MainMenuResult::LocalGame(player_input.clone());
                        }
                        Menu::CANCEL_INDEX => {
                            menu_state = MainMenuState::Root(build_main_menu());
                        }
                        _ => {}
                    }
                }
            }
            MainMenuState::NetworkGame => {
                unreachable!("Networking is not implemented yet");
            }
            MainMenuState::Editor(menu_instance) => {
                if let Some(res) = menu_instance.ui(&mut *root_ui()) {
                    match res.into_usize() {
                        EDITOR_OPTION_CREATE => {
                            return MainMenuResult::Editor {
                                input_scheme: EditorInputScheme::Mouse,
                                is_new_map: true,
                            }
                        }
                        EDITOR_OPTION_LOAD => {
                            return MainMenuResult::Editor {
                                input_scheme: EditorInputScheme::Mouse,
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
        let gamepad_context = storage::get::<GamepadContext>();
        if is_key_pressed(KeyCode::Escape)
            || is_gamepad_btn_pressed(Some(&gamepad_context), Button::B)
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

        let gamepad_context = storage::get_mut::<GamepadContext>();
        for (ix, gamepad) in gamepad_context.gamepads() {
            if gamepad.digital_inputs.activated(fishsticks::Button::Start)
                && !player_input.contains(&GameInputScheme::Gamepad(ix))
            {
                player_input.push(GameInputScheme::Gamepad(ix));
            }
        }
    }

    let size = vec2(LOCAL_GAME_MENU_WIDTH, LOCAL_GAME_MENU_HEIGHT);
    let position = (vec2(screen_width(), screen_height()) - size) / 2.0;

    Panel::new(hash!(), size, position).ui(ui, |ui, _| {
        {
            let gui_resources = storage::get::<GuiResources>();
            ui.push_skin(&gui_resources.skins.menu);
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

/*
struct NetworkUiState {
    input_scheme: GameInputScheme,
    connection_kind: NetworkConnectionKind,
    connection: NetworkConnection,
    custom_relay: bool,
}

fn network_game_ui(ui: &mut ui::Ui, state: &mut NetworkUiState) -> Option<MainMenuResult> {
    let mut connection_kind_ui = state.connection_kind as usize;

    widgets::ComboBox::new(hash!(), &["Lan network", "STUN server", "Relay server"])
        .ratio(0.4)
        .label("Connection type")
        .ui(ui, &mut connection_kind_ui);

    match connection_kind_ui {
        x if x == NetworkConnectionKind::Stun as usize => {
            state.connection_kind = NetworkConnectionKind::Stun;
        }
        x if x == NetworkConnectionKind::Lan as usize => {
            state.connection_kind = NetworkConnectionKind::Lan;
        }
        x if x == NetworkConnectionKind::Relay as usize => {
            state.connection_kind = NetworkConnectionKind::Relay;
        }
        _ => unreachable!(),
    }

    if state.connection_kind == NetworkConnectionKind::Relay {
        widgets::Checkbox::new(hash!())
            .label("Use custom relay server")
            .ratio(0.4)
            .ui(ui, &mut state.custom_relay);

        if state.custom_relay {
            widgets::InputText::new(hash!())
                .ratio(0.4)
                .label("Self addr")
                .ui(ui, &mut state.connection.relay_addr);
        }
    }

    let mut self_addr = state.connection.local_addr.clone();
    widgets::InputText::new(hash!())
        .ratio(0.4)
        .label("Self addr")
        .ui(ui, &mut self_addr);

    widgets::InputText::new(hash!())
        .ratio(0.4)
        .label("Opponent addr")
        .ui(ui, &mut state.connection.opponent_addr);

    state.connection.update(state.connection_kind);

    if ui.button(None, "Probe connection") {
        state.connection.probe();
    }

    ui.label(
        None,
        &format!("Connection status: {:?}", state.connection.status),
    );

    if state.connection.status == NetworkConnectionStatus::Connected
        && ui.button(None, "Connect (A) (Enter)")
        && state.connection.connect()
    {
        return Some(MainMenuResult::NetworkGame {
            socket: state.connection.socket.take().unwrap(),
            id: if state.connection.local_addr > state.connection.opponent_addr {
                0
            } else {
                1
            },
            input_scheme: state.input_scheme,
        });
    }

    ui.label(
        vec2(430., 310.),
        &format!("Input: {:?}", state.input_scheme),
    );

    ui.label(vec2(360., 330.), "Press V/L/Start to change");
    if is_key_pressed(KeyCode::V) {
        state.input_scheme = GameInputScheme::KeyboardLeft;
    }

    if is_key_pressed(KeyCode::L) {
        state.input_scheme = GameInputScheme::KeyboardRight;
    }

    let gamepad_system = storage::get_mut::<GamepadContext>();
    for (ix, gamepad) in gamepad_system.gamepads() {
        if gamepad.digital_inputs.activated(fishsticks::Button::Start) {
            state.input_scheme = GameInputScheme::Gamepad(ix);
        }
    }

    None
}
*/
