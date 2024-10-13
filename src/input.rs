use std::sync::RwLock;

use crate::{
    prelude::*,
    settings::{InputKind, PlayerControlMapping, Settings},
};

#[cfg(not(target_arch = "wasm32"))]
use bones_framework::networking::{input::NetworkPlayerControl, proto::DenseMoveDirection};
use strum::EnumIter;

fn _gamepad_axis_lefty(event: &GamepadEvent) -> Option<f32> {
    match event {
        GamepadEvent::Axis(GamepadAxisEvent {
            axis: GamepadAxis::LeftStickY,
            gamepad: 0,
            value,
            ..
        }) => Some(*value),
        _ => None,
    }
}

fn _gamepad_button_south(event: &GamepadEvent) -> Option<f32> {
    match event {
        GamepadEvent::Axis(GamepadAxisEvent {
            axis: GamepadAxis::LeftStickY,
            gamepad: 0,
            value,
            ..
        }) => Some(*value),
        _ => None,
    }
}

pub fn game_plugin(game: &mut Game) {
    game.systems.add_startup_system(load_controler_mapping);
    game.insert_shared_resource(EguiInputHook::new(handle_egui_input));
    game.init_shared_resource::<GlobalPlayerControls>();
    game.init_shared_resource::<PlayerInputCollector>();
    game.init_shared_resource::<PreviousGamepadInputs>();
}

// Startup system to load game control mapping resource from the storage and insert the player input
// collector.
fn load_controler_mapping(game: &mut Game) {
    let control_mapping = {
        let storage = game.shared_resource::<Storage>().unwrap();
        storage.get::<Settings>().unwrap().player_controls.clone()
    };
    game.insert_shared_resource(control_mapping);
}

fn collect_player_controls(game: &mut Game) {
    let controls = 'controls: {
        let mut collector = game.shared_resource_mut::<PlayerInputCollector>().unwrap();
        let Some(mapping) = game.shared_resource::<PlayerControlMapping>() else {
            break 'controls default();
        };
        let keyboard = game.shared_resource::<KeyboardInputs>().unwrap();
        let gamepad = game.shared_resource::<GamepadInputs>().unwrap();
        collector.apply_inputs(&mapping, &keyboard, &gamepad);
        collector.update_just_pressed();
        collector.advance_frame();
        GlobalPlayerControls(
            collector
                .get_current_controls()
                .clone()
                .into_iter()
                .collect(),
        )
    };
    game.insert_shared_resource(controls);
}

/// Settings to configure the [`handle_egui_input`] system
#[derive(Default, Debug, Clone, Copy)]
pub struct EguiInputSettings {
    /// If set to `true`, then all keyboard inputs will be sucked into a black hole so that egui
    /// doesn't read them.
    pub disable_keyboard_input: bool,
    /// If set to `true`, gamepad inputs will not be converted to egui inputs for menu navigation.
    pub disable_gamepad_input: bool,
}

#[derive(Clone, Deref, DerefMut, HasSchema)]
struct PreviousGamepadInputs(Arc<RwLock<[GamepadControl; MAX_PLAYERS as usize]>>);

impl Default for PreviousGamepadInputs {
    fn default() -> Self {
        Self(Arc::new(RwLock::new([default(); MAX_PLAYERS as usize])))
    }
}

#[derive(Clone, Copy, Debug, Default, HasSchema)]
struct GamepadControl {
    pub menu_confirm: bool,
    pub menu_back: bool,
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
    pub moving: bool,
}

