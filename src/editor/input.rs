use macroquad::{experimental::collections::storage, prelude::*};

use fishsticks::{Axis, Button};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorInputScheme {
    Keyboard,
    Gamepad(fishsticks::GamepadId),
}

#[derive(Default, Debug, Clone, Copy)]
pub struct EditorInput {
    pub action: bool,
    pub back: bool,
    pub context_menu: bool,
    pub camera_pan: Vec2,
    pub camera_zoom: f32,
    pub cursor_move: Vec2,
    pub undo: bool,
    pub redo: bool,
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

            if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::Z) {
                if is_key_down(KeyCode::LeftShift) {
                    input.redo = true;
                } else {
                    input.undo = true;
                }
            }
        }
        EditorInputScheme::Gamepad(ix) => {
            let gamepad_system = storage::get_mut::<fishsticks::GamepadContext>();
            let gamepad = gamepad_system.gamepad(ix);

            if let Some(gamepad) = gamepad {
                input.action = gamepad.digital_inputs.activated(Button::B);
                input.back = gamepad.digital_inputs.activated(Button::A);
                input.context_menu = gamepad.digital_inputs.activated(Button::X);

                input.camera_pan = {
                    let direction_x = match gamepad.analog_inputs.value(Axis::LeftX) {
                        Some(value) => value.get(),
                        None => 0.0,
                    };

                    let direction_y = match gamepad.analog_inputs.value(Axis::LeftY) {
                        Some(value) => value.get(),
                        None => 0.0,
                    };

                    let direction = vec2(direction_x, direction_y);

                    direction.normalize_or_zero()
                };

                input.cursor_move = {
                    let direction_x = match gamepad.analog_inputs.value(Axis::RightX) {
                        Some(value) => value.get(),
                        None => 0.0,
                    };

                    let direction_y = match gamepad.analog_inputs.value(Axis::RightY) {
                        Some(value) => value.get(),
                        None => 0.0,
                    };

                    let direction = vec2(direction_x, direction_y);

                    direction.normalize_or_zero()
                };
            }
        }
    }

    input
}
