use macroquad::{
    experimental::collections::storage,
    input::{is_key_down, KeyCode},
};

use fishsticks::{Axis, Button};

use nanoserde::{DeBin, SerBin};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputScheme {
    /// Left side of the keyboard, around WASD
    KeyboardRight,
    /// Left side of the keyboard, around Arrows
    KeyboardLeft,
    /// Gamepad index
    Gamepad(fishsticks::GamepadId),
}

#[derive(Default, Debug, Clone, Copy, DeBin, SerBin)]
pub struct Input {
    pub jump: bool,
    pub throw: bool,
    pub fire: bool,
    pub slide: bool,

    pub left: bool,
    pub right: bool,
    pub down: bool,
}

pub fn collect_input(scheme: InputScheme) -> Input {
    let mut input = Input::default();

    if let InputScheme::KeyboardLeft = scheme {
        input.throw = is_key_down(KeyCode::C);
        input.fire = is_key_down(KeyCode::V) || is_key_down(KeyCode::LeftControl);

        input.jump = is_key_down(KeyCode::W) || is_key_down(KeyCode::Space);
        input.left = is_key_down(KeyCode::A);
        input.down = is_key_down(KeyCode::S);
        input.right = is_key_down(KeyCode::D);

        input.slide = is_key_down(KeyCode::C);
    }

    if let InputScheme::KeyboardRight = scheme {
        input.throw = is_key_down(KeyCode::K);
        input.fire = is_key_down(KeyCode::L);

        input.jump = is_key_down(KeyCode::Up);
        input.left = is_key_down(KeyCode::Left);
        input.down = is_key_down(KeyCode::Down);
        input.right = is_key_down(KeyCode::Right);

        input.slide = is_key_down(KeyCode::RightControl);
    }

    if let InputScheme::Gamepad(ix) = scheme {
        let gamepad_system = storage::get_mut::<fishsticks::GamepadContext>();
        let gamepad = gamepad_system.gamepad(ix);

        if let Some(gamepad) = gamepad {
            input.throw = gamepad.digital_inputs.activated(Button::X);
            input.fire = gamepad.digital_inputs.activated(Button::B);

            input.jump = gamepad.digital_inputs.activated(Button::A);

            input.left = gamepad.digital_inputs.activated(Button::DPadLeft)
                || matches!(
                    gamepad.analog_inputs.value(Axis::LeftX),
                    Some(value) if value.get() < -0.5
                );

            input.right = gamepad.digital_inputs.activated(Button::DPadRight)
                || matches!(
                    gamepad.analog_inputs.value(Axis::LeftX),
                    Some(value) if value.get() > 0.5
                );

            input.down = gamepad.digital_inputs.activated(Button::DPadDown)
                || matches!(
                    gamepad.analog_inputs.value(Axis::LeftY),
                    Some(value) if value.get() < -0.5
                );
        }
    }

    input
}
