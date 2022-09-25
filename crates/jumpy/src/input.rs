use crate::prelude::*;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .add_plugin(InputManagerPlugin::<MenuAction>::default());
    }
}

#[derive(Debug, Copy, Clone, Actionlike, Deserialize, Eq, PartialEq, Hash)]
pub enum PlayerAction {
    Move,
    // Attacks
    Attack,
    Throw,
    Shoot,
}

#[derive(Debug, Copy, Clone, Actionlike, Deserialize, Eq, PartialEq, Hash)]
pub enum MenuAction {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Back,
    Pause,
    ToggleFullscreen,
}
