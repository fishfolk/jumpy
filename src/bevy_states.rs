//! Bevy [`States`] used in the game.
//!
//! We have three different Bevy state types. The [`EngineState`] is the "parent" or "main" state.
//! We take advantage of the ability to have multiple different kinds of states in Bevy and have
//! [`InGameState`] and [`GameEditorState`] represent the game pause state and editor visibility
//! when the [`EngineState`] is set to [`EngineState::InGame`].

use crate::prelude::*;

/// Bevy [`States`] related to the top-level state of the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, States, Default)]
pub enum EngineState {
    /// The initial state: the game is loading it's platform-specific [`Storage`].
    #[default]
    LoadingPlatformStorage,
    /// The game is in the process of loading assets and processing the metadata.
    ///
    /// See also: [`load_game()`][loading::GameLoader::load].
    LoadingGameData,
    /// The game is on the main menu.
    ///
    /// This includes other sub-menus such as player/map selection, and settings menus, etc.
    MainMenu,
    /// There is a game match being played.
    ///
    /// This includes when you are editing a map, which has a game match going during editing.
    InGame,
}

/// Bevy [`States`] that are relevant while the [`EngineState::InGame`] state is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, States, Default)]
pub enum InGameState {
    /// The game match is playing.
    #[default]
    Playing,
    /// The game is paused.
    Paused,
}

/// Bevy [`States`] tracking the editor visibility, relevant only while the [`EngineState::InGame`]
/// state is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, States, Default)]
pub enum GameEditorState {
    /// The editor is hidden.
    #[default]
    Hidden,
    /// The editor is visible.
    Visible,
}
