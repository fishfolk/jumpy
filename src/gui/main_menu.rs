use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{self, hash, root_ui, widgets},
};

use fishsticks::GamepadContext;

use super::{GuiResources, Panel};

use crate::{is_gamepad_btn_pressed, EditorInputScheme, GameInputScheme};

const WINDOW_MARGIN: f32 = 22.0;

const MENU_WIDTH: f32 = 340.0;
const MENU_HEIGHT: f32 = 282.0;

const MENU_BUTTON_WIDTH: f32 = MENU_WIDTH - (WINDOW_MARGIN * 2.0);
const MENU_BUTTON_HEIGHT: f32 = 42.0;

const MODE_SELECTION_TAB_COUNT: u32 = 2;

pub enum MainMenuResult {
    LocalGame(Vec<GameInputScheme>),
    Editor {
        input_scheme: EditorInputScheme,
        is_new_map: bool,
    },
    Quit,
}

pub async fn show_main_menu() -> MainMenuResult {
    let mut current_tab = 0;

    let mut player_input = Vec::new();

    loop {
        let mut res = None;

        {
            let mut gamepad_system = storage::get_mut::<GamepadContext>();

            let _ = gamepad_system.update();

            if is_key_pressed(KeyCode::Left)
                || is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::LeftShoulder)
            {
                let next_tab = current_tab as i32 - 1;
                current_tab = if next_tab < 0 {
                    MODE_SELECTION_TAB_COUNT - 1
                } else {
                    next_tab as u32 % MODE_SELECTION_TAB_COUNT
                };
            }

            if is_key_pressed(KeyCode::Right)
                || is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::RightShoulder)
            {
                current_tab += 1;
                current_tab %= MODE_SELECTION_TAB_COUNT;
            }
        }

        let size = vec2(MENU_WIDTH, MENU_HEIGHT);
        let position = (vec2(screen_width(), screen_height() + 70.0) - size) / 2.0;

        {
            let gui_resources = storage::get::<GuiResources>();

            root_ui().push_skin(&gui_resources.skins.menu_header);

            let label = "FISH FIGHT";

            let size = root_ui().calc_size(label);
            let position = vec2((screen_width() - size.x) / 2.0, position.y - 35.0 - size.y);

            widgets::Label::new(label)
                .position(position)
                .ui(&mut root_ui());

            root_ui().pop_skin();

            root_ui().push_skin(&gui_resources.skins.menu);
        }

        Panel::new(hash!(), size, position).ui(&mut root_ui(), |ui| {
            let size = vec2(MENU_BUTTON_WIDTH, MENU_BUTTON_HEIGHT);

            let tab_res = widgets::Tabbar::new(hash!(), size, &["<< LB, Local", "Editor, RB >>"])
                .selected_tab(Some(&mut current_tab))
                .ui(ui);

            match tab_res {
                0 => {
                    res = local_game_ui(ui, &mut player_input);
                }
                1 => {
                    res = editor_ui(ui);
                }
                _ => unreachable!(),
            }

            {
                let position = vec2(
                    0.0,
                    MENU_HEIGHT - (WINDOW_MARGIN * 2.0) - MENU_BUTTON_HEIGHT,
                );
                let size = vec2(MENU_BUTTON_WIDTH, MENU_BUTTON_HEIGHT);

                let is_btn_clicked = widgets::Button::new("Quit")
                    .position(position)
                    .size(size)
                    .ui(ui);

                if is_btn_clicked {
                    res = Some(MainMenuResult::Quit);
                }
            }
        });

        root_ui().pop_skin();

        if let Some(res) = res {
            return res;
        }

        next_frame().await;
    }
}

fn local_game_ui(
    ui: &mut ui::Ui,
    player_input: &mut Vec<GameInputScheme>,
) -> Option<MainMenuResult> {
    let gamepad_system = storage::get_mut::<GamepadContext>();

    let is_ready = player_input.len() == 2;

    if player_input.len() < 2 {
        if is_key_pressed(KeyCode::Enter) {
            if !player_input.contains(&GameInputScheme::KeyboardLeft) {
                player_input.push(GameInputScheme::KeyboardLeft);
            } else {
                player_input.push(GameInputScheme::KeyboardRight);
            }
        }

        for (ix, gamepad) in gamepad_system.gamepads() {
            if gamepad.digital_inputs.activated(fishsticks::Button::Start)
                && !player_input.contains(&GameInputScheme::Gamepad(ix))
            {
                player_input.push(GameInputScheme::Gamepad(ix));
            }
        }
    }

    {
        let position = vec2(12.0, 76.0);

        if !player_input.is_empty() {
            ui.label(position, "Player 1: READY");
        } else {
            ui.label(position, "Player 1: press START or ENTER");
        }
    }

    {
        let position = vec2(12.0, 108.0);

        if player_input.len() > 1 {
            ui.label(position, "Player 2: READY");
        } else {
            ui.label(position, "Player 2: press START or ENTER");
        }
    }

    if is_ready {
        let key_enter = is_key_pressed(KeyCode::Enter);
        let btn_a = is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::A);
        let btn_start = is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::Start);

        let position = vec2(0.0, 150.0);
        let size = vec2(MENU_BUTTON_WIDTH, MENU_BUTTON_HEIGHT);

        let is_btn_clicked = widgets::Button::new("Start Game")
            .position(position)
            .size(size)
            .ui(ui);

        if is_btn_clicked || key_enter || btn_a || btn_start {
            return Some(MainMenuResult::LocalGame(player_input.clone()));
        }
    }

    None
}

fn editor_ui(ui: &mut ui::Ui) -> Option<MainMenuResult> {
    let size = vec2(MENU_BUTTON_WIDTH, MENU_BUTTON_HEIGHT);

    {
        let position = vec2(0.0, 104.0);

        let is_btn_clicked = widgets::Button::new("Create Map")
            .position(position)
            .size(size)
            .ui(ui);

        if is_btn_clicked {
            return Some(MainMenuResult::Editor {
                input_scheme: EditorInputScheme::Keyboard,
                is_new_map: true,
            });
        }
    }

    {
        let position = vec2(0.0, 150.0);

        let is_btn_clicked = widgets::Button::new("Load Map")
            .position(position)
            .size(size)
            .ui(ui);

        if is_btn_clicked {
            return Some(MainMenuResult::Editor {
                input_scheme: EditorInputScheme::Keyboard,
                is_new_map: false,
            });
        }
    }

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
