use crate::prelude::*;

#[derive(Default)]
pub struct PlayerInputCollector {
    last_controls: [PlayerControl; MAX_PLAYERS],
    current_controls: [PlayerControl; MAX_PLAYERS],
}

impl PlayerInputCollector {
    /// Update the internal state with new inputs. This must be called every render frame with the
    /// input events.
    pub fn update(&mut self, keyboard: &KeyboardInputs, _gamepad: &GamepadInputs) {
        let p1 = &mut self.current_controls[0];

        for event in &keyboard.key_events {
            if event.key_code == Some(KeyCode::Space) {
                p1.jump_pressed = event.button_state.pressed();
            }
        }
    }

    /// Get the player inputs for the next game simulation frame.
    pub fn get(&mut self) -> &[PlayerControl; MAX_PLAYERS] {
        (0..MAX_PLAYERS).for_each(|i| {
            let current = &mut self.current_controls[i];
            let last = &self.last_controls[i];

            current.jump_just_pressed = current.jump_pressed && !last.jump_pressed
        });

        self.last_controls = self.current_controls.clone();

        &self.current_controls
    }
}
