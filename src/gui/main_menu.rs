use std::borrow::BorrowMut;

use ff_core::prelude::*;

use fishsticks::{Axis, Button, GamepadContext};
use hecs::World;

use ff_core::gui::background::draw_main_menu_background;
use ff_core::gui::{
    get_gui_theme, Menu, MenuEntry, MenuResult, Panel, WINDOW_BG_COLOR, WINDOW_MARGIN_H,
    WINDOW_MARGIN_V,
};
use ff_core::Result;

use crate::player::{PlayerAnimations, PlayerControllerKind, PlayerParams};
use crate::{build_state_for_game_mode, gui, GameMode, GuiTheme, Map};
use ff_core::input::{is_gamepad_button_pressed, GameInputScheme};
use ff_core::macroquad::ui::{root_ui, widgets};
use ff_core::macroquad::window::next_frame;
use ff_core::macroquad::{hash, ui};
use ff_core::resources::MapResource;

use crate::player::character::{get_character, iter_characters};

const MENU_WIDTH: f32 = 300.0;

const HEADER_TEXTURE_ID: &str = "main_menu_header";

const LOCAL_GAME_MENU_WIDTH: f32 = 400.0;
const LOCAL_GAME_MENU_HEIGHT: f32 = 200.0;

const MAP_SELECT_SCREEN_MARGIN_FACTOR: f32 = 0.1;
const MAP_SELECT_PREVIEW_TARGET_WIDTH: f32 = 250.0;
const MAP_SELECT_PREVIEW_RATIO: f32 = 10.0 / 16.0;
const MAP_SELECT_PREVIEW_SHRINK_FACTOR: f32 = 0.8;

enum MainMenuResult {
    LocalGame {
        map: Map,
        players: Vec<PlayerParams>,
    },
    Editor {
        map: Option<Map>,
    },
    ReloadResources,
    Quit,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[allow(dead_code)]
enum MainMenuLevel {
    Root,
    LocalGame,
    Settings,
    Editor,
    Credits,
    CharacterSelect,
    GameMapSelect,
    EditorMapSelect,
}

const MAX_PLAYERS: usize = 4;

const CHARACTER_SELECT_SECTION_WIDTH: f32 = 300.0;
const CHARACTER_SELECT_SECTION_HEIGHT: f32 = 400.0;

const CHARACTER_SELECT_SECTION_MARGIN: f32 = 16.0;

const CHARACTER_SELECT_NAVIGATION_GRACE_TIME: f32 = 0.25;

const CHARACTER_SELECT_NAVIGATION_BTN_WIDTH: f32 = 64.0;
const CHARACTER_SELECT_NAVIGATION_BTN_HEIGHT: f32 = 64.0;

const ROOT_OPTION_LOCAL_GAME: usize = 0;
const ROOT_OPTION_EDITOR: usize = 1;
const ROOT_OPTION_SETTINGS: usize = 2;
const ROOT_OPTION_RELOAD_RESOURCES: usize = 3;
const ROOT_OPTION_CREDITS: usize = 4;

const LOCAL_GAME_OPTION_SUBMIT: usize = 0;

const SETTINGS_OPTION_TEST: usize = 0;

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
    .with_cancel_button(None)
}

fn build_settings_menu() -> Menu {
    Menu::new(
        hash!("main_menu", "settings"),
        MENU_WIDTH,
        &[MenuEntry {
            index: SETTINGS_OPTION_TEST,
            title: "Test".to_string(),
            ..Default::default()
        }],
    )
    .with_confirm_button(None)
    .with_cancel_button(None)
}

#[derive(Default, Clone)]
struct CharacterSelectState {
    selections: Vec<usize>,
    sprites: Vec<AnimatedSprite>,
    navigation_grace_timers: Vec<f32>,
    is_ready: Vec<bool>,
}

