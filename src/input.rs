use macroquad::{
    experimental::collections::storage,
    input::{
        is_key_down,
        is_key_pressed,
        KeyCode,
        MouseButton,
        is_mouse_button_down,
        is_mouse_button_pressed,
    },
    math::{
        Vec2,
        vec2,
    },
};

use quad_gamepad::GamepadButton;

use nanoserde::{DeBin, SerBin};
use macroquad::input::mouse_wheel;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorInputScheme {
    Keyboard,
    Gamepad(usize),
}

#[derive(Default, Debug, Clone, Copy)]
pub struct EditorInput {
    pub action: bool,
    pub back: bool,
    pub context_menu: bool,
    pub camera_move: Vec2,
    pub camera_zoom: f32,
    pub cursor_move: Vec2,
}

pub fn collect_editor_input(scheme: EditorInputScheme) -> EditorInput {
    let mut input = EditorInput::default();

    match scheme {
        EditorInputScheme::Keyboard => {
            input.action = is_mouse_button_down(MouseButton::Left);
            input.back = is_mouse_button_down(MouseButton::Middle);
            input.context_menu = is_mouse_button_pressed(MouseButton::Right);

            if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                input.camera_move.x = -1.0;
            } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                input.camera_move.x = 1.0;
            }

            if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
                input.camera_move.y = -1.0;
            } else if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
                input.camera_move.y = 1.0;
            }

            let (_, zoom) = mouse_wheel();
            if zoom < 0.0 {
                input.camera_zoom = -1.0;
            } else if zoom > 0.0 {
                input.camera_zoom = 1.0;
            }
        },
        EditorInputScheme::Gamepad(ix) => {
            let gui_resources = storage::get_mut::<crate::gui::GuiResources>();

            let state = gui_resources.gamepads.state(ix);

            input.action = state.digital_state[GamepadButton::B as usize];
            input.back = state.digital_state[GamepadButton::A as usize];
            input.context_menu = state.digital_state[GamepadButton::X as usize];

            input.camera_move = {
                let direction = vec2(
                    state.analog_state[0],
                    state.analog_state[1],
                );

                direction.normalize_or_zero()
            };

            input.cursor_move = {
                let direction = vec2(
                    state.analog_state[2],
                    state.analog_state[3],
                );

                direction.normalize_or_zero()
            };
        }
    }

    input
}