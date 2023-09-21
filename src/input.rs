use crate::{
    prelude::*,
    settings::{InputKind, PlayerControlSetting},
};

#[derive(Default)]
pub struct PlayerInputCollector {
    last_controls: [PlayerControl; MAX_PLAYERS],
    current_controls: [PlayerControl; MAX_PLAYERS],
}

impl PlayerInputCollector {
    /// Update the internal state with new inputs. This must be called every render frame with the
    /// input events.
    pub fn update(
        &mut self,
        mapping: &crate::settings::PlayerControlMethods,
        keyboard: &KeyboardInputs,
        gamepad: &GamepadInputs,
    ) {
        // Helper to get the value of the given input type for the given player.
        let get_input_value = |input_map: &InputKind, player_idx: usize| {
            match input_map {
                crate::settings::InputKind::Button(mapped_button) => {
                    for input in &gamepad.gamepad_events {
                        if let GamepadEvent::Button(e) = input {
                            if &e.button == mapped_button && e.gamepad == player_idx as u32 {
                                let value = if e.value < 0.1 { 0.0 } else { e.value };
                                return Some(value);
                            }
                        }
                    }
                }
                crate::settings::InputKind::AxisPositive(mapped_axis) => {
                    for input in &gamepad.gamepad_events {
                        if let GamepadEvent::Axis(e) = input {
                            if &e.axis == mapped_axis && e.gamepad == player_idx as u32 {
                                let value = if e.value < 0.1 { 0.0 } else { e.value };
                                return Some(value);
                            }
                        }
                    }
                }
                crate::settings::InputKind::AxisNegative(mapped_axis) => {
                    for input in &gamepad.gamepad_events {
                        if let GamepadEvent::Axis(e) = input {
                            if &e.axis == mapped_axis && e.gamepad == player_idx as u32 {
                                let value = if e.value > -0.1 { 0.0 } else { e.value };
                                return Some(value);
                            }
                        }
                    }
                }
                crate::settings::InputKind::Keyboard(mapped_key) => {
                    for input in &keyboard.key_events {
                        if input.key_code == Set(*mapped_key) {
                            return Some(if input.button_state.pressed() {
                                1.0
                            } else {
                                0.0
                            });
                        }
                    }
                }
            };
            None
        };

        let apply_controls =
            |control: &mut PlayerControl, player_idx: usize, mapping: &PlayerControlSetting| {
                for (button_pressed, button_map) in [
                    (&mut control.jump_pressed, &mapping.jump),
                    (&mut control.grab_pressed, &mapping.grab),
                    (&mut control.shoot_pressed, &mapping.shoot),
                    (&mut control.slide_pressed, &mapping.slide),
                ] {
                    if let Some(value) = get_input_value(button_map, player_idx) {
                        *button_pressed = value > 0.0;
                    }
                }

                if let Some(left) = get_input_value(&mapping.movement.left, player_idx) {
                    control.left = left.abs();
                }
                if let Some(right) = get_input_value(&mapping.movement.right, player_idx) {
                    control.right = right.abs();
                }
                if let Some(up) = get_input_value(&mapping.movement.up, player_idx) {
                    control.up = up.abs();
                }
                if let Some(down) = get_input_value(&mapping.movement.down, player_idx) {
                    control.down = down.abs();
                }
            };

        for (i, control) in self.current_controls.iter_mut().enumerate() {
            if i == 0 {
                apply_controls(control, i, &mapping.keyboard1);
            } else if i == 1 {
                apply_controls(control, i, &mapping.keyboard2);
            }
            apply_controls(control, i, &mapping.gamepad);
        }
    }

    /// Get the player inputs for the next game simulation frame.
    pub fn get(&mut self) -> &[PlayerControl; MAX_PLAYERS] {
        (0..MAX_PLAYERS).for_each(|i| {
            let current = &mut self.current_controls[i];
            let last = &self.last_controls[i];

            current.move_direction = vec2(current.right - current.left, current.up - current.down);
            current.moving = current.move_direction.length_squared() > 0.0;

            for (just_pressed, current_pressed, last_pressed) in [
                (
                    &mut current.jump_just_pressed,
                    current.jump_pressed,
                    last.jump_pressed,
                ),
                (
                    &mut current.shoot_just_pressed,
                    current.shoot_pressed,
                    last.shoot_pressed,
                ),
                (
                    &mut current.grab_just_pressed,
                    current.grab_pressed,
                    last.grab_pressed,
                ),
                (
                    &mut current.slide_just_pressed,
                    current.slide_pressed,
                    last.slide_pressed,
                ),
                (&mut current.just_moved, current.moving, last.moving),
            ] {
                *just_pressed = current_pressed && !last_pressed;
            }
        });

        self.last_controls = self.current_controls.clone();

        &self.current_controls
    }
}