#[derive(Default, Clone)]
struct MapSelectState {
    selected: usize,
    hovered: i32,
    current_page: i32,
    mouse_position: Vec2,
}

impl CharacterSelectState {
    pub fn new(player_cnt: usize) -> Self {
        CharacterSelectState {
            selections: (0..player_cnt).map(|i| i).collect(),
            sprites: (0..player_cnt)
                .map(|i| {
                    let character = get_character(i);
                    let texture_res = get_texture(&character.sprite.texture_id);
                    let meta: AnimatedSpriteMetadata = character.sprite.clone().into();
                    let animations = meta
                        .animations
                        .iter()
                        .map(|meta| meta.clone().into())
                        .collect::<Vec<_>>();
                    let params = meta.into();
                    AnimatedSprite::new(
                        texture_res.texture,
                        texture_res.frame_size(),
                        &animations,
                        params,
                    )
                })
                .collect(),
            navigation_grace_timers: (0..player_cnt).map(|_| 0.0).collect(),
            is_ready: (0..player_cnt).map(|_| false).collect(),
        }
    }
}

#[derive(Clone)]
pub struct MainMenuState {
    header_texture: Option<Texture2D>,
    current_level: MainMenuLevel,
    current_instance: Option<Menu>,
    local_input: Vec<GameInputScheme>,
    character_select_state: CharacterSelectState,
    map_select_state: MapSelectState,
    player_cnt: usize,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self::new()
    }
}

impl MainMenuState {
    const STATE_ID: &'static str = "main_menu";

    pub fn new() -> Self {
        MainMenuState {
            header_texture: None,
            current_level: MainMenuLevel::Root,
            current_instance: Some(build_main_menu()),
            local_input: Vec::new(),
            character_select_state: CharacterSelectState::default(),
            map_select_state: MapSelectState::default(),
            player_cnt: 0,
        }
    }

    fn set_level(&mut self, level: MainMenuLevel) {
        if level != self.current_level {
            self.current_level = level;

            self.current_instance = match level {
                MainMenuLevel::Root => Some(build_main_menu()),
                MainMenuLevel::Editor => Some(build_editor_menu()),
                MainMenuLevel::Settings => Some(build_settings_menu()),
                _ => None,
            }
        }
    }

    fn draw_local_game(&mut self) {
        let player_cnt = self.local_input.len();

        if player_cnt > 1
            && (is_key_pressed(KeyCode::Enter) || is_gamepad_button_pressed(Button::Start))
        {
            self.character_select_state = CharacterSelectState::new(player_cnt);
            self.set_level(MainMenuLevel::CharacterSelect);
        } else if is_key_pressed(KeyCode::Escape) || is_gamepad_button_pressed(Button::East) {
            self.set_level(MainMenuLevel::Root);
        } else if player_cnt < MAX_PLAYERS {
            if is_key_pressed(KeyCode::Enter) {
                if !self.local_input.contains(&GameInputScheme::KeyboardLeft) {
                    self.local_input.push(GameInputScheme::KeyboardLeft);
                } else {
                    self.local_input.push(GameInputScheme::KeyboardRight);
                }
            }

            let gamepad_ctx = get_gamepad_context();
            for (ix, gamepad) in gamepad_ctx.gamepads() {
                if gamepad.digital_inputs.activated(fishsticks::Button::Start)
                    && !self.local_input.contains(&GameInputScheme::Gamepad(ix))
                {
                    self.local_input.push(GameInputScheme::Gamepad(ix));
                }
            }
        }

        let viewport = get_viewport();

        let size = vec2(LOCAL_GAME_MENU_WIDTH, LOCAL_GAME_MENU_HEIGHT);

        let position = vec2(viewport.width - size.x, viewport.height - size.y) / 2.0;

        Panel::new(hash!(), size, position).ui(&mut *root_ui(), |ui, _| {
            {
                let gui_theme = get_gui_theme();
                ui.push_skin(&gui_theme.menu);
            }

            {
                let position = vec2(12.0, 12.0);

                if !self.local_input.is_empty() {
                    ui.label(position, "Player 1: READY");
                } else {
                    ui.label(position, "Player 1: press A or SPACE");
                }
            }

            {
                let position = vec2(12.0, 44.0);

                if self.local_input.len() > 1 {
                    ui.label(position, "Player 2: READY");
                } else {
                    ui.label(position, "Player 2: press A or SPACE");
                }
            }

            {
                let mut position = vec2(12.0, 108.0);

                if player_cnt > 1 {
                    ui.label(position, "Press START or ENTER to begin");
                    position.y += 24.0;
                }

                ui.label(position, "Press B or ESC to cancel");
            }

            ui.pop_skin();
        });

        if player_cnt > 1
            && (is_key_pressed(KeyCode::Enter) || is_gamepad_button_pressed(Button::Start))
        {
            self.character_select_state = CharacterSelectState::new(player_cnt);
            self.player_cnt = player_cnt;
            self.set_level(MainMenuLevel::CharacterSelect);
        }
    }

