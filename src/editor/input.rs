use macroquad::{experimental::collections::storage, prelude::*};

use fishsticks::{Axis, Button};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EditorInputScheme {
    Mouse,
    Gamepad(fishsticks::GamepadId),
}

#[derive(Debug, Clone, Copy)]
pub struct EditorInput {
    pub action: bool,
    pub double_click: bool,
    pub back: bool,
    pub context_menu: bool,
    pub camera_pan: Vec2,
    pub camera_zoom: f32,
    pub cursor_movement: Vec2,
    pub undo: bool,
    pub redo: bool,
    pub toggle_menu: bool,
    pub toggle_grid: bool,

    double_click_timer: f32,
}

impl Default for EditorInput {
    fn default() -> Self {
        EditorInput {
            action: false,
            double_click: false,
            back: false,
            context_menu: false,
            camera_pan: Vec2::ZERO,
            camera_zoom: 0.0,
            cursor_movement: Vec2::ZERO,
            undo: false,
            redo: false,
            toggle_menu: false,
            toggle_grid: false,
            double_click_timer: Self::DOUBLE_CLICK_THRESHOLD,
        }
    }
}

impl EditorInput {
    const DOUBLE_CLICK_THRESHOLD: f32 = 0.25;

    fn clear(&mut self) {
        self.action = false;
        self.double_click = false;
        self.back = false;
        self.context_menu = false;
        self.camera_pan = Vec2::ZERO;
        self.camera_zoom = 0.0;
        self.cursor_movement = Vec2::ZERO;
        self.undo = false;
        self.redo = false;
        self.toggle_menu = false;
        self.toggle_grid = false;
    }

    pub fn update(&mut self, scheme: EditorInputScheme) {
        let was_action = self.action || self.double_click;

        self.clear();

        match scheme {
            EditorInputScheme::Mouse => {
                if self.double_click_timer < Self::DOUBLE_CLICK_THRESHOLD {
                    self.double_click_timer = (self.double_click_timer + get_frame_time())
                        .clamp(0.0, Self::DOUBLE_CLICK_THRESHOLD);
                }

                if is_mouse_button_down(MouseButton::Left) {
                    self.action = true;

                    if !was_action && self.double_click_timer < Self::DOUBLE_CLICK_THRESHOLD {
                        self.double_click = true;
                    } else {
                        self.double_click_timer = 0.0;
                    }
                }

                self.back = is_mouse_button_down(MouseButton::Middle);
                self.context_menu = is_mouse_button_pressed(MouseButton::Right);

                self.toggle_menu = is_key_pressed(KeyCode::Escape);

                if is_key_down(KeyCode::Left) || is_key_down(KeyCode::A) {
                    self.camera_pan.x = -1.0;
                } else if is_key_down(KeyCode::Right) || is_key_down(KeyCode::D) {
                    self.camera_pan.x = 1.0;
                }

                if is_key_down(KeyCode::Up) || is_key_down(KeyCode::W) {
                    self.camera_pan.y = -1.0;
                } else if is_key_down(KeyCode::Down) || is_key_down(KeyCode::S) {
                    self.camera_pan.y = 1.0;
                }

                let (_, zoom) = mouse_wheel();
                if zoom < 0.0 {
                    self.camera_zoom = -1.0;
                } else if zoom > 0.0 {
                    self.camera_zoom = 1.0;
                }

                if is_key_down(KeyCode::LeftControl) && is_key_pressed(KeyCode::Z) {
                    if is_key_down(KeyCode::LeftShift) {
                        self.redo = true;
                    } else {
                        self.undo = true;
                    }
                }

                self.toggle_grid = is_key_pressed(KeyCode::G);
            }
            EditorInputScheme::Gamepad(ix) => {
                let gamepad_system = storage::get_mut::<fishsticks::GamepadContext>();
                let gamepad = gamepad_system.gamepad(ix);

                if let Some(gamepad) = gamepad {
                    self.action = gamepad.digital_inputs.activated(Button::B);
                    self.back = gamepad.digital_inputs.activated(Button::A);
                    self.context_menu = gamepad.digital_inputs.activated(Button::X);

                    self.camera_pan = {
                        let direction_x = gamepad.analog_inputs.value(Axis::LeftX);
                        let direction_y = gamepad.analog_inputs.value(Axis::LeftY);

                        let direction = vec2(direction_x, direction_y);

                        direction.normalize_or_zero()
                    };

                    self.cursor_movement = {
                        let direction_x = gamepad.analog_inputs.value(Axis::RightX);
                        let direction_y = gamepad.analog_inputs.value(Axis::RightY);

                        let direction = vec2(direction_x, direction_y);

                        direction.normalize_or_zero()
                    };
                }
            }
        }
    }
}
