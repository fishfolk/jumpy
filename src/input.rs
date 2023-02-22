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

/// Bevy resource containing the editor action to perform for this frame.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct EditorAction(Option<jumpy_core::input::EditorInput>);