    fn draw_character_select(&mut self) {
        let section_size = vec2(
            CHARACTER_SELECT_SECTION_WIDTH,
            CHARACTER_SELECT_SECTION_HEIGHT,
        );
        let total_size = vec2(
            ((section_size.x + CHARACTER_SELECT_SECTION_MARGIN) * self.player_cnt as f32)
                - CHARACTER_SELECT_SECTION_MARGIN,
            section_size.y,
        );

        let viewport = get_viewport();

        let first_position = (vec2(viewport.width, viewport.height) - total_size) / 2.0;

        {
            let gui_theme = get_gui_theme();
            root_ui().push_skin(&gui_theme.default);
        }

        for (i, input_scheme) in self.local_input.iter().enumerate() {
            let section_position = vec2(
                first_position.x + ((section_size.x + CHARACTER_SELECT_SECTION_MARGIN) * i as f32),
                first_position.y,
            );

            let mut current_selection = self.character_select_state.selections[i] as i32;

            let mut should_navigate_left = false;
            let mut should_navigate_right = false;
            let mut should_confirm = false;

            {
                let can_navigate = self.character_select_state.navigation_grace_timers[i]
                    >= CHARACTER_SELECT_NAVIGATION_GRACE_TIME;

                match *input_scheme {
                    GameInputScheme::KeyboardRight => {
                        should_navigate_left = can_navigate && is_key_down(KeyCode::Left);
                        should_navigate_right = can_navigate && is_key_down(KeyCode::Right);
                        should_confirm =
                            is_key_pressed(KeyCode::L) || is_key_pressed(KeyCode::Enter);
                    }
                    GameInputScheme::KeyboardLeft => {
                        should_navigate_left = can_navigate && is_key_down(KeyCode::A);
                        should_navigate_right = can_navigate && is_key_down(KeyCode::D);
                        should_confirm =
                            is_key_pressed(KeyCode::V) || is_key_pressed(KeyCode::LeftControl);
                    }
                    GameInputScheme::Gamepad(gamepad_id) => {
                        let gamepad_context = get_gamepad_context();
                        let gamepad = gamepad_context.gamepad(gamepad_id);

                        if let Some(gamepad) = gamepad {
                            should_navigate_left = can_navigate
                                && (gamepad.analog_inputs.digital_value(Axis::LeftStickX) < 0.0
                                    || gamepad.digital_inputs.just_activated(Button::DPadLeft));

                            should_navigate_right = can_navigate
                                && (gamepad.analog_inputs.digital_value(Axis::LeftStickX) > 0.0
                                    || gamepad.digital_inputs.just_activated(Button::DPadRight));

                            should_confirm = gamepad.digital_inputs.just_activated(Button::South);
                        }
                    }
                }

                Panel::new(hash!("section", i), section_size, section_position)
                    .with_title(&format!("Player {}", i + 1), true)
                    .with_background_color(WINDOW_BG_COLOR)
                    .ui(&mut *root_ui(), |ui, inner_size| {
                        let sprite = &mut self.character_select_state.sprites[i];

                        // TODO: Calculate scale from a fixed target size, based on ui layout
                        sprite.scale = 2.0;

                        let animation_size = sprite.size();
                        let animation_transform = {
                            let position = section_position
                                + vec2((section_size.x - animation_size.width) / 2.0, 100.0);
                            Transform::from(position)
                        };

                        draw_one_animated_sprite(&animation_transform, sprite);

                        {
                            let gui_theme = get_gui_theme();
                            ui.push_skin(&gui_theme.window_header);

                            let name_label = &get_character(current_selection as usize).name;

                            let label_size = ui.calc_size(name_label);
                            let label_position = vec2(
                                (inner_size.x - label_size.x) / 2.0,
                                inner_size.y
                                    - CHARACTER_SELECT_NAVIGATION_BTN_HEIGHT
                                    - CHARACTER_SELECT_SECTION_MARGIN
                                    - label_size.y,
                            );

                            widgets::Label::new(name_label)
                                .position(label_position)
                                .ui(ui);

                            ui.pop_skin();
                        }

                        let btn_size = vec2(
                            CHARACTER_SELECT_NAVIGATION_BTN_WIDTH,
                            CHARACTER_SELECT_NAVIGATION_BTN_HEIGHT,
                        );

                        let btn_section = vec2(inner_size.x / 2.0, inner_size.y - btn_size.y);

                        {
                            let btn_position = vec2(
                                btn_section.x
                                    - (CHARACTER_SELECT_SECTION_MARGIN / 2.0)
                                    - btn_size.x,
                                btn_section.y,
                            );

                            should_navigate_left = widgets::Button::new("<")
                                .size(btn_size)
                                .position(btn_position)
                                .ui(ui)
                                || should_navigate_left;
                        }

                        {
                            let btn_position = vec2(
                                btn_section.x + (CHARACTER_SELECT_SECTION_MARGIN / 2.0),
                                btn_section.y,
                            );

                            should_navigate_right = widgets::Button::new(">")
                                .size(btn_size)
                                .position(btn_position)
                                .ui(ui)
                                || should_navigate_right;
                        }
                    });
            }

            if !self.character_select_state.is_ready.contains(&true)
                && (should_navigate_left || should_navigate_right)
            {
                let mut is_taken = true;
                while is_taken {
                    if should_navigate_left {
                        current_selection -= 1;
                    } else if should_navigate_right {
                        current_selection += 1;
                    }

                    if current_selection < 0 {
                        current_selection = iter_characters().len() as i32 - 1;
                    } else {
                        current_selection %= iter_characters().len() as i32;
                    }

                    is_taken = self
                        .character_select_state
                        .selections
                        .iter()
                        .enumerate()
                        .any(|(ii, selection)| ii != i && *selection == current_selection as usize);
                }

                self.character_select_state.selections[i] = current_selection as usize;

                self.character_select_state.navigation_grace_timers[i] = 0.0;

                let character = get_character(current_selection as usize);

                let meta: AnimatedSpriteMetadata = character.sprite.clone().into();

                let texture_res = get_texture(&meta.texture_id);

                let animations = meta
                    .animations
                    .iter()
                    .cloned()
                    .map(|a| a.into())
                    .collect::<Vec<_>>();

                self.character_select_state.sprites[i] = AnimatedSprite::new(
                    texture_res.texture,
                    texture_res.frame_size(),
                    animations.as_slice(),
                    meta.clone().into(),
                );
            }

            if should_confirm {
                self.character_select_state.is_ready[i] = true;
            }
        }

        if !self.character_select_state.is_ready.contains(&false) {
            self.set_level(MainMenuLevel::GameMapSelect);
        }

        root_ui().pop_skin();
    }

