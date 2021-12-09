use macroquad::{
    experimental::collections::storage,
    input::{is_key_down, KeyCode},
};

use fishsticks::{Axis, Button};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameInputScheme {
    /// Left side of the keyboard, around WASD
    KeyboardRight,
    /// Left side of the keyboard, around Arrows
    KeyboardLeft,
    /// Gamepad index
    Gamepad(fishsticks::GamepadId),
}

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameInput {
    pub jump: bool,
    pub pickup: bool,
    pub fire: bool,
    pub slide: bool,

    pub left: bool,
    pub right: bool,
    pub down: bool,
}

pub fn collect_input(scheme: GameInputScheme) -> GameInput {
    let mut input = GameInput::default();

    if let GameInputScheme::KeyboardLeft = scheme {
        input.pickup = is_key_down(KeyCode::C);
        input.fire = is_key_down(KeyCode::V) || is_key_down(KeyCode::LeftControl);

        input.jump = is_key_down(KeyCode::W) || is_key_down(KeyCode::Space);
        input.left = is_key_down(KeyCode::A);
        input.down = is_key_down(KeyCode::S);
        input.right = is_key_down(KeyCode::D);

        input.slide = is_key_down(KeyCode::C);
    }

    if let GameInputScheme::KeyboardRight = scheme {
        input.pickup = is_key_down(KeyCode::K);
        input.fire = is_key_down(KeyCode::L);

        input.jump = is_key_down(KeyCode::Up);
        input.left = is_key_down(KeyCode::Left);
        input.down = is_key_down(KeyCode::Down);
        input.right = is_key_down(KeyCode::Right);

        input.slide = is_key_down(KeyCode::RightControl);
    }

    if let GameInputScheme::Gamepad(ix) = scheme {
        let gamepad_system = storage::get_mut::<fishsticks::GamepadContext>();
        let gamepad = gamepad_system.gamepad(ix);

        if let Some(gamepad) = gamepad {
            input.pickup = gamepad.digital_inputs.activated(Button::X);
            input.fire = gamepad.digital_inputs.activated(Button::B);

            input.jump = gamepad.digital_inputs.activated(Button::A);

            input.left = gamepad.digital_inputs.activated(Button::DPadLeft)
                || gamepad.analog_inputs.digital_value(Axis::LeftX) < 0.0;

            input.right = gamepad.digital_inputs.activated(Button::DPadRight)
                || gamepad.analog_inputs.digital_value(Axis::LeftX) > 0.0;

            input.down = gamepad.digital_inputs.activated(Button::DPadDown)
                || gamepad.analog_inputs.digital_value(Axis::LeftY) > 0.0;

            input.slide = gamepad.digital_inputs.activated(Button::Y)
        }
    }

    input
}
