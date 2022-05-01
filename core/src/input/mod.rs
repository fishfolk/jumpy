pub mod mapping;

pub use mapping::{Button, KeyCode};

use fishsticks::Axis;

use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

pub use fishsticks::GamepadContext;

use crate::{Config, Result};

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    pub fire: bool,
    pub jump: bool,
    pub pickup: bool,
    pub float: bool,
    pub crouch: bool,
    pub slide: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameInputScheme {
    /// Left side of the keyboard, around WASD
    KeyboardRight,
    /// Left side of the keyboard, around Arrows
    KeyboardLeft,
    /// Gamepad index
    Gamepad(fishsticks::GamepadId),
}

pub fn update_gamepad_context(context: Option<&mut GamepadContext>) -> Result<()> {
    if let Some(context) = context {
        context.update()?;
    } else {
        let mut context = storage::get_mut::<GamepadContext>();
        context.update()?;
    }

    Ok(())
}

pub fn is_gamepad_btn_pressed(context: Option<&GamepadContext>, btn: fishsticks::Button) -> bool {
    let check = |context: &GamepadContext| -> bool {
        for (_, gamepad) in context.gamepads() {
            if gamepad.digital_inputs.just_activated(btn) {
                return true;
            }
        }

        false
    };

    if let Some(context) = context {
        check(context)
    } else {
        let context = storage::get::<GamepadContext>();
        check(&context)
    }
}

pub fn collect_local_input(input_scheme: GameInputScheme) -> PlayerInput {
    let mut input = PlayerInput::default();

    if let GameInputScheme::Gamepad(ix) = &input_scheme {
        let gamepad_context = storage::get_mut::<GamepadContext>();
        let gamepad = gamepad_context.gamepad(*ix);

        if let Some(gamepad) = gamepad {
            let input_mapping = {
                let config = storage::get::<Config>();
                config
                    .input
                    .get_gamepad_mapping((*ix).into())
                    .unwrap_or_else(|| ix.into())
            };

            input.left = gamepad.digital_inputs.activated(Button::DPadLeft.into())
                || gamepad.analog_inputs.digital_value(Axis::LeftStickX) < 0.0;

            input.right = gamepad.digital_inputs.activated(Button::DPadRight.into())
                || gamepad.analog_inputs.digital_value(Axis::LeftStickX) > 0.0;

            input.fire = gamepad.digital_inputs.activated(input_mapping.fire.into());

            input.jump = gamepad
                .digital_inputs
                .just_activated(input_mapping.jump.into());

            input.pickup = gamepad
                .digital_inputs
                .just_activated(input_mapping.pickup.into());

            input.crouch = gamepad.digital_inputs.activated(Button::DPadDown.into())
                || gamepad.analog_inputs.digital_value(Axis::LeftStickY) > 0.0;

            input.slide = input.crouch
                && gamepad
                    .digital_inputs
                    .just_activated(input_mapping.slide.into());
        }
    } else {
        let input_mapping = {
            let config = storage::get::<Config>();

            if matches!(input_scheme, GameInputScheme::KeyboardRight) {
                config.input.keyboard_primary.clone()
            } else {
                config.input.keyboard_secondary.clone()
            }
        };

        input.left = is_key_down(input_mapping.left.into());
        input.right = is_key_down(input_mapping.right.into());
        input.fire = is_key_down(input_mapping.fire.into());
        input.jump = is_key_pressed(input_mapping.jump.into());
        input.pickup = is_key_pressed(input_mapping.pickup.into());
        input.float = is_key_down(input_mapping.jump.into());
        input.crouch = is_key_down(input_mapping.crouch.into());
        input.slide = input.crouch && is_key_pressed(input_mapping.slide.into());
    }

    input
}
