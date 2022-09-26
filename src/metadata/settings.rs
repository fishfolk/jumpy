use bevy::prelude::Gamepad;
use bevy_has_load_progress::HasLoadProgress;
use leafwing_input_manager::{axislike::VirtualDPad, prelude::InputMap, user_input::InputKind};
use serde::{Deserialize, Serialize};

use crate::input::PlayerAction;

/// Global settings, stored and accessed through [`crate::platform::Storage`]
#[derive(HasLoadProgress, Deserialize, Serialize, Debug, Clone)]
#[has_load_progress(none)]
pub struct Settings {
    // The player controller bindings
    pub player_controls: PlayerControlMethods,
}

impl Settings {
    /// The key used to store the settings in the [`crate::platform::Storage`] resource.
    pub const STORAGE_KEY: &'static str = "settings";
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
    #[allow(unused)] // TODO: Remove when we use it
    pub fn get_input_map(&self, player_idx: usize) -> InputMap<PlayerAction> {
        let mut input_map = InputMap::default();

        input_map.set_gamepad(Gamepad { id: player_idx });

        let mut add_controls = |ctrls: &PlayerControls| {
            input_map.insert(ctrls.movement.clone(), PlayerAction::Move);
            input_map.insert(ctrls.flop_attack, PlayerAction::Attack);
            input_map.insert(ctrls.shoot, PlayerAction::Shoot);
            input_map.insert(ctrls.throw, PlayerAction::Throw);
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
    pub flop_attack: InputKind,
    pub throw: InputKind,
    pub shoot: InputKind,
}
