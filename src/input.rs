use crate::{
    prelude::*,
    settings::{InputKind, PlayerControlMapping, PlayerControlSetting, Settings},
};

pub fn game_plugin(game: &mut Game) {
    game.systems.add_startup_system(load_controler_mapping);
    game.systems.add_before_system(collect_player_input);
}

// Startup system to load game control mapping resource from the storage and insert the player input
// collector.
fn load_controler_mapping(game: &mut Game) {
    let control_mapping = {
        let storage = game.shared_resource::<Storage>().unwrap();
        storage.get::<Settings>().unwrap().player_controls.clone()
    };
    game.insert_shared_resource(control_mapping);
    game.insert_shared_resource(PlayerInputCollector::default());
}

/// Game system that takes the raw input events and converts it to player controls based on the
/// player input map.
fn collect_player_input(game: &mut Game) {
    let controls = {
        let mut collector = game.shared_resource_mut::<PlayerInputCollector>().unwrap();
        let mapping = game.shared_resource::<PlayerControlMapping>().unwrap();
        let keyboard = game.shared_resource::<KeyboardInputs>().unwrap();
        let gamepad = game.shared_resource::<GamepadInputs>().unwrap();
        collector.update(&mapping, &keyboard, &gamepad);
        GlobalPlayerControls(collector.get().clone().into_iter().collect())
    };
    game.insert_shared_resource(controls);
}

#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
#[repr(C)]
pub struct GlobalPlayerControls(SVec<PlayerControl>);

/// Player control input state
#[derive(HasSchema, Default, Clone, Debug)]
#[repr(C)]
pub struct PlayerControl {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
    pub move_direction: Vec2,
    pub just_moved: bool,
    pub moving: bool,

    pub menu_back_pressed: bool,
    pub menu_back_just_pressed: bool,
    pub menu_confirm_pressed: bool,
    pub menu_confirm_just_pressed: bool,
    pub escape_pressed: bool,
    pub escape_just_pressed: bool,

    pub pause_pressed: bool,
    pub pause_just_pressed: bool,

    pub jump_pressed: bool,
    pub jump_just_pressed: bool,

    pub shoot_pressed: bool,
    pub shoot_just_pressed: bool,

    pub grab_pressed: bool,
    pub grab_just_pressed: bool,

    pub slide_pressed: bool,
    pub slide_just_pressed: bool,
}

#[derive(Default, HasSchema, Clone)]
pub struct PlayerInputCollector {
    last_controls: [PlayerControl; MAX_PLAYERS],
    current_controls: [PlayerControl; MAX_PLAYERS],
}

impl PlayerInputCollector {
    /// Update the internal state with new inputs. This must be called every render frame with the
    /// input events.
    pub fn update(
        &mut self,
        mapping: &crate::settings::PlayerControlMapping,
        keyboard: &KeyboardInputs,
        gamepad: &GamepadInputs,
    ) {
        // Helper to get the value of the given input type for the given player.
        let get_input_value = |input_map: &InputKind, player_idx: usize| {
            match input_map {
                crate::settings::InputKind::None => (),
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
                    (&mut control.pause_pressed, &mapping.pause),
                    (&mut control.jump_pressed, &mapping.jump),
                    (&mut control.grab_pressed, &mapping.grab),
                    (&mut control.shoot_pressed, &mapping.shoot),
                    (&mut control.slide_pressed, &mapping.slide),
                    (&mut control.menu_back_pressed, &mapping.menu_back),
                    (&mut control.menu_confirm_pressed, &mapping.menu_confirm),
                    (&mut control.escape_pressed, &mapping.escape),
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
                    &mut current.pause_just_pressed,
                    current.pause_pressed,
                    last.pause_pressed,
                ),
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
                (
                    &mut current.menu_back_just_pressed,
                    current.menu_back_pressed,
                    last.menu_back_pressed,
                ),
                (
                    &mut current.menu_confirm_just_pressed,
                    current.menu_confirm_pressed,
                    last.menu_confirm_pressed,
                ),
                (
                    &mut current.escape_just_pressed,
                    current.escape_pressed,
                    last.escape_pressed,
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