/// Game system that takes the raw input events and converts it to player controls based on the
/// player input map.
pub fn handle_egui_input(game: &mut Game, egui_input: &mut egui::RawInput) {
    // We collect the global player controls here in the egui input hoook so that it will be
    // available immediately to egui, and then available to the rest of the systems that run after.
    collect_player_controls(game);

    let ctx = game.shared_resource::<EguiCtx>().unwrap();
    let settings = ctx.get_state::<EguiInputSettings>();
    let events = &mut egui_input.events;

    // Remove keyboard events if disabled.
    if settings.disable_keyboard_input {
        events.retain(|x| !matches!(x, egui::Event::Key { .. }));
    }

    // Forward gamepad events to egui if not disabled.
    'forward_gamepad_inputs: {
        if settings.disable_gamepad_input {
            break 'forward_gamepad_inputs;
        }

        let Some(mapping) = &game.shared_resource::<PlayerControlMapping>() else {
            break 'forward_gamepad_inputs;
        };
        let mapping = &mapping.gamepad;

        let previous_gamepad_controls =
            Arc::clone(&game.shared_resource::<PreviousGamepadInputs>().unwrap());
        let mut previous_gamepad_controls = previous_gamepad_controls.write().unwrap();
        let gamepad = game.shared_resource::<GamepadInputs>().unwrap();

        let push_key = |events: &mut Vec<egui::Event>, key| {
            events.push(egui::Event::Key {
                key,
                pressed: true,
                repeat: false,
                modifiers: default(),
            });
        };

        for (gamepad_idx, previous_control) in previous_gamepad_controls.iter_mut().enumerate() {
            let gamepad_idx = gamepad_idx as u32;

            // let ControlSource::Gamepad(gamepad_idx) = source else {
            //     continue;
            // };

            let mut current_control = *previous_control;

            let mapping_is_active = |input_map: InputKind| match input_map {
                InputKind::Button(mapped_button) => {
                    if gamepad_idx == 0 && mapped_button == GamepadButton::South {
                        for value in gamepad
                            .gamepad_events
                            .iter()
                            .filter_map(_gamepad_button_south)
                        {
                            info!(?value, "SOUTH");
                        }
                    }
                    for input in gamepad.gamepad_events.iter().rev() {
                        if let GamepadEvent::Button(e) = input {
                            if e.button == mapped_button && e.gamepad == gamepad_idx {
                                let value = if e.value < 0.1 { 0.0 } else { e.value };
                                return Some(value);
                            }
                        }
                    }
                    None
                }
                InputKind::AxisPositive(mapped_axis) => {
                    for input in gamepad.gamepad_events.iter().rev() {
                        if let GamepadEvent::Axis(e) = input {
                            if e.axis == mapped_axis && e.gamepad == gamepad_idx {
                                let value = if e.value < 0.1 { 0.0 } else { e.value };
                                return Some(value);
                            }
                        }
                    }
                    None
                }
                InputKind::AxisNegative(mapped_axis) => {
                    // if gamepad_idx == 0 && mapped_axis == GamepadAxis::LeftStickY {
                    //     for value in gamepad
                    //         .gamepad_events
                    //         .iter()
                    //         .filter_map(_gamepad_leftstick_y_value)
                    //         .map(|value| if value > -0.1 { 0.0 } else { value })
                    //     {
                    //         info!(?value, "down event");
                    //     }
                    // }
                    for input in gamepad.gamepad_events.iter().rev() {
                        if let GamepadEvent::Axis(e) = input {
                            if e.axis == mapped_axis && e.gamepad == gamepad_idx {
                                let value = if e.value > -0.1 { 0.0 } else { e.value };
                                return Some(value);
                            }
                        }
                    }
                    None
                }
                _ => None,
            };

            //
            // Collect button press inputs
            //

            for (button_pressed, button_map) in [
                (&mut current_control.menu_confirm, &mapping.menu_confirm),
                // (&mut current_control.menu_back, &mapping.menu_back),
            ] {
                // TODO: remove `clone()` when this type implements `Copy`
                if let Some(value) = mapping_is_active(button_map.clone()) {
                    info!(value = ?Some(value), "confirm mapping");
                    *button_pressed = value.abs() > 0.0;
                }
            }

            // if gamepad_idx == 0 {
            //     info!(
            //         previous = previous_control.menu_confirm,
            //         current = current_control.menu_confirm
            //     );
            // }

            //
            // Collect joystick inputs
            //

            // helper for merging two inputs (like dpad + joystick for example) allowing multiple bindings
            // for same control
            let merge_inputs = |input1: &InputKind, input2: &InputKind| {
                // TODO: remove `clone()` when this type implements `Copy`
                match (
                    mapping_is_active(input1.clone()),
                    mapping_is_active(input2.clone()),
                ) {
                    // Both inputs have a value and the first is zero -- use the second value
                    (Some(0.0), Some(value2)) => Some(value2),
                    // First input has a non-zero value -- use the first value
                    (Some(value1), _) => Some(value1),
                    // First input has no value -- use the second
                    (None, value2) => value2,
                }
                .map(f32::abs)
            };

            if let Some(left) = merge_inputs(&mapping.movement.left, &mapping.movement_alt.left) {
                current_control.left = left;
            }
            if let Some(right) = merge_inputs(&mapping.movement.right, &mapping.movement_alt.right)
            {
                current_control.right = right;
            }
            if let Some(up) = merge_inputs(&mapping.movement.up, &mapping.movement_alt.up) {
                current_control.up = up;
            }
            if let Some(down) = merge_inputs(&mapping.movement.down, &mapping.movement_alt.down) {
                current_control.down = down;
            }
            let move_direction = vec2(
                current_control.right - current_control.left,
                current_control.up - current_control.down,
            );
            current_control.moving = move_direction.length_squared() > 0.01;

            //
            // Forward collected inputs to egui
            //

            // info!(
            //     prev = previous_control.menu_confirm,
            //     curr = current_control.menu_confirm,
            //     "menu confirm"
            // );
            if current_control.menu_confirm && !previous_control.menu_confirm {
                // push_key(events, egui::Key::Enter);
                info!("ENTER <<<<<<<<<<<<<<<<<<<<<<<");
            }
            if current_control.menu_back && !previous_control.menu_back {
                push_key(events, egui::Key::Escape);
            }

            if current_control.moving && !previous_control.moving {
                if move_direction.x < -0.1 {
                    push_key(events, egui::Key::ArrowLeft);
                }
                if move_direction.x > 0.1 {
                    push_key(events, egui::Key::ArrowRight);
                }
                if move_direction.y > 0.1 {
                    push_key(events, egui::Key::ArrowUp);
                }
                if move_direction.y < -0.1 {
                    push_key(events, egui::Key::ArrowDown);
                }
            }

            *previous_control = current_control;
        }
    }
}

