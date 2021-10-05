use macroquad::{
    experimental::collections::storage,
    input::{is_key_down, KeyCode},
};

use quad_gamepad::GamepadButton;

use nanoserde::{DeBin, SerBin};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InputScheme {
    /// Left side of the keyboard, around WASD
    KeyboardRight,
    /// Left side of the keyboard, around Arrows
    KeyboardLeft,
    /// Gamepad index
    Gamepad(usize),
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
        use GamepadButton::*;

        let gui_resources = storage::get_mut::<crate::gui::GuiResources>();

        let state = gui_resources.gamepads.state(ix);

        input.throw = state.digital_state[X as usize];
        input.fire = state.digital_state[B as usize];

        input.jump = state.digital_state[A as usize];
        input.left = state.analog_state[0] < -0.5;
        input.right = state.analog_state[0] > 0.5;
        input.down = state.analog_state[1] > 0.5;

        input.slide = state.digital_state[Y as usize];
    }

    input
}
