//! Player input types.
//!
//! We use the [`leafwing-input-manager`] plugin for collecting user input in Jumpy. This crate
//! installs the input manager plugin and registers the [`PlayerAction`] type as the collectible
//! player input.
//!
//! [`leafwing-input-manager`]: https://docs.rs/leafwing-input-manager

use crate::prelude::*;

/// Input plugin.
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
pub struct CurrentEditorInput(Option<jumpy_core::input::EditorInput>);
