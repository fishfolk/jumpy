use crate::prelude::*;

/// Settings plugin
pub fn game_plugin(game: &mut Game) {
    game.systems.add_startup_system(load_settings);
}

/// Startup system to load the game settings or use the default settings specified in the game meta.
fn load_settings(game: &mut Game) {
    let default_settings = {
        let assets = game.shared_resource::<AssetServer>().unwrap();
        assets.root::<GameMeta>().default_settings.clone()
    };
    let mut storage = game.shared_resource_mut::<Storage>().unwrap();
    if storage.get::<Settings>().is_none() {
        storage.insert(default_settings);
    }
}

/// Global settings, stored and accessed through [`crate::platform::Storage`]
#[derive(HasSchema, Debug, Clone, Default)]
#[repr(C)]
pub struct Settings {
    /// The player controller bindings
    pub player_controls: PlayerControlMapping,
    /// The address of the matchmaking server to connect to for online games.
    pub matchmaking_server: String,
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

/// Binds inputs to player actions
#[derive(HasSchema, Clone, Debug, Default)]
#[repr(C)]
pub struct PlayerControlSetting {
    pub movement: VirtualDPad,
    pub pause: InputKind,
    pub jump: InputKind,
    pub grab: InputKind,
    pub shoot: InputKind,
    pub slide: InputKind,
    pub menu_back: InputKind,
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
