use macroquad::{
    experimental::collections::storage,
    input::{is_key_down, KeyCode},
};

use nanoserde::{DeBin, SerBin};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputScheme {
    /// Left side of the keyboard, around WASD
    KeyboardRight,
    /// Left side of the keyboard, around Arrows
    KeyboardLeft,
    /// Gamepad index
    Gamepad(gilrs::GamepadId),
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
        use gilrs::{Axis, Button};

        let gui_resources = storage::get_mut::<crate::gui::GuiResources>();

        let gamepad = gui_resources.gamepads.gamepad(ix);

        input.throw = gamepad.is_pressed(Button::West);
        input.fire = gamepad.is_pressed(Button::East);

        input.jump = gamepad.is_pressed(Button::South);
        input.left = gamepad.value(Axis::LeftStickX) < -0.5;
        input.right = gamepad.value(Axis::LeftStickX) > 0.5;
        input.down = gamepad.value(Axis::LeftStickY) < -0.5;

        input.slide = gamepad.is_pressed(Button::North);
    }

    input
}
