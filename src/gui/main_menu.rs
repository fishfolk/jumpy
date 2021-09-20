use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{self, hash, root_ui, widgets},
};

use crate::{gui::GuiResources, input::InputScheme, nodes::network::Message, GameType};

use std::net::UdpSocket;

const RELAY_ADDR: &str = "173.0.157.169:35000";
const WINDOW_WIDTH: f32 = 700.;
const WINDOW_HEIGHT: f32 = 400.;

fn local_game_ui(ui: &mut ui::Ui, players: &mut Vec<InputScheme>) -> Option<GameType> {
    let gui_resources = storage::get_mut::<GuiResources>();

    if players.len() < 2 {
        if is_key_pressed(KeyCode::V) {
            //
            if !players.contains(&InputScheme::KeyboardLeft) {
                players.push(InputScheme::KeyboardLeft);
            }
        }
        if is_key_pressed(KeyCode::L) {
            //
            if !players.contains(&InputScheme::KeyboardRight) {
                players.push(InputScheme::KeyboardRight);
            }
        }
        for ix in 0..quad_gamepad::MAX_DEVICES {
            let state = gui_resources.gamepads.state(ix);

            if state.digital_state[quad_gamepad::GamepadButton::Start as usize] {
                //
                if !players.contains(&InputScheme::Gamepad(ix)) {
                    players.push(InputScheme::Gamepad(ix));
                }
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

    ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 50., 70.), |ui| {
        if players.get(0).is_none() {
            ui.label(None, "Player 1: Not connected");
        }
        if let Some(input) = players.get(0) {
            ui.label(None, "Player 1: Connected!");
            ui.label(None, &format!("{:?}", input));
        }
    });
    ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 50., 70.), |ui| {
        if players.get(1).is_none() {
            ui.label(None, "Player 2: Not connected");
        }
        if let Some(input) = players.get(1) {
            ui.label(None, "Player 2: Connected!");
            ui.label(None, &format!("{:?}", input));
        }
    });
    if players.len() == 2 {
        let btn_a = is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::A);
        let enter = is_key_pressed(KeyCode::Enter);

        if ui.button(None, "Ready! (A) (Enter)") || btn_a || enter {
            return Some(GameType::Local(players.clone()));
        }
    }

    None
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum ConnectionKind {
    Lan,
    Stun,
    Relay,
    Unknown,
}

#[derive(Debug, PartialEq)]
enum ConnectionStatus {
    Unknown,
    Connected,
}

struct Connection {
    kind: ConnectionKind,
    socket: Option<UdpSocket>,
    local_addr: String,
    opponent_addr: String,
    relay_addr: String,
    status: ConnectionStatus,
}

impl Connection {
    fn new() -> Connection {
        Connection {
            kind: ConnectionKind::Unknown,
            socket: None,
            local_addr: "".to_string(),
            opponent_addr: "".to_string(),
            relay_addr: RELAY_ADDR.to_string(),
            status: ConnectionStatus::Unknown,
        }
    }

    fn update(&mut self, kind: ConnectionKind) {
        if let Some(socket) = self.socket.as_mut() {
            let mut buf = [0; 100];
            if let Ok(_) = socket.recv(&mut buf) {
                let _message: Message = nanoserde::DeBin::deserialize_bin(&buf[..]).ok().unwrap();
                self.status = ConnectionStatus::Connected;
            }
        }

        if kind != self.kind {
            self.kind = kind;
            self.status = ConnectionStatus::Unknown;

            use std::net::SocketAddr;

            let addrs = [
                SocketAddr::from(([0, 0, 0, 0], 3400)),
                SocketAddr::from(([0, 0, 0, 0], 3401)),
                SocketAddr::from(([0, 0, 0, 0], 3402)),
                SocketAddr::from(([0, 0, 0, 0], 3403)),
            ];
            match kind {
                ConnectionKind::Lan => {
                    let socket = UdpSocket::bind(&addrs[..]).unwrap();

                    self.local_addr = format!("{}", socket.local_addr().unwrap());
                    socket.set_nonblocking(true).unwrap();

                    self.socket = Some(socket);
                }
                ConnectionKind::Stun => {
                    let socket = UdpSocket::bind(&addrs[..]).unwrap();

                    let sc = stunclient::StunClient::with_google_stun_server();
                    self.local_addr = format!("{}", sc.query_external_address(&socket).unwrap());
                    socket.set_nonblocking(true).unwrap();

                    self.socket = Some(socket);
                }
                ConnectionKind::Relay => {
                    let socket = UdpSocket::bind(&addrs[..]).unwrap();
                    socket.connect(&self.relay_addr).unwrap();
                    socket.set_nonblocking(true).unwrap();

                    loop {
                        let _ = socket
                            .send(&nanoserde::SerBin::serialize_bin(&Message::RelayRequestId));

                        let mut buf = [0; 100];
                        if let Ok(_) = socket.recv(&mut buf) {
                            let message: Message =
                                nanoserde::DeBin::deserialize_bin(&buf[..]).ok().unwrap();
                            if let Message::RelayIdAssigned(id) = message {
                                self.local_addr = format!("{}", id);
                                break;
                            }
                        }
                    }
                    self.socket = Some(socket);
                }
                _ => {}
            }
        }
    }

