use std::borrow::Cow;

use bevy::prelude::Gamepad;
use leafwing_input_manager::{axislike::VirtualDPad, prelude::InputMap, user_input::InputKind};
use serde::{Deserialize, Serialize};

use crate::{input::PlayerAction, platform::Storage};

use super::GameMeta;

/// Global settings, stored and accessed through [`crate::platform::Storage`]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Settings {
    /// The player controller bindings
    pub player_controls: PlayerControlMethods,
    /// The address of the matchmaking server to connect to for online games.
    pub matchmaking_server: String,
}

impl Settings {
    /// The key used to store the settings in the [`crate::platform::Storage`] resource.
    pub const STORAGE_KEY: &'static str = "settings";

    pub fn get_stored_or_default<'w>(
        game: &'w GameMeta,
        storage: &'w mut Storage,
    ) -> Cow<'w, Self> {
        if let Some(settings) = storage.get::<Self>(Self::STORAGE_KEY) {
            Cow::Owned(settings)
        } else {
            Cow::Borrowed(&game.default_settings)
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlayerControlMethods {
    /// Controls for game remotes
    pub gamepad: PlayerControls,
    /// Controls for keyboard player 1
    pub keyboard1: PlayerControls,
    /// Controls for keyboard player 2
    pub keyboard2: PlayerControls,
}

impl PlayerControlMethods {
    /// Get the input map for the given player index
    pub fn get_input_map(&self, player_idx: usize) -> InputMap<PlayerAction> {
        let mut input_map = InputMap::default();

        input_map.set_gamepad(Gamepad { id: player_idx });

        let mut add_controls = |ctrls: &PlayerControls| {
            input_map.insert(ctrls.movement.clone(), PlayerAction::Move);
            input_map.insert(ctrls.jump, PlayerAction::Jump);
            input_map.insert(ctrls.grab, PlayerAction::Grab);
            input_map.insert(ctrls.shoot, PlayerAction::Shoot);
            input_map.insert(ctrls.slide, PlayerAction::Slide);
        };

        add_controls(&self.gamepad);

        match player_idx {
            0 => add_controls(&self.keyboard1),
            1 => add_controls(&self.keyboard2),
            _ => (),
        }

        input_map
    }
}

/// Binds inputs to player actions
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PlayerControls {
    pub movement: VirtualDPad,
    pub jump: InputKind,
    pub grab: InputKind,
    pub shoot: InputKind,
    pub slide: InputKind,
}
