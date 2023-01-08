use super::*;

use bevy::reflect::{FromReflect, Reflect};
use bevy_ggrs::ggrs::{InputStatus, PlayerHandle};
use leafwing_input_manager::plugin::InputManagerSystem;
use numquant::{IntRange, Quantized};

use crate::metadata::PlayerMeta;

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputs>()
            .init_resource::<LocalPlayerInputBuffer>()
            .register_type::<PlayerInputs>()
            .register_type::<PlayerInput>()
            .register_type::<Vec<PlayerInput>>()
            .register_type::<PlayerControl>()
            .add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .add_system_to_stage(
                CoreStage::PreUpdate,
                update_input_buffer.after(InputManagerSystem::Update),
            )
            .add_system_to_stage(CoreStage::Last, clear_input_buffer)
            .extend_rollback_plugin(|plugin| plugin.register_rollback_type::<PlayerInputs>())
            .add_rollback_system(RollbackStage::Input, update_user_input);
    }
}

/// This is a resource that gets inserted every time the menu wants to modify the pause state of the
/// game.
///
/// The resource is removed every time it is read, so it will only be present in the world when
/// there is an intent to change the pause state.
#[derive(Resource)]
pub struct WantsGamePause(pub bool);

/// A buffer holding the player inputs until they are read by the game simulation.
#[derive(Reflect, Default, Resource)]
pub struct LocalPlayerInputBuffer {
    /// The buffers for each player. Non-local players will have empty buffers.
    pub players: [DensePlayerControl; MAX_PLAYERS],
    /// Indicates that the buffer has been read and should be reset at the end of the render frame.
    pub has_been_read: bool,
}

/// Update the player input buffer. This makes sure that if the frame rate exceeds the simulation
/// updates per second that any inputs pressed in between frames will be detected.
fn update_input_buffer(
    mut buffer: ResMut<LocalPlayerInputBuffer>,
    players: Query<(&PlayerIdx, &ActionState<PlayerAction>)>,
) {
    for (player_idx, action_state) in &players {
        let control = &mut buffer.players[player_idx.0];

        control.set_move_direction(DenseMoveDirection(
            action_state
                .axis_pair(PlayerAction::Move)
                .unwrap_or_default()
                .xy(),
        ));

        control
            .set_jump_pressed(action_state.pressed(PlayerAction::Jump) || control.jump_pressed());
        control.set_shoot_pressed(
            action_state.pressed(PlayerAction::Shoot) || control.shoot_pressed(),
        );
        control.set_slide_pressed(
            action_state.pressed(PlayerAction::Slide) || control.slide_pressed(),
        );
        control
            .set_grab_pressed(action_state.pressed(PlayerAction::Grab) || control.grab_pressed());
    }
}

/// Clear the input buffer if it has been read this frame
fn clear_input_buffer(mut buffer: ResMut<LocalPlayerInputBuffer>) {
    if buffer.has_been_read {
        *buffer = default()
    }
}

/// The GGRS input system
pub fn input_system(
    player_handle: In<PlayerHandle>,
    mut commands: Commands,
    mut buffer: ResMut<LocalPlayerInputBuffer>,
    wants_game_pause: Option<Res<WantsGamePause>>,
) -> DensePlayerControl {
    buffer.has_been_read = true;
    let mut input = buffer.players[player_handle.0];

    if let Some(wants_game_pause) = wants_game_pause {
        commands.remove_resource::<WantsGamePause>();
        input.set_wants_to_set_pause(true);
        input.set_pause_value(wants_game_pause.0);
    }

    input
}

/// The control inputs that a player may make.
#[derive(Debug, Copy, Clone, Actionlike, Deserialize, Eq, PartialEq, Hash)]
pub enum PlayerAction {
    Move,
    Jump,
    Shoot,
    Grab,
    Slide,
}

/// The inputs for each player in this simulation frame.
#[derive(Reflect, Clone, Debug, Component, Resource)]
#[reflect(Default, Resource)]
pub struct PlayerInputs {
    /// This will be `true` if _all_ of the inputs for all players for this frame have been
    /// confirmed ( so presumably will not be rolled back ).
    pub is_confirmed: bool,
    pub players: Vec<PlayerInput>,
}

impl Default for PlayerInputs {
    fn default() -> Self {
        Self {
            is_confirmed: false,
            players: vec![default(); MAX_PLAYERS],
            // has_updated: false,
        }
    }
}

/// Player input, not just controls, but also other status that comes from the player, such as the
/// selected player and whether the player is actually active.
#[derive(Reflect, Default, Clone, Debug, FromReflect)]
#[reflect(Default)]
pub struct PlayerInput {
    /// The player is currently "connected" and actively providing input.
    pub active: bool,
    /// This may be a null handle if a player hasn't been selected yet
    pub selected_player: AssetHandle<PlayerMeta>,
    /// The player control input
    pub control: PlayerControl,
    /// The player control input from the last fixed update
    pub previous_control: PlayerControl,
}