    fn draw_map_select(&mut self) -> Option<Map> {
        let mut up = is_key_pressed(KeyCode::Up) || is_key_pressed(KeyCode::W);
        let mut down = is_key_pressed(KeyCode::Down) || is_key_pressed(KeyCode::S);
        let mut right = is_key_pressed(KeyCode::Right) || is_key_pressed(KeyCode::D);
        let mut left = is_key_pressed(KeyCode::Left) || is_key_pressed(KeyCode::A);
        let mut start = is_key_pressed(KeyCode::Enter);

        let (page_up, page_down) = {
            let mouse_wheel = get_mouse_wheel_values();
            (mouse_wheel.y > 0.0, mouse_wheel.y < 0.0)
        };

        for (_, gamepad) in get_gamepad_context().gamepads() {
            use fishsticks::{Axis, Button};

            up |= gamepad.digital_inputs.just_activated(Button::DPadUp)
                || matches!(
                    gamepad.analog_inputs.just_activated_digital(Axis::LeftStickY),
                    Some(value) if value < 0.0
                );

            down |= gamepad.digital_inputs.just_activated(Button::DPadDown)
                || matches!(
                    gamepad.analog_inputs.just_activated_digital(Axis::LeftStickY),
                    Some(value) if value > 0.0
                );

            left |= gamepad.digital_inputs.just_activated(Button::DPadLeft)
                || matches!(
                    gamepad.analog_inputs.just_activated_digital(Axis::LeftStickX),
                    Some(value) if value < 0.0
                );

            right |= gamepad.digital_inputs.just_activated(Button::DPadRight)
                || matches!(
                    gamepad.analog_inputs.just_activated_digital(Axis::LeftStickX),
                    Some(value) if value > 0.0
                );

            start |= gamepad.digital_inputs.just_activated(Button::South)
                || gamepad.digital_inputs.just_activated(Button::Start);
        }

        let map_cnt = iter_maps().len();

        let gui_theme = get_gui_theme();
        root_ui().push_skin(&gui_theme.map_selection);

        let viewport = get_viewport();
        let screen_margins = vec2(
            viewport.width * MAP_SELECT_SCREEN_MARGIN_FACTOR,
            viewport.height * MAP_SELECT_SCREEN_MARGIN_FACTOR,
        );
        let content_size = vec2(
            viewport.width - (screen_margins.x * 2.0),
            viewport.height - (screen_margins.y * 2.0),
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
                self.map_select_state.hovered -= entries_per_row as i32;
                if self.map_select_state.hovered < 0 {
                    self.map_select_state.hovered +=
                        1 + map_cnt as i32 + (map_cnt % entries_per_row) as i32;
                    if self.map_select_state.hovered >= map_cnt as i32 {
                        self.map_select_state.hovered = map_cnt as i32 - 1;
                    }
                }
            }

            if down {
                let old = self.map_select_state.hovered;
                self.map_select_state.hovered += entries_per_row as i32;
                if self.map_select_state.hovered >= map_cnt as i32 {
                    if old == map_cnt as i32 - 1 {
                        self.map_select_state.hovered = 0;
                    } else {
                        self.map_select_state.hovered = map_cnt as i32 - 1;
                    }
                }
            }

            if left {
                let row_begin = (self.map_select_state.hovered / entries_per_row as i32)
                    * entries_per_row as i32;
                self.map_select_state.hovered -= 1;
                if self.map_select_state.hovered < row_begin {
                    self.map_select_state.hovered = row_begin + entries_per_row as i32 - 1;
                }
            }

            if right {
                let row_begin = (self.map_select_state.hovered / entries_per_row as i32)
                    * entries_per_row as i32;
                self.map_select_state.hovered += 1;
                if self.map_select_state.hovered >= row_begin + entries_per_row as i32 {
                    self.map_select_state.hovered = row_begin;
                }
            }

            self.map_select_state.current_page =
                self.map_select_state.hovered / entries_per_page as i32;

            if page_up {
                self.map_select_state.current_page -= 1;
                if self.map_select_state.current_page < 0 {
                    self.map_select_state.current_page = page_cnt as i32 - 1;
                    self.map_select_state.hovered +=
                        (map_cnt + (entries_per_page - (map_cnt % entries_per_page))
                            - entries_per_page) as i32;
                    if self.map_select_state.hovered >= map_cnt as i32 {
                        self.map_select_state.hovered = map_cnt as i32 - 1
                    }
                } else {
                    self.map_select_state.hovered -= entries_per_page as i32;
                }
            }

            if page_down {
                self.map_select_state.current_page += 1;
                if self.map_select_state.current_page >= page_cnt as i32 {
                    self.map_select_state.current_page = 0;
                    self.map_select_state.hovered %= entries_per_page as i32;
                } else {
                    self.map_select_state.hovered += entries_per_page as i32;
                    if self.map_select_state.hovered >= map_cnt as i32 {
                        self.map_select_state.hovered = map_cnt as i32 - 1;
                    }
                }
            }

            self.map_select_state.current_page %= page_cnt as i32;

            {
                if page_cnt > 1 {
                    let pagination_label = format!(
                        "page {}/{}",
                        self.map_select_state.current_page + 1,
                        page_cnt
                    );

                    let label_size = root_ui().calc_size(&pagination_label);
                    let label_position =
                        viewport.as_vec2() - vec2(WINDOW_MARGIN_H, WINDOW_MARGIN_V) - label_size;

                    widgets::Label::new(&pagination_label)
                        .position(label_position)
                        .ui(&mut *root_ui());
                }

                let begin = (self.map_select_state.current_page as usize * entries_per_page)
                    .clamp(0, map_cnt);
                let end = (begin as usize + entries_per_page).clamp(begin, map_cnt);

                for (pi, i) in (begin..end).enumerate() {
                    let map_entry = get_map(i);
                    let is_hovered = self.map_select_state.hovered == i as i32;

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

                    let mouse_position = get_mouse_position();

                    if self.map_select_state.mouse_position != mouse_position
                        && rect.contains(mouse_position.into())
                    {
                        self.map_select_state.hovered = i as _;
                    }

                    let texture: ff_core::macroquad::texture::Texture2D = map_entry.preview.into();

                    if widgets::Button::new(texture)
                        .size(rect.size())
                        .position(rect.point())
                        .ui(&mut *root_ui())
                        || start
                    {
                        root_ui().pop_skin();

                        let res = get_map(self.map_select_state.selected);
                        return Some(res.map.clone());
                    }
                }
            }
        }

