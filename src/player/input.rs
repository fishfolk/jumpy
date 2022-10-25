use super::*;

use bevy::reflect::{FromReflect, Reflect};

use crate::{metadata::PlayerMeta, networking::proto::NetClientMatchInfo};

pub struct PlayerInputPlugin;

impl Plugin for PlayerInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInputs>()
            .register_type::<PlayerInputs>()
            .register_type::<PlayerInput>()
            .register_type::<Vec<PlayerInput>>()
            .register_type::<PlayerControl>()
            .add_plugin(InputManagerPlugin::<PlayerAction>::default())
            .add_system_to_stage(CoreStage::PreUpdate, update_user_input)
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
#[reflect(Default, Resource)]
pub struct PlayerInputs {
    pub players: Vec<PlayerInput>,

    /// This field indicates whether or not the user input has been updated since the last run of
    /// the `reset_input` system.
    pub has_updated: bool,
}

impl Default for PlayerInputs {
    fn default() -> Self {
        Self {
            players: vec![default(); MAX_PLAYERS],
            has_updated: false,
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
#[derive(Reflect, Default, Clone, Debug, FromReflect)]
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

fn update_user_input(
    mut player_inputs: ResMut<PlayerInputs>,
    players: Query<(&PlayerIdx, &ActionState<PlayerAction>)>,
    client_match_info: Option<Res<NetClientMatchInfo>>,
) {
    for (player_idx, action_state) in &players {
        let actual_player_idx = if let Some(match_info) = &client_match_info {
            match_info.player_idx
        } else {
            player_idx.0
        };

        let PlayerInput {
            control,
            previous_control,
            ..
        } = &mut player_inputs.players[actual_player_idx];

        control.moving = action_state.pressed(PlayerAction::Move);
        control.just_moved = control.moving && !previous_control.moving;
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

    player_inputs.has_updated = true;
}

/// Reset player inputs to prepare for the next update
fn reset_input(mut player_inputs: ResMut<PlayerInputs>) {
    if player_inputs.has_updated {
        for player in &mut player_inputs.players {
            player.previous_control = player.control.clone();
            player.control = default();
        }

        player_inputs.has_updated = false;
    }
}