/// Player control input state
#[derive(Reflect, Default, Clone, Debug, FromReflect, Deserialize, Serialize)]
#[reflect(Default)]
pub struct PlayerControl {
    pub just_moved: bool,
    pub moving: bool,
    pub move_direction: Vec2,

    pub jump_pressed: bool,
    pub jump_just_pressed: bool,

    pub shoot_pressed: bool,
    pub shoot_just_pressed: bool,

    pub grab_pressed: bool,
    pub grab_just_pressed: bool,

    pub slide_pressed: bool,
    pub slide_just_pressed: bool,
}

bitfield::bitfield! {
    /// A player's controller inputs densely packed into a single u16.
    ///
    /// This is used when sending player inputs across the network.
    #[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, PartialEq, Eq, Reflect)]
    #[repr(transparent)]
    pub struct DensePlayerControl(u32);
    impl Debug;
    jump_pressed, set_jump_pressed: 0;
    shoot_pressed, set_shoot_pressed: 1;
    grab_pressed, set_grab_pressed: 2;
    slide_pressed, set_slide_pressed: 3;
    from into DenseMoveDirection, move_direction, set_move_direction: 15, 4;
    /// This bit will be set if this player wants to try and pause or un-pause the game
    wants_to_set_pause, set_wants_to_set_pause: 16;
    /// This value is only relevant if `wants_to_set_pause` is true, and it indicates whether the
    /// player wants to pause or unpause the game.
    pause_value, set_pause_value: 17;
}

impl Default for DensePlayerControl {
    fn default() -> Self {
        let mut control = Self(0);
        control.set_move_direction(default());
        control
    }
}

/// A newtype around [`Vec2`] that implements [`From<u16>`] and [`Into<u16>`] as a way to compress
/// user stick input for use in [`DensePlayerControl`].
#[derive(Debug, Deref, DerefMut, Default)]
struct DenseMoveDirection(pub Vec2);

/// This is the specific [`Quantized`] type that we use to represent movement directions in
/// [`DenseMoveDirection`].
type MoveDirQuant = Quantized<IntRange<u32, 0b111111, -1, 1>>;

impl From<u32> for DenseMoveDirection {
    fn from(bits: u32) -> Self {
        // maximum movement value representable, we use 6 bits to represent each movement direction.
        let max = 0b111111;
        // The first six bits represent the x movement
        let x_move_bits = bits & max;
        // The second six bits represents the y movement
        let y_move_bits = (bits >> 6) & max;

        // Round near-zero values to zero
        let mut x = MoveDirQuant::from_raw(x_move_bits).to_f32();
        if x.abs() < 0.02 {
            x = 0.0;
        }
        let mut y = MoveDirQuant::from_raw(y_move_bits).to_f32();
        if y.abs() < 0.02 {
            y = 0.0;
        }

        DenseMoveDirection(Vec2::new(x, y))
    }
}

impl From<DenseMoveDirection> for u32 {
    fn from(dir: DenseMoveDirection) -> Self {
        let x_bits = MoveDirQuant::from_f32(dir.x).raw();
        let y_bits = MoveDirQuant::from_f32(dir.y).raw();

        x_bits | (y_bits << 6)
    }
}

#[derive(Resource, Default, Deref, DerefMut, Clone, Debug)]
pub struct Inputs(pub Vec<(DensePlayerControl, InputStatus)>);

/// Updates the [`PlayerInputs`] resource from input collected from GGRS.
fn update_user_input(
    mut commands: Commands,
    inputs: Res<Inputs>,
    mut player_inputs: ResMut<PlayerInputs>,
) {
    player_inputs.is_confirmed = inputs
        .iter()
        .map(|x| x.1)
        .all(|x| x == InputStatus::Confirmed);

    let someone_wants_to_pause = inputs
        .iter()
        .any(|x| x.0.wants_to_set_pause() && x.0.pause_value());
    let someone_wants_to_unpause = inputs
        .iter()
        .any(|x| x.0.wants_to_set_pause() && !x.0.pause_value());

    if someone_wants_to_pause {
        commands.insert_resource(NextState(InGameState::Paused));
    } else if someone_wants_to_unpause {
        commands.insert_resource(NextState(InGameState::Playing));
    }

    for (player_idx, (input, _)) in inputs.iter().enumerate() {
        let PlayerInput {
            control,
            previous_control,
            ..
        } = &mut player_inputs.players[player_idx];

        let move_direction = input.move_direction();

        control.moving = move_direction.0 != Vec2::ZERO;
        control.just_moved = control.moving && !previous_control.moving;

        control.move_direction = move_direction.0;

        control.jump_pressed = input.jump_pressed();
        control.jump_just_pressed = control.jump_pressed && !previous_control.jump_pressed;

        control.shoot_pressed = input.shoot_pressed();
        control.shoot_just_pressed = control.shoot_pressed && !previous_control.shoot_pressed;

        control.grab_pressed = input.grab_pressed();
        control.grab_just_pressed = control.grab_pressed && !previous_control.grab_pressed;

        control.slide_pressed = input.slide_pressed();
        control.slide_just_pressed = control.slide_pressed && !previous_control.slide_pressed;

        *previous_control = control.clone();
    }
}
