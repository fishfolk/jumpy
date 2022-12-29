use crate::prelude::*;

pub struct JumpyPlayerInputPlugin;

impl Plugin for JumpyPlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InputManagerPlugin::<PlayerAction>::default());
    }
}

/// The control inputs that a player may make.
#[derive(Debug, Copy, Clone, Actionlike, Deserialize, Eq, PartialEq, Hash)]
pub enum PlayerAction {
    Move,
    Jump,
    Shoot,
    Grab,
    Slide,
}
