use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{self, hash, root_ui, widgets},
};

use fishsticks::GamepadContext;

use crate::{
    game::{NetworkConnection, NetworkConnectionKind, NetworkConnectionStatus},
    gui::GuiResources,
    is_gamepad_btn_pressed, EditorInputScheme, GameInputScheme,
};

const MAIN_MENU_WIDTH: f32 = 700.0;
const MAIN_MENU_HEIGHT: f32 = 400.0;

#[allow(dead_code)]
pub enum MainMenuResult {
    Editor {
        input_scheme: EditorInputScheme,
        is_new_map: bool,
    },
    LocalGame(Vec<GameInputScheme>),
    NetworkGame {
        socket: std::net::UdpSocket,
        id: usize,
        input_scheme: GameInputScheme,
    },
}

const MODE_SELECTION_TAB_COUNT: u32 = 2;

pub async fn show_main_menu() -> MainMenuResult {
    let mut players = vec![];

    let mut tab = 0;
    loop {
        let mut res = None;

        {
            let mut gamepad_system = storage::get_mut::<GamepadContext>();

            let _ = gamepad_system.update();

            if is_key_pressed(KeyCode::Left)
                || is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::LeftShoulder)
                || is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::LeftStick)
            {
                let next_tab = tab as i32 - 1;
                tab = if next_tab < 0 {
                    MODE_SELECTION_TAB_COUNT - 1
                } else {
                    next_tab as u32 % MODE_SELECTION_TAB_COUNT
                };
            }

            if is_key_pressed(KeyCode::Right)
                || is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::RightShoulder)
                || is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::RightStick)
            {
                tab += 1;
                tab %= MODE_SELECTION_TAB_COUNT;
            }
        }

        {
            let gui_resources = storage::get::<GuiResources>();
            root_ui().push_skin(&gui_resources.skins.menu);
        }

        root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - MAIN_MENU_WIDTH / 2.,
                screen_height() / 2. - MAIN_MENU_HEIGHT / 2.,
            ),
            Vec2::new(MAIN_MENU_WIDTH, MAIN_MENU_HEIGHT),
            |ui| match widgets::Tabbar::new(
                hash!(),
                vec2(MAIN_MENU_WIDTH - 50., 50.),
                &["<< LB, Local", "Editor, RB >>"],
            )
            .selected_tab(Some(&mut tab))
            .ui(ui)
            {
                0 => {
                    res = local_game_ui(ui, &mut players);
                }
                1 => {
                    res = editor_ui(ui);
                }
                _ => unreachable!(),
            },
        );

        root_ui().pop_skin();

        if let Some(res) = res {
            return res;
        }
        next_frame().await;
    }
}

fn local_game_ui(ui: &mut ui::Ui, players: &mut Vec<GameInputScheme>) -> Option<MainMenuResult> {
    let gamepad_system = storage::get_mut::<GamepadContext>();

    if players.len() < 2 {
        if is_key_pressed(KeyCode::V) {
            //
            if !players.contains(&GameInputScheme::KeyboardLeft) {
                players.push(GameInputScheme::KeyboardLeft);
            }
        }
        if is_key_pressed(KeyCode::L) {
            //
            if !players.contains(&GameInputScheme::KeyboardRight) {
                players.push(GameInputScheme::KeyboardRight);
            }
        }
        for (ix, gamepad) in gamepad_system.gamepads() {
            if gamepad.digital_inputs.activated(fishsticks::Button::Start)
                && !players.contains(&GameInputScheme::Gamepad(ix))
            {
                players.push(GameInputScheme::Gamepad(ix));
            }
        }
    }

    ui.label(None, "To connect:");
    ui.label(None, "Press Start on gamepad");
    ui.separator();

    ui.label(None, "Or V for keyboard 1");
    ui.label(None, "Or L for keyboard 2");

    ui.separator();
    ui.separator();
    ui.separator();
    ui.separator();

    ui.group(hash!(), vec2(MAIN_MENU_WIDTH / 2. - 50., 70.), |ui| {
        if players.get(0).is_none() {
            ui.label(None, "Player 1: Not connected");
        }
        if let Some(input) = players.get(0) {
            ui.label(None, "Player 1: Connected!");
            ui.label(None, &format!("{:?}", input));
        }
    });
    ui.group(hash!(), vec2(MAIN_MENU_WIDTH / 2. - 50., 70.), |ui| {
        if players.get(1).is_none() {
            ui.label(None, "Player 2: Not connected");
        }
        if let Some(input) = players.get(1) {
            ui.label(None, "Player 2: Connected!");
            ui.label(None, &format!("{:?}", input));
        }
    });
    if players.len() == 2 {
        let btn_a = is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::A);
        let enter = is_key_pressed(KeyCode::Enter);

        if ui.button(None, "Ready! (A) (Enter)") || btn_a || enter {
            return Some(MainMenuResult::LocalGame(players.clone()));
        }
    }

    None
}

fn editor_ui(ui: &mut ui::Ui) -> Option<MainMenuResult> {
    let gamepad_system = storage::get_mut::<GamepadContext>();

    ui.label(None, "Level Editor");

    ui.separator();
    ui.separator();
    ui.separator();

    let mut input_scheme = EditorInputScheme::Keyboard;

    for (ix, gamepad) in gamepad_system.gamepads() {
        if gamepad.digital_inputs.activated(fishsticks::Button::Start) {
            input_scheme = EditorInputScheme::Gamepad(ix);
        }
    }

    ui.label(None, "To connect:");
    ui.label(None, "Press Start on gamepad");

    ui.separator();

    ui.label(None, "Or proceed using keyboard and mouse...");

    ui.separator();
    ui.separator();
    ui.separator();
    ui.separator();

    ui.group(
        hash!(),
        vec2(MAIN_MENU_WIDTH / 2. - 50., 70.),
        |ui| match input_scheme {
            EditorInputScheme::Keyboard => {
                ui.label(None, "Gamepad not connected");
            }
            EditorInputScheme::Gamepad(i) => {
                ui.label(None, "Gamepad connected!");
                ui.label(None, &format!("{:?}", i));
            }
        },
    );

    let btn_a = is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::A);
    let btn_b = is_gamepad_btn_pressed(&gamepad_system, fishsticks::Button::B);

    if ui.button(None, "Create map (A)") || btn_a {
        return Some(MainMenuResult::Editor {
            input_scheme,
            is_new_map: true,
        });
    }

    ui.same_line(204.0);

    if ui.button(None, "Load map (B)") || btn_b {
        return Some(MainMenuResult::Editor {
            input_scheme,
            is_new_map: false,
        });
    }

    None
}

#[allow(dead_code)]
struct NetworkUiState {
    input_scheme: GameInputScheme,
    connection_kind: NetworkConnectionKind,
    connection: NetworkConnection,
    custom_relay: bool,
}

#[allow(dead_code)]
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
