use leafwing_input_manager::Actionlike;
use serde::Deserialize;

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