    pub fn connect(&mut self) {
        let socket = self.socket.as_mut().unwrap();
        match self.kind {
            ConnectionKind::Lan | ConnectionKind::Stun => {
                socket.connect(&self.opponent_addr).unwrap();
            }
            ConnectionKind::Relay => {
                let other_id = self.opponent_addr.parse::<u64>().unwrap();
                loop {
                    let _ = socket.send(&nanoserde::SerBin::serialize_bin(
                        &Message::RelayConnectTo(other_id),
                    ));

                    let mut buf = [0; 100];
                    if let Ok(_) = socket.recv(&mut buf) {
                        let message: Message =
                            nanoserde::DeBin::deserialize_bin(&buf[..]).ok().unwrap();
                        if let Message::RelayConnected = message {
                            break;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    pub fn probe(&mut self) -> Option<()> {
        assert!(self.socket.is_some());

        if self.kind == ConnectionKind::Relay {
            return Some(());
        }
        let socket = self.socket.as_mut().unwrap();

        socket.connect(&self.opponent_addr).ok()?;

        for _ in 0..100 {
            socket
                .send(&nanoserde::SerBin::serialize_bin(&Message::Idle))
                .ok()?;
        }

        None
    }
}
struct NetworkUiState {
    input_scheme: InputScheme,
    connection_kind: ConnectionKind,
    connection: Connection,
    custom_relay: bool,
}

fn is_gamepad_btn_pressed(gui_resources: &GuiResources, btn: quad_gamepad::GamepadButton) -> bool {
    for ix in 0..quad_gamepad::MAX_DEVICES {
        let state = gui_resources.gamepads.state(ix);
        if state.digital_state[btn as usize] && !state.digital_state_prev[btn as usize] {
            return true;
        }
    }

    return false;
}

fn network_game_ui(ui: &mut ui::Ui, state: &mut NetworkUiState) -> Option<GameType> {
    let mut connection_kind_ui = state.connection_kind as usize;
    widgets::ComboBox::new(hash!(), &["Lan network", "STUN server", "Relay server"])
        .ratio(0.4)
        .label("Connection type")
        .ui(ui, &mut connection_kind_ui);
    match connection_kind_ui {
        x if x == ConnectionKind::Stun as usize => {
            state.connection_kind = ConnectionKind::Stun;
        }
        x if x == ConnectionKind::Lan as usize => {
            state.connection_kind = ConnectionKind::Lan;
        }
        x if x == ConnectionKind::Relay as usize => {
            state.connection_kind = ConnectionKind::Relay;
        }
        _ => unreachable!(),
    }

    if state.connection_kind == ConnectionKind::Relay {
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

    if state.connection.status == ConnectionStatus::Connected
        && ui.button(None, "Connect (A) (Enter)")
    {
        state.connection.connect();

        return Some(GameType::Network {
            socket: state.connection.socket.take().unwrap(),
            id: if state.connection.local_addr > state.connection.opponent_addr {
                0
            } else {
                1
            },
            input_scheme: state.input_scheme.clone(),
        });
    }

    ui.label(
        vec2(430., 310.),
        &format!("Input: {:?}", state.input_scheme),
    );
    ui.label(vec2(360., 330.), "Press V/L/Start to change");
    if is_key_pressed(KeyCode::V) {
        state.input_scheme = InputScheme::KeyboardLeft;
    }
    if is_key_pressed(KeyCode::L) {
        state.input_scheme = InputScheme::KeyboardRight;
    }
    for ix in 0..quad_gamepad::MAX_DEVICES {
        let gui_resources = storage::get_mut::<GuiResources>();
        let gamepad_state = gui_resources.gamepads.state(ix);

        if gamepad_state.digital_state[quad_gamepad::GamepadButton::Start as usize] {
            state.input_scheme = InputScheme::Gamepad(ix);
        }
    }

    None
}

pub async fn game_type() -> GameType {
    let mut players = vec![];

    let mut network_ui_state = NetworkUiState {
        connection: Connection::new(),
        input_scheme: InputScheme::KeyboardLeft,
        connection_kind: ConnectionKind::Lan,
        custom_relay: false,
    };

    let mut tab = 0;
    loop {
        let mut res = None;

        {
            let mut gui_resources = storage::get_mut::<GuiResources>();

            gui_resources.gamepads.update();

            if is_key_pressed(KeyCode::Left)
                || is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::BumperLeft)
                || is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::ThumbLeft)
            {
                tab += 1;
                tab %= 2;
            }
            // for two tabs going left and right is the same thing
            if is_key_pressed(KeyCode::Right)
                || is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::BumperRight)
                || is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::ThumbRight)
            {
                tab += 1;
                tab %= 2;
            }
        }

        {
            let gui_resources = storage::get_mut::<GuiResources>();
            root_ui().push_skin(&gui_resources.skins.login_skin);
        }

        root_ui().window(
            hash!(),
            Vec2::new(
                screen_width() / 2. - WINDOW_WIDTH / 2.,
                screen_height() / 2. - WINDOW_HEIGHT / 2.,
            ),
            Vec2::new(WINDOW_WIDTH, WINDOW_HEIGHT),
            |ui| match widgets::Tabbar::new(
                hash!(),
                vec2(WINDOW_WIDTH - 50., 50.),
                &["<< Local game, LT", "Network game, RT >>"],
            )
            .selected_tab(Some(&mut tab))
            .ui(ui)
            {
                0 => {
                    res = local_game_ui(ui, &mut players);
                }
                1 => {
                    res = network_game_ui(ui, &mut network_ui_state);
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

pub async fn location_select() -> String {
    let mut hovered: i32 = 0;

    let mut old_mouse_position = mouse_position();

    // skip a frame to let Enter be unpressed from the previous screen
    next_frame().await;

    let mut prev_up = false;
    let mut prev_down = false;
    let mut prev_right = false;
    let mut prev_left = false;

    loop {
        let mut gui_resources = storage::get_mut::<GuiResources>();

        gui_resources.gamepads.update();

        let mut up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
        let mut down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        let mut right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
        let mut left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        let mut start = is_key_pressed(KeyCode::Enter);

        for ix in 0..quad_gamepad::MAX_DEVICES {
            use quad_gamepad::GamepadButton::*;

            let state = gui_resources.gamepads.state(ix);
            if state.status == quad_gamepad::ControllerStatus::Connected {
                up |= !prev_up && state.analog_state[1] < -0.5;
                down |= !prev_down && state.analog_state[1] > 0.5;
                left |= !prev_left && state.analog_state[0] < -0.5;
                right |= !prev_right && state.analog_state[0] > 0.5;
                start |= (state.digital_state[A as usize] && !state.digital_state_prev[A as usize])
                    || (state.digital_state[Start as usize]
                        && !state.digital_state_prev[Start as usize]);

                prev_up = state.analog_state[1] < -0.5;
                prev_down = state.analog_state[1] > 0.5;
                prev_left = state.analog_state[0] < -0.5;
                prev_right = state.analog_state[0] > 0.5;
            }
        }
        clear_background(BLACK);

        let levels_amount = gui_resources.levels.len();

        root_ui().push_skin(&gui_resources.skins.main_menu_skin);

        let rows = (levels_amount + 2) / 3;
        let w = (screen_width() - 120.) / 3. - 50.;
        let h = (screen_height() - 180.) / rows as f32 - 50.;

        {
            if up {
                hovered -= 3;
                let ceiled_levels_amount = levels_amount as i32 + 3 - (levels_amount % 3) as i32;
                if hovered < 0 {
                    hovered = (hovered + ceiled_levels_amount as i32) % ceiled_levels_amount;
                    if hovered >= levels_amount as i32 {
                        hovered -= 3;
                    }
                }
            }

            if down {
                hovered += 3;
                if hovered >= levels_amount as i32 {
                    let row = hovered % 3;
                    hovered = row;
                }
            }
            if left {
                hovered -= 1;
            }
            if right {
                hovered += 1;
            }
            hovered = (hovered + levels_amount as i32) % levels_amount as i32;

            let levels = &mut gui_resources.levels;

            for (n, level) in levels.iter_mut().enumerate() {
                let is_hovered = hovered == n as i32;

                let rect = Rect::new(
                    60. + (n % 3) as f32 * (w + 50.) - level.size * 30.,
                    90. + 25. + (n / 3) as f32 * (h + 50.) - level.size * 30.,
                    w + level.size * 60.,
                    h + level.size * 60.,
                );
                if old_mouse_position != mouse_position() && rect.contains(mouse_position().into())
                {
                    hovered = n as _;
                }

                if is_hovered {
                    level.size = level.size * 0.8 + 1.0 * 0.2;
                } else {
                    level.size = level.size * 0.9 + 0.0;
                }

                if ui::widgets::Button::new(level.preview)
                    .size(rect.size())
                    .position(rect.point())
                    .ui(&mut *root_ui())
                    || start
                {
                    root_ui().pop_skin();
                    let level = &levels[hovered as usize];
                    return level.map.clone();
                }
            }
        }

        root_ui().pop_skin();

        old_mouse_position = mouse_position();

        next_frame().await;
    }
}