        root_ui().pop_skin();

        self.map_select_state.mouse_position = get_mouse_position();

        None
    }

    fn draw_credits(&mut self) {
        self.set_level(MainMenuLevel::Root);
    }

    fn draw_current(&mut self) -> Option<MainMenuResult> {
        if !matches!(
            self.current_level,
            MainMenuLevel::GameMapSelect | MainMenuLevel::EditorMapSelect
        ) {
            if let Some(texture) = self.header_texture {
                draw_main_menu_background(true);

                let size = texture.size();

                let viewport = get_viewport();
                let position = vec2((viewport.width - size.width) / 2.0, 35.0);

                widgets::Texture::new(texture.into())
                    .position(position)
                    .size(size.width, size.height)
                    .ui(&mut *root_ui());
            }
        }

        if let Some(instance) = &mut self.current_instance {
            if let Some(res) = instance.ui(&mut *root_ui()) {
                match self.current_level {
                    MainMenuLevel::Root => {
                        if res.is_cancel() {
                            return Some(MainMenuResult::Quit);
                        } else {
                            match res.into_usize() {
                                ROOT_OPTION_LOCAL_GAME => {
                                    self.set_level(MainMenuLevel::LocalGame);
                                }
                                ROOT_OPTION_EDITOR => {
                                    self.set_level(MainMenuLevel::Editor);
                                }
                                ROOT_OPTION_SETTINGS => {
                                    self.set_level(MainMenuLevel::Settings);
                                }
                                ROOT_OPTION_CREDITS => {
                                    self.set_level(MainMenuLevel::Credits);
                                }
                                _ => {}
                            }
                        }
                    }
                    MainMenuLevel::Editor => {
                        if res.is_cancel() {
                            self.set_level(MainMenuLevel::Root);
                        } else {
                            match res.into_usize() {
                                EDITOR_OPTION_CREATE => {
                                    return Some(MainMenuResult::Editor { map: None });
                                }
                                EDITOR_OPTION_LOAD => {
                                    self.set_level(MainMenuLevel::EditorMapSelect);
                                }
                                _ => {}
                            }
                        }
                    }
                    MainMenuLevel::Settings => {
                        if res.is_confirm() {
                            self.set_level(MainMenuLevel::Root);
                        } else if res.is_cancel() {
                            self.set_level(MainMenuLevel::Root);
                        } else {
                            match res.into_usize() {
                                SETTINGS_OPTION_TEST => {}
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        } else {
            match self.current_level {
                MainMenuLevel::LocalGame => self.draw_local_game(),
                MainMenuLevel::CharacterSelect => self.draw_character_select(),
                MainMenuLevel::GameMapSelect | MainMenuLevel::EditorMapSelect => {
                    if let Some(map) = self.draw_map_select() {
                        if self.current_level == MainMenuLevel::GameMapSelect {
                            return Some(MainMenuResult::LocalGame {
                                map,
                                players: self
                                    .character_select_state
                                    .selections
                                    .clone()
                                    .into_iter()
                                    .enumerate()
                                    .map(|(i, index)| PlayerParams {
                                        index: i as u8,
                                        controller: PlayerControllerKind::LocalInput(
                                            self.local_input[i],
                                        ),
                                        character: get_character(index).clone(),
                                    })
                                    .collect(),
                            });
                        } else {
                            return Some(MainMenuResult::Editor { map: Some(map) });
                        }
                    }
                }
                MainMenuLevel::Credits => self.draw_credits(),
                _ => {}
            }
        }

        None
    }
}

impl GameState for MainMenuState {
    fn id(&self) -> String {
        Self::STATE_ID.to_string()
    }

    fn begin(&mut self, _world: Option<World>) -> Result<()> {
        self.header_texture = try_get_texture(HEADER_TEXTURE_ID).map(|res| res.texture);

        self.set_level(MainMenuLevel::Root);

        Ok(())
    }

    fn update(&mut self, delta_time: f32) -> Result<()> {
        update_gamepad_context().unwrap();

        for sprite in &mut self.character_select_state.sprites {
            for t in &mut self.character_select_state.navigation_grace_timers {
                *t += delta_time;
            }

            update_one_animated_sprite(delta_time, sprite)?;
        }

        Ok(())
    }

    fn draw(&mut self) -> Result<()> {
        if let Some(res) = self.draw_current() {
            match res {
                MainMenuResult::LocalGame { map, players } => {
                    let state = build_state_for_game_mode(GameMode::Local, map, &players);
                    dispatch_event(GameEvent::StateTransition(Box::new(state)));
                }
                MainMenuResult::Editor { map } => {
                    // let state = build_editor_state(map);
                    // dispatch_event(GameEvent::StateTransition(Box::new(state)));
                }
                MainMenuResult::ReloadResources => {
                    dispatch_event(GameEvent::ReloadResources);
                }
                MainMenuResult::Quit => {
                    dispatch_event(GameEvent::Quit);
                }
            }
        }

        Ok(())
    }
}
