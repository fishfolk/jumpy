use macroquad::{experimental::collections::storage, input, prelude::KeyCode};

use gamepad_rs::ControllerStatus::Connected;

#[derive(Default, Clone)]
pub struct InputAxises {
    pub up: bool,
    pub up_pressed: bool,
    pub down: bool,
    pub down_pressed: bool,
    pub left: bool,
    pub left_pressed: bool,
    pub right: bool,
    pub right_pressed: bool,
    pub start: bool,
    pub start_pressed: bool,
    pub btn_a: bool,
    pub btn_a_pressed: bool,
    pub btn_b: bool,
    pub btn_b_pressed: bool,
}

impl InputAxises {
    pub fn update(&mut self) {
        let old = self.clone();

        self.left = input::is_key_down(KeyCode::Left) || input::is_key_down(KeyCode::A);
        self.right = input::is_key_down(KeyCode::Right) || input::is_key_down(KeyCode::D);
        self.up = input::is_key_down(KeyCode::Up) || input::is_key_down(KeyCode::W);
        self.down = input::is_key_down(KeyCode::Down) || input::is_key_down(KeyCode::S);
        self.start = input::is_key_down(KeyCode::Escape);
        self.btn_a = input::is_key_down(KeyCode::Enter);
        self.btn_b = input::is_key_down(KeyCode::L);

        let controller = storage::get_mut::<gamepad_rs::ControllerContext>();
        for i in 0..2 {
            let state = controller.state(i);
            // let info = controller.info(i);

            if state.status == Connected {
                // { && info.name.contains("MY-POWER CO") {
                let x = state.analog_state[0];
                let y = state.analog_state[1];

                self.left |= x < -0.5;
                self.right |= x > 0.5;
                self.up |= y < -0.5;
                self.down |= y > 0.5;

                self.start |= state.digital_state[9];
                self.btn_a |= state.digital_state[2];
                self.btn_b |= state.digital_state[1];
            }
        }

        if !old.start && self.start {
            self.start_pressed = true;
        } else {
            self.start_pressed = false;
        }

        if !old.btn_a && self.btn_a {
            self.btn_a_pressed = true;
        } else {
            self.btn_a_pressed = false;
        }

        if !old.btn_b && self.btn_b {
            self.btn_b_pressed = true;
        } else {
            self.btn_b_pressed = false;
        }

        if !old.up && self.up {
            self.up_pressed = true;
        } else {
            self.up_pressed = false;
        }

        if !old.down && self.down {
            self.down_pressed = true;
        } else {
            self.down_pressed = false;
        }

        if !old.left && self.left {
            self.left_pressed = true;
        } else {
            self.left_pressed = false;
        }

        if !old.right && self.right {
            self.right_pressed = true;
        } else {
            self.right_pressed = false;
        }
    }
}
