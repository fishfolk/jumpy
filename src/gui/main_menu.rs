use std::path::Path;

use std::net::UdpSocket;

use macroquad::{
    experimental::collections::storage,
    prelude::*,
    ui::{self, hash, root_ui, widgets},
};

use crate::{
    error::Result,
    gui::GuiResources,
    input::InputScheme,
    nodes::network::Message,
    resources::{map_name_to_filename, MapResource},
    text::{draw_aligned_text, HorizontalAlignment, VerticalAlignment},
    EditorInputScheme, GameType, Resources,
};

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

    ui.group(
        hash!(),
        vec2(WINDOW_WIDTH / 2. - 50., 70.),
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

    let btn_a = is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::A);
    let btn_b = is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::B);

    if ui.button(None, "Create map (A)") || btn_a {
        return Some(GameType::Editor {
            input_scheme,
            is_new_map: true,
        });
    }

    ui.same_line(204.0);

    if ui.button(None, "Load map (B)") || btn_b {
        return Some(GameType::Editor {
            input_scheme,
            is_new_map: false,
        });
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
                    MODE_SELECTION_TAB_COUNT - 1
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
                &["<< LB, Local", "Editor", "Network, RB >>"],
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

const MAP_SELECT_SCREEN_MARGIN_FACTOR: f32 = 0.1;
const MAP_SELECT_PREVIEW_TARGET_WIDTH: f32 = 250.0;
const MAP_SELECT_PREVIEW_RATIO: f32 = 10.0 / 16.0;
const MAP_SELECT_PREVIEW_SHRINK_FACTOR: f32 = 0.8;

pub async fn location_select() -> MapResource {
    let mut current_page: i32;
    let mut hovered: i32 = 0;

    let mut previous_mouse_pos = mouse_position();

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

        let (_, mouse_wheel) = mouse_wheel();
        let page_up = mouse_wheel < 0.0;
        let page_down = mouse_wheel > 0.0;

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

        let resources = storage::get::<Resources>();
        let map_cnt = resources.maps.len();

        root_ui().push_skin(&gui_resources.skins.main_menu_skin);

        let screen_size = vec2(screen_width(), screen_height());
        let screen_margins = vec2(
            screen_size.x * MAP_SELECT_SCREEN_MARGIN_FACTOR,
            screen_size.y * MAP_SELECT_SCREEN_MARGIN_FACTOR,
        );
        let content_size = vec2(
            screen_size.x - (screen_margins.x * 2.0),
            screen_size.y - (screen_margins.y * 2.0),
        );

        let entries_per_row = (content_size.x / MAP_SELECT_PREVIEW_TARGET_WIDTH).round() as usize;
        let row_cnt = (map_cnt / entries_per_row) + 1;

        let entry_size = {
            let width = content_size.x / entries_per_row as f32;
            vec2(width, width * MAP_SELECT_PREVIEW_RATIO)
        };

        let rows_per_page = (content_size.y / entry_size.y) as usize;
        let entries_per_page = rows_per_page * entries_per_row;

        let page_cnt = (row_cnt / rows_per_page) + 1;

        {
            if up {
                hovered -= entries_per_row as i32;
                if hovered < 0 {
                    hovered += 1 + map_cnt as i32 + (map_cnt % entries_per_row) as i32;
                    if hovered >= map_cnt as i32 {
                        hovered = map_cnt as i32 - 1;
                    }
                }
            }

            if down {
                let old = hovered;
                hovered += entries_per_row as i32;
                if hovered >= map_cnt as i32 {
                    if old == map_cnt as i32 - 1 {
                        hovered = 0;
                    } else {
                        hovered = map_cnt as i32 - 1;
                    }
                }
            }

            if left {
                let row_begin = (hovered / entries_per_row as i32) * entries_per_row as i32;
                hovered -= 1;
                if hovered < row_begin {
                    hovered = row_begin + entries_per_row as i32 - 1;
                }
            }

            if right {
                let row_begin = (hovered / entries_per_row as i32) * entries_per_row as i32;
                hovered += 1;
                if hovered >= row_begin + entries_per_row as i32 {
                    hovered = row_begin;
                }
            }

            current_page = hovered / entries_per_page as i32;

            if page_up {
                current_page -= 1;
                if current_page < 0 {
                    current_page = page_cnt as i32 - 1;
                    hovered += (map_cnt + (entries_per_page - (map_cnt % entries_per_page))
                        - entries_per_page) as i32;
                    if hovered >= map_cnt as i32 {
                        hovered = map_cnt as i32 - 1
                    }
                } else {
                    hovered -= entries_per_page as i32;
                }
            }

            if page_down {
                current_page += 1;
                if current_page >= page_cnt as i32 {
                    current_page = 0;
                    hovered %= entries_per_page as i32;
                } else {
                    hovered += entries_per_page as i32;
                    if hovered >= map_cnt as i32 {
                        hovered = map_cnt as i32 - 1;
                    }
                }
            }

            current_page %= page_cnt as i32;

            {
                if page_cnt > 1 {
                    draw_aligned_text(
                        &format!("page {}/{}", current_page + 1, page_cnt),
                        screen_size - vec2(25.0, 25.0),
                        HorizontalAlignment::Right,
                        VerticalAlignment::Bottom,
                        Default::default(),
                    );
                }

                let begin = (current_page as usize * entries_per_page).clamp(0, map_cnt);
                let end = (begin as usize + entries_per_page).clamp(begin, map_cnt);

                for (pi, i) in (begin..end).enumerate() {
                    let map_entry = resources.maps.get(i).unwrap();
                    let is_hovered = hovered == i as i32;

                    let mut rect = Rect::new(
                        screen_margins.x + ((pi % entries_per_row) as f32 * entry_size.x),
                        screen_margins.y + ((pi / entries_per_row) as f32 * entry_size.y),
                        entry_size.x,
                        entry_size.y,
                    );

                    if !is_hovered {
                        let w = rect.w * MAP_SELECT_PREVIEW_SHRINK_FACTOR;
                        let h = rect.h * MAP_SELECT_PREVIEW_SHRINK_FACTOR;

                        rect.x += (rect.w - w) / 2.0;
                        rect.y += (rect.h - h) / 2.0;

                        rect.w = w;
                        rect.h = h;
                    }

                    if previous_mouse_pos != mouse_position()
                        && rect.contains(mouse_position().into())
                    {
                        hovered = i as _;
                    }

                    if ui::widgets::Button::new(map_entry.preview)
                        .size(rect.size())
                        .position(rect.point())
                        .ui(&mut *root_ui())
                        || start
                    {
                        root_ui().pop_skin();
                        let res = resources.maps.get(hovered as usize).cloned().unwrap();
                        return res;
                    }
                }
            }
        }

        previous_mouse_pos = mouse_position();

        root_ui().pop_skin();

        next_frame().await;
    }
}

