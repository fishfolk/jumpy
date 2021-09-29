use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{self, hash, root_ui, widgets},
};

use crate::{gui::{
    Level,
    GuiResources,
}, input::InputScheme, nodes::network::Message, GameType, EditorInputScheme};

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

fn editor_ui(ui: &mut ui::Ui) -> Option<GameType> {
    let gui_resources = storage::get_mut::<GuiResources>();

    ui.label(None, "Level Editor");

    ui.separator();
    ui.separator();
    ui.separator();

    let mut input_scheme = EditorInputScheme::Keyboard;

    for i in 0..quad_gamepad::MAX_DEVICES {
        let state = gui_resources.gamepads.state(i);

        if state.digital_state[quad_gamepad::GamepadButton::Start as usize] {
            input_scheme = EditorInputScheme::Gamepad(i);
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

    ui.group(hash!(), vec2(WINDOW_WIDTH / 2. - 50., 70.), |ui| {
        match input_scheme {
            EditorInputScheme::Keyboard => {
                ui.label(None, "Gamepad not connected");
            }
            EditorInputScheme::Gamepad(i) => {
                ui.label(None, "Gamepad connected!");
                ui.label(None, &format!("{:?}", i));
            }
        }
    });

    let btn_a = is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::A);
    let btn_b = is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::B);

    if ui.button(None, "Create map (A)") || btn_a {
        return Some(GameType::Editor { input_scheme, is_new_map: true });
    }

    ui.same_line(204.0);

    if ui.button(None, "Load map (B)") || btn_b {
        return Some(GameType::Editor { input_scheme, is_new_map: false });
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
            if socket.recv(&mut buf).is_ok() {
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
                        if socket.recv(&mut buf).is_ok() {
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

    pub fn connect(&mut self) -> bool {
        let socket = self.socket.as_mut().unwrap();
        match self.kind {
            ConnectionKind::Lan | ConnectionKind::Stun => {
                socket.connect(&self.opponent_addr).unwrap();
            }
            ConnectionKind::Relay => {
                let other_id;
                match self.opponent_addr.parse::<u64>() {
                    Ok(v) => other_id = v,
                    Err(_) => return false,
                };
                loop {
                    let _ = socket.send(&nanoserde::SerBin::serialize_bin(
                        &Message::RelayConnectTo(other_id),
                    ));

                    let mut buf = [0; 100];
                    if socket.recv(&mut buf).is_ok() {
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
        true
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

    false
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
        && state.connection.connect()
    {
        return Some(GameType::Network {
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

const MODE_SELECTION_TAB_COUNT: u32 = 3;

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
                let next_tab = tab as i32 - 1;
                tab = if next_tab < 0 {
                    MODE_SELECTION_TAB_COUNT
                } else {
                    next_tab as u32 % MODE_SELECTION_TAB_COUNT
                };
            }

            if is_key_pressed(KeyCode::Right)
                || is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::BumperRight)
                || is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::ThumbRight)
            {
                tab += 1;
                tab %= MODE_SELECTION_TAB_COUNT;
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
                &["<< LT, Local", "Editor", "Network, RT >>"],
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
                2 => {
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

pub async fn location_select() -> Level {
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
                    let level = levels.get(hovered as usize).unwrap();
                    return level.clone();
                }
            }
        }

        root_ui().pop_skin();

        old_mouse_position = mouse_position();

        next_frame().await;
    }
}

pub async fn new_map() -> (String, Vec2, UVec2) {
    let mut res = None;

    let size = vec2(350.0, 230.0);
    let position = vec2(
        (screen_width() - size.x) / 2.0,
        (screen_height() - size.y) / 2.0,
    );

    next_frame().await;

    let mut gui_resources = storage::get_mut::<GuiResources>();
    root_ui().push_skin(&gui_resources.skins.login_skin);

    let mut map_name = "Unnamed Map".to_string();
    let mut map_grid_width = "100".to_string();
    let mut map_grid_height = "100".to_string();
    let mut map_tile_width  = "32".to_string();
    let mut map_tile_height  = "32".to_string();

    loop {
        gui_resources.gamepads.update();

        clear_background(BLACK);

        widgets::Window::new(hash!(), position, size).titlebar(false).movable(false).ui(&mut *root_ui(), |ui| {
            ui.label(None, "Create new map");

            ui.separator();
            ui.separator();
            ui.separator();
            ui.separator();

            {
                let size = vec2(173.0, 25.0);

                widgets::InputText::new(hash!())
                    .size(size)
                    .ratio(1.0)
                    .label("Map name")
                    .ui(ui, &mut map_name);
            }

            ui.separator();
            ui.separator();

            {
                let size = vec2(75.0, 25.0);

                widgets::InputText::new(hash!())
                    .size(size)
                    .ratio(1.0)
                    .label("x")
                    .ui(ui, &mut map_grid_width);

                ui.same_line(size.x + 25.0);

                widgets::InputText::new(hash!())
                    .size(size)
                    .ratio(1.0)
                    .label("Grid size")
                    .ui(ui, &mut map_grid_height);

                widgets::InputText::new(hash!())
                    .size(size)
                    .ratio(1.0)
                    .label("x")
                    .ui(ui, &mut map_tile_width);

                ui.same_line(size.x + 25.0);

                widgets::InputText::new(hash!())
                    .size(size)
                    .ratio(1.0)
                    .label("Tile size")
                    .ui(ui, &mut map_tile_height);
            }

            ui.separator();
            ui.separator();
            ui.separator();
            ui.separator();

            let btn_a = is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::A);
            let enter = is_key_pressed(KeyCode::Enter);

            if ui.button(None, "Confirm (A) (Enter)") || btn_a || enter {
                // TODO: Validate input

                let grid_size = uvec2(
                    map_grid_width.parse::<u32>().unwrap(),
                    map_grid_height.parse::<u32>().unwrap(),
                );

                let tile_size = vec2(
                    map_tile_width.parse::<f32>().unwrap(),
                    map_tile_height.parse::<f32>().unwrap(),
                );

                let map_params = (map_name.clone(), tile_size, grid_size);
                res = Some(map_params);
            }
        });

        if let Some(res) = res {
            root_ui().pop_skin();
            return res;
        }

        next_frame().await;
    }
}