/// Resource containing the global player control inputs.
///
/// It is important to note that these controls are updated every system frame, and therefore
/// the `just_pressed` and `just_moved` flags are not accurate in the context of a fixed
/// update match loop. Matches with fixed updates have their own input resource.
///
/// This resource is used throughout the menu where the inputs are collected every frame, not
/// every fixed update.
#[derive(HasSchema, Clone, Default, Deref, DerefMut)]
pub struct GlobalPlayerControls(HashMap<ControlSource, PlayerControl>);

impl GlobalPlayerControls {
    /// Iterator over inputs that originated from gamepads.
    pub fn gamepads(&self) -> impl Iterator<Item = &PlayerControl> {
        self.iter().filter_map(|(source, control)| {
            matches!(source, ControlSource::Gamepad(_)).then_some(control)
        })
    }
}

/// The source of player control inputs
#[derive(Debug, Clone, Copy, Default, HasSchema, Hash, Eq, PartialEq, EnumIter)]
#[repr(C, u8)]
pub enum ControlSource {
    #[default]
    /// The first keyboard controls
    Keyboard1,
    /// The second keyboard controls
    Keyboard2,
    /// A gamepad control with the given index
    Gamepad(u32),
}

/// Player control input state
#[derive(HasSchema, Default, Clone, Copy, Debug)]
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
    pub menu_start_pressed: bool,
    pub menu_start_just_pressed: bool,

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

    pub ragdoll_pressed: bool,
    pub ragdoll_just_pressed: bool,
}

#[derive(HasSchema, Clone)]
pub struct PlayerInputCollector {
    current_controls: HashMap<ControlSource, PlayerControl>,
    last_controls: HashMap<ControlSource, PlayerControl>,
}

impl PlayerInputCollector {
    pub fn get_current_controls(&self) -> &HashMap<ControlSource, PlayerControl> {
        &self.current_controls
    }
}

impl Default for PlayerInputCollector {
    fn default() -> Self {
        let def_controls = || {
            let mut m = HashMap::default();
            // We always have the keyboard controls "plugged in"
            m.insert(ControlSource::Keyboard1, default());
            m.insert(ControlSource::Keyboard2, default());
            for i in 0..MAX_PLAYERS {
                m.insert(ControlSource::Gamepad(i), default());
            }
            m
        };
        Self {
            current_controls: def_controls(),
            last_controls: def_controls(),
        }
    }
}

