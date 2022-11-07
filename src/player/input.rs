use super::*;

use bevy::reflect::{FromReflect, Reflect};
use ggrs::{InputStatus, PlayerHandle};
use numquant::{IntRange, Quantized};

use crate::metadata::PlayerMeta;

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputs>()
            .register_type::<PlayerInputs>()
            .register_type::<PlayerInput>()
            .register_type::<Vec<PlayerInput>>()
            .register_type::<PlayerControl>()
            .add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .extend_rollback_schedule(|schedule| {
                schedule.add_system_to_stage(RollbackStage::PreUpdate, update_user_input);
                // .add_system_to_stage(FixedUpdateStage::Last, reset_input);
            });
    }
}

/// The GGRS input system
pub fn input_system(
    player_handle: In<PlayerHandle>,
    players: Query<(&PlayerIdx, &ActionState<PlayerAction>)>,
) -> DensePlayerControl {
    if let Some((_, action_state)) = players.iter().find(|(idx, ..)| idx.0 == player_handle.0) {
        let mut control = DensePlayerControl(0);

        control.set_move_direction(DenseMoveDirection(
            action_state
                .axis_pair(PlayerAction::Move)
                .unwrap_or_default()
                .xy(),
        ));

        control.set_jump_pressed(action_state.pressed(PlayerAction::Jump));
        control.set_shoot_pressed(action_state.pressed(PlayerAction::Shoot));
        control.set_slide_pressed(action_state.pressed(PlayerAction::Slide));
        control.set_grab_pressed(action_state.pressed(PlayerAction::Grab));

        control
    } else {
        warn!("Couldn't find player input");
        DensePlayerControl(0)
    }
}

/// The control inputs that a player may make
#[derive(Debug, Copy, Clone, Actionlike, Deserialize, Eq, PartialEq, Hash)]
pub enum PlayerAction {
    Move,
    Jump,
    Shoot,
    Grab,
    Slide,
}

#[derive(Reflect, Clone, Debug)]
#[reflect(Default, Resource)]
pub struct PlayerInputs {
    pub players: Vec<PlayerInput>,
    // /// This field indicates whether or not the user input has been updated since the last run of
    // /// the `reset_input` system.
    // pub has_updated: bool,
}

impl Default for PlayerInputs {
    fn default() -> Self {
        Self {
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
    #[derive(bytemuck::Pod, bytemuck::Zeroable, Copy, Clone, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct DensePlayerControl(u16);
    impl Debug;
    jump_pressed, set_jump_pressed: 0;
    shoot_pressed, set_shoot_pressed: 1;
    grab_pressed, set_grab_pressed: 2;
    slide_pressed, set_slide_pressed: 3;
    from into DenseMoveDirection, move_direction, set_move_direction: 16, 4;
}

/// A newtype around [`Vec2`] that implements [`From<u16>`] and [`Into<u16>`] as a way to compress
/// user stick input for use in [`DensePlayerControl`].
#[derive(Debug, Deref, DerefMut)]
struct DenseMoveDirection(pub Vec2);

/// This is the specific [`Quantized`] type that we use to represent movement directions in
/// [`DenseMoveDirection`].
type MoveDirQuant = Quantized<IntRange<u16, 0b111111, -1, 1>>;

impl From<u16> for DenseMoveDirection {
    fn from(bits: u16) -> Self {
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

impl From<DenseMoveDirection> for u16 {
    fn from(dir: DenseMoveDirection) -> Self {
        let x_bits = MoveDirQuant::from_f32(dir.x).raw();
        let y_bits = MoveDirQuant::from_f32(dir.y).raw();

        x_bits | (y_bits << 6)
    }
}

/// Updates the [`PlayerInputs`] resource from input collected from GGRS.
fn update_user_input(
    inputs: Res<Vec<(DensePlayerControl, InputStatus)>>,
    mut player_inputs: ResMut<PlayerInputs>,
) {
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

        if input.jump_pressed() {
            control.jump_pressed = true;
            control.jump_just_pressed = !previous_control.jump_pressed;
        }
        if input.grab_pressed() {
            control.grab_pressed = true;
            control.grab_just_pressed = !previous_control.grab_pressed;
        }
        if input.shoot_pressed() {
            control.shoot_pressed = true;
            control.shoot_just_pressed = !previous_control.shoot_pressed;
        }
        if input.slide_pressed() {
            control.slide_pressed = true;
            control.slide_just_pressed = !previous_control.slide_pressed;
        }
    }

    // player_inputs.has_updated = true;
}

// /// Reset player inputs to prepare for the next update
// fn reset_input(mut player_inputs: ResMut<PlayerInputs>) {
//     if player_inputs.has_updated {
//         for player in &mut player_inputs.players {
//             player.previous_control = player.control.clone();
//             player.control = default();
//         }

//         player_inputs.has_updated = false;
//     }
// }
