use super::*;

use bevy::reflect::{FromReflect, Reflect};

use crate::metadata::PlayerMeta;

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputs>()
            .register_type::<PlayerInputs>()
            .register_type::<PlayerInput>()
            .register_type::<PlayerControl>()
            .add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .add_system_to_stage(CoreStage::PreUpdate, update_user_input)
            .add_system_to_stage(FixedUpdateStage::PreUpdate, update_active_players)
            .add_system_to_stage(FixedUpdateStage::Last, reset_input);
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
#[reflect(Default)]
pub struct PlayerInputs {
    players: Vec<PlayerInput>,
}

impl Default for PlayerInputs {
    fn default() -> Self {
        Self {
            players: vec![default(); MAX_PLAYERS],
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
    pub selected_player: Handle<PlayerMeta>,
    /// The player control input
    pub control: PlayerControl,
    /// The player control input from the last fixed update
    pub previous_control: PlayerControl,
}

/// Player control input state
#[derive(Reflect, Default, Clone, Debug, FromReflect)]
#[reflect(Default)]
pub struct PlayerControl {
    move_direction: Vec2,

    jump_pressed: bool,
    jump_just_pressed: bool,

    shoot_pressed: bool,
    shoot_just_pressed: bool,

    grab_pressed: bool,
    grab_just_pressed: bool,

    slide_pressed: bool,
    slide_just_pressed: bool,
}

fn update_user_input(
    mut player_inputs: ResMut<PlayerInputs>,
    players: Query<(&PlayerIdx, &ActionState<PlayerAction>)>,
) {
    for (player_idx, action_state) in &players {
        let PlayerInput {
            control,
            previous_control,
            ..
        } = &mut player_inputs.players[player_idx.0];

        control.move_direction = action_state
            .axis_pair(PlayerAction::Move)
            .unwrap_or_default()
            .xy();

        if action_state.pressed(PlayerAction::Jump) {
            control.jump_pressed = true;
            control.jump_just_pressed = !previous_control.jump_pressed;
        }
        if action_state.pressed(PlayerAction::Grab) {
            control.grab_pressed = true;
            control.grab_just_pressed = !previous_control.grab_pressed;
        }
        if action_state.pressed(PlayerAction::Shoot) {
            control.shoot_pressed = true;
            control.shoot_just_pressed = !previous_control.shoot_pressed;
        }
        if action_state.pressed(PlayerAction::Slide) {
            control.slide_pressed = true;
            control.slide_just_pressed = !previous_control.slide_pressed;
        }
    }
}

/// Reset player inputs to prepare for the next update
fn reset_input(mut player_inputs: ResMut<PlayerInputs>) {
    for player in &mut player_inputs.players {
        player.previous_control = player.control.clone();
        player.control = default();
    }
}

/// Updates the [`active`][PlayerInput::active] status of players
fn update_active_players(mut player_inputs: ResMut<PlayerInputs>) {
    // For now, players one and to are always active
    player_inputs.players[0].active = true;
    player_inputs.players[1].active = true;
}