impl<'a>
    bones_framework::input::InputCollector<'a, PlayerControlMapping, ControlSource, PlayerControl>
    for PlayerInputCollector
{
    fn update_just_pressed(&mut self) {
        self.current_controls
            .iter_mut()
            .for_each(|(source, current)| {
                let last = self.last_controls.entry(*source).or_default();

                current.move_direction =
                    vec2(current.right - current.left, current.up - current.down);
                current.moving = current.move_direction.length_squared() > 0.01;

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
                        &mut current.ragdoll_just_pressed,
                        current.ragdoll_pressed,
                        last.ragdoll_pressed,
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
                        &mut current.menu_start_just_pressed,
                        current.menu_start_pressed,
                        last.menu_start_pressed,
                    ),
                    (&mut current.just_moved, current.moving, last.moving),
                ] {
                    *just_pressed = current_pressed && !last_pressed;
                }
            });
    }

    fn advance_frame(&mut self) {
        self.last_controls = self.current_controls.clone();
    }

    /// Update the internal state with new inputs. This must be called every render frame with the
    /// input events.
    fn apply_inputs(
        &mut self,
        mapping: &PlayerControlMapping,
        keyboard: &KeyboardInputs,
        gamepad: &GamepadInputs,
    ) {
        // Helper to get the value of the given input type for the given player.
        let get_input_value = |input_map: &InputKind, control_source: &ControlSource| match (
            input_map,
            control_source,
        ) {
            (InputKind::Button(mapped_button), ControlSource::Gamepad(idx)) => {
                for input in gamepad.gamepad_events.iter().rev() {
                    if let GamepadEvent::Button(e) = input {
                        if &e.button == mapped_button && e.gamepad == *idx {
                            let value = if e.value < 0.1 { 0.0 } else { e.value };
                            return Some(value);
                        }
                    }
                }
                None
            }
            (InputKind::AxisPositive(mapped_axis), ControlSource::Gamepad(idx)) => {
                for input in gamepad.gamepad_events.iter().rev() {
                    if let GamepadEvent::Axis(e) = input {
                        if &e.axis == mapped_axis && e.gamepad == *idx {
                            let value = if e.value < 0.1 { 0.0 } else { e.value };
                            return Some(value);
                        }
                    }
                }
                None
            }
            (InputKind::AxisNegative(mapped_axis), ControlSource::Gamepad(idx)) => {
                for input in gamepad.gamepad_events.iter().rev() {
                    if let GamepadEvent::Axis(e) = input {
                        if &e.axis == mapped_axis && e.gamepad == *idx {
                            let value = if e.value > -0.1 { 0.0 } else { e.value };
                            return Some(value);
                        }
                    }
                }
                None
            }
            (
                InputKind::Keyboard(mapped_key),
                ControlSource::Keyboard1 | ControlSource::Keyboard2,
            ) => {
                for input in keyboard.key_events.iter().rev() {
                    if input.key_code.option() == Some(*mapped_key) {
                        return Some(if input.button_state.pressed() {
                            1.0
                        } else {
                            0.0
                        });
                    }
                }
                None
            }
            _ => None,
        };

        for (source, control) in self.current_controls.iter_mut() {
            let mapping = match source {
                ControlSource::Keyboard1 => &mapping.keyboard1,
                ControlSource::Keyboard2 => &mapping.keyboard2,
                ControlSource::Gamepad(_) => &mapping.gamepad,
            };

            for (button_pressed, button_map) in [
                (&mut control.pause_pressed, &mapping.pause),
                (&mut control.jump_pressed, &mapping.jump),
                (&mut control.grab_pressed, &mapping.grab),
                (&mut control.shoot_pressed, &mapping.shoot),
                (&mut control.slide_pressed, &mapping.slide),
                (&mut control.ragdoll_pressed, &mapping.ragdoll),
                (&mut control.menu_back_pressed, &mapping.menu_back),
                (&mut control.menu_confirm_pressed, &mapping.menu_confirm),
                (&mut control.menu_start_pressed, &mapping.menu_start),
            ] {
                if let Some(value) = get_input_value(button_map, source) {
                    *button_pressed = value > 0.0;
                }
            }

            // helper for merging two inputs (like dpad + joystick for example) allowing multiple bindings
            // for same control
            let merge_inputs = |input1: &InputKind, input2: &InputKind| -> Option<f32> {
                match (
                    get_input_value(input1, source),
                    get_input_value(input2, source),
                ) {
                    // Both inputs have a value and the first is zero -- use the second value
                    (Some(0.0), Some(value2)) => Some(value2),
                    // First input has a non-zero value -- use the first value
                    (Some(value1), _) => Some(value1),
                    // First input has no value -- use the second
                    (None, value2) => value2,
                }
                .map(f32::abs)
            };

            if let Some(left) = merge_inputs(&mapping.movement.left, &mapping.movement_alt.left) {
                control.left = left;
            }
            if let Some(right) = merge_inputs(&mapping.movement.right, &mapping.movement_alt.right)
            {
                control.right = right;
            }
            if let Some(up) = merge_inputs(&mapping.movement.up, &mapping.movement_alt.up) {
                control.up = up;
            }
            if let Some(down) = merge_inputs(&mapping.movement.down, &mapping.movement_alt.down) {
                control.down = down;
            }
        }
    }

    // TODO: Fix bones Trait definition, player_idx not relevant
    fn get_control(&self, _player_idx: usize, control_source: ControlSource) -> &PlayerControl {
        self.current_controls.get(&control_source).unwrap()
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl NetworkPlayerControl<DensePlayerControl> for PlayerControl {
    fn get_dense_input(&self) -> DensePlayerControl {
        let mut dense_control = DensePlayerControl::default();
        dense_control.set_jump_pressed(self.jump_pressed);
        dense_control.set_grab_pressed(self.grab_pressed);
        dense_control.set_slide_pressed(self.slide_pressed);
        dense_control.set_shoot_pressed(self.shoot_pressed);
        dense_control.set_ragdoll_pressed(self.ragdoll_pressed);
        dense_control.set_move_direction(proto::DenseMoveDirection(self.move_direction));
        dense_control
    }

    fn update_from_dense(&mut self, new_control: &DensePlayerControl) {
        let jump_pressed = new_control.jump_pressed();
        self.jump_just_pressed = jump_pressed && !self.jump_pressed;
        self.jump_pressed = jump_pressed;

        let grab_pressed = new_control.grab_pressed();
        self.grab_just_pressed = grab_pressed && !self.grab_pressed;
        self.grab_pressed = grab_pressed;

        let shoot_pressed = new_control.shoot_pressed();
        self.shoot_just_pressed = shoot_pressed && !self.shoot_pressed;
        self.shoot_pressed = shoot_pressed;

        let ragdoll_pressed = new_control.ragdoll_pressed();
        self.ragdoll_just_pressed = ragdoll_pressed && !self.ragdoll_pressed;
        self.ragdoll_pressed = ragdoll_pressed;

        let was_moving = self.move_direction.length_squared() > f32::MIN_POSITIVE;
        self.move_direction = new_control.move_direction().0;
        let is_moving = self.move_direction.length_squared() > f32::MIN_POSITIVE;
        self.just_moved = !was_moving && is_moving;
    }
}

#[cfg(not(target_arch = "wasm32"))]
bitfield::bitfield! {
    /// A player's controller inputs densely packed into a single u32.
    ///
    /// This is used when sending player inputs across the network.
    #[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, PartialEq, Eq)]//, Reflect)]
    #[repr(transparent)]
    pub struct DensePlayerControl(u32);
    impl Debug;
    pub jump_pressed, set_jump_pressed: 0;
    pub shoot_pressed, set_shoot_pressed: 1;
    pub grab_pressed, set_grab_pressed: 2;
    pub slide_pressed, set_slide_pressed: 3;
    pub ragdoll_pressed, set_ragdoll_pressed: 4;
    pub from into DenseMoveDirection, move_direction, set_move_direction: 16, 5;
}

#[cfg(not(target_arch = "wasm32"))]
impl Default for DensePlayerControl {
    fn default() -> Self {
        let mut control = Self(0);
        control.set_move_direction(default());
        control
    }
}

#[cfg(not(target_arch = "wasm32"))]
/// Used to implement input type config for bones networking.
pub struct NetworkInputConfig;

#[cfg(not(target_arch = "wasm32"))]
impl<'a> bones_framework::networking::input::NetworkInputConfig<'a> for NetworkInputConfig {
    type Dense = DensePlayerControl;
    type Control = PlayerControl;
    type PlayerControls = MatchInputs;
    type InputCollector = PlayerInputCollector;
}