pub async fn create_map() -> Result<MapResource> {
    let mut res = None;

    let size = vec2(450.0, 500.0);
    let position = vec2(
        (screen_width() - size.x) / 2.0,
        (screen_height() - size.y) / 2.0,
    );

    next_frame().await;

    let mut gui_resources = storage::get_mut::<GuiResources>();
    root_ui().push_skin(&gui_resources.skins.login_skin);

    let mut name = "Unnamed Map".to_string();
    let mut description = "".to_string();
    let mut grid_width = "100".to_string();
    let mut grid_height = "100".to_string();
    let mut tile_width = "32".to_string();
    let mut tile_height = "32".to_string();

    let map_exports_path = {
        let resources = storage::get::<Resources>();
        Path::new(&resources.assets_dir).join(Resources::MAP_EXPORTS_DEFAULT_DIR)
    };

    loop {
        gui_resources.gamepads.update();

        clear_background(BLACK);

        widgets::Window::new(hash!(), position, size)
            .titlebar(false)
            .movable(false)
            .ui(&mut *root_ui(), |ui| {
                ui.label(None, "New map");

                ui.separator();
                ui.separator();
                ui.separator();
                ui.separator();

                {
                    let size = vec2(300.0, 25.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .ui(ui, &mut name);
                }

                ui.separator();
                ui.separator();
                ui.separator();
                ui.separator();

                {
                    let path_label = map_exports_path
                        .join(map_name_to_filename(&name))
                        .with_extension(Resources::MAP_EXPORTS_EXTENSION);

                    widgets::Label::new(format!("'{}'", path_label.to_str().unwrap())).ui(ui);
                }

                ui.separator();
                ui.separator();
                ui.separator();
                ui.separator();

                {
                    let size = vec2(300.0, 75.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .ui(ui, &mut description);
                }

                ui.separator();
                ui.separator();
                ui.separator();
                ui.separator();

                {
                    let size = vec2(75.0, 25.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .label("x")
                        .ui(ui, &mut tile_width);

                    ui.same_line(size.x + 25.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .label("Tile size")
                        .ui(ui, &mut tile_height);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .label("x")
                        .ui(ui, &mut grid_width);

                    ui.same_line(size.x + 25.0);

                    widgets::InputText::new(hash!())
                        .size(size)
                        .ratio(1.0)
                        .label("Grid size")
                        .ui(ui, &mut grid_height);
                }

                ui.separator();
                ui.separator();
                ui.separator();
                ui.separator();

                let btn_a = is_gamepad_btn_pressed(&*gui_resources, quad_gamepad::GamepadButton::A);
                let enter = is_key_pressed(KeyCode::Enter);

                if ui.button(None, "Confirm (A) (Enter)") || btn_a || enter {
                    // TODO: Validate input

                    let tile_size = vec2(
                        tile_width.parse::<f32>().unwrap(),
                        tile_height.parse::<f32>().unwrap(),
                    );

                    let grid_size = uvec2(
                        grid_width.parse::<u32>().unwrap(),
                        grid_height.parse::<u32>().unwrap(),
                    );

                    let params = (name.clone(), description.clone(), tile_size, grid_size);

                    res = Some(params);
                }
            });

        if let Some((name, description, tile_size, grid_size)) = res {
            root_ui().pop_skin();

            let description = if description.is_empty() {
                None
            } else {
                Some(description.as_str())
            };

            let mut resources = storage::get_mut::<Resources>();
            return resources.create_map(&name, description, tile_size, grid_size, true);
        }

        next_frame().await;
    }
}
