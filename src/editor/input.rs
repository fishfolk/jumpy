use macroquad::{
    experimental::{
        collections::storage,
    },
    prelude::*,
};

use quad_gamepad::GamepadButton;

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
    pub camera_pan: Vec2,
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
                input.camera_pan.x = -1.0;
            } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                input.camera_pan.x = 1.0;
            }

            if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
                input.camera_pan.y = -1.0;
            } else if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
                input.camera_pan.y = 1.0;
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

            input.camera_pan = {
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