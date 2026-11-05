use strum::IntoEnumIterator;

use crate::prelude::*;

/// Settings plugin
pub fn game_plugin(game: &mut Game) {
    game.systems.add_startup_system(load_settings);
}

/// Startup system to load the game settings or use the default settings specified in the game meta.
fn load_settings(game: &mut Game) {
    let default_settings = {
        let assets = game.shared_resource::<AssetServer>();
        let settings = &assets.root::<GameMeta>().default_settings;
        settings.clone()
    };
    let mut storage = game.shared_resource_mut::<Storage>();
    if storage.get::<Settings>().is_none() {
        storage.insert(default_settings);
    }
}

/// Global settings, stored and accessed through [`Storage`].
#[derive(HasSchema, Debug, Clone)]
#[repr(C)]
pub struct Settings {
    /// The main scaling factor for all game audios. This is done on top of the
    /// scaling factor specific to the audio type.
    pub main_volume: f32,
    /// The scaling factor for music.
    pub music_volume: f32,
    /// The scaling factor for sound effects.
    pub effects_volume: f32,
    /// Whether to display the game fullscreen.
    pub fullscreen: bool,
    /// The player controller bindings
    pub player_controls: PlayerControlMapping,
    /// The address of the matchmaking server to connect to for online games.
    pub matchmaking_server: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            main_volume: 1.0,
            music_volume: 1.0,
            effects_volume: 1.0,
            fullscreen: true,
            player_controls: default(),
            matchmaking_server: default(),
        }
    }
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerControlMapping {
    /// Controls for game remotes
    pub gamepad: PlayerControlSetting,
    /// Controls for keyboard player 1
    pub keyboard1: PlayerControlSetting,
    /// Controls for keyboard player 2
    pub keyboard2: PlayerControlSetting,
}

impl PlayerControlMapping {
    pub fn map_control_source(&self, source: ControlSource) -> &PlayerControlSetting {
        match source {
            ControlSource::Keyboard1 => &self.keyboard1,
            ControlSource::Keyboard2 => &self.keyboard2,
            ControlSource::Gamepad(_) => &self.gamepad,
        }
    }

    /// Return an iterator for PlayerControlSetting for all control sources.
    pub fn all_settings(&self) -> impl Iterator<Item = &PlayerControlSetting> {
        ControlSource::iter().map(|s| self.map_control_source(s))
    }
}

/// Binds inputs to player actions
#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerControlSetting {
    pub movement: VirtualDPad,
    pub movement_alt: VirtualDPad,
    pub pause: InputKind,
    pub jump: InputKind,
    pub grab: InputKind,
    pub shoot: InputKind,
    pub slide: InputKind,
    pub ragdoll: InputKind,
    pub menu_back: InputKind,
    pub menu_start: InputKind,
    pub menu_confirm: InputKind,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct VirtualDPad {
    pub up: InputKind,
    pub down: InputKind,
    pub left: InputKind,
    pub right: InputKind,
}

#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C, u8)]
pub enum InputKind {
    #[default]
    None,
    Button(GamepadButton),
    AxisPositive(GamepadAxis),
    AxisNegative(GamepadAxis),
    Keyboard(KeyCode),
}

impl std::fmt::Display for InputKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InputKind::None => write!(f, "[None]"),
            InputKind::Button(btn) => write!(f, "{btn}"),
            InputKind::AxisPositive(axis) => write!(f, "{axis} +"),
            InputKind::AxisNegative(axis) => write!(f, "{axis} -"),
            InputKind::Keyboard(key) => write!(f, "{key:?}"),
        }
    }
}
