use crate::{prelude::*, MAX_PLAYERS};

pub fn install(session: &mut GameSession) {
    session.world.init_resource::<PlayerInputs>();
}

/// The inputs for each player in this simulation frame.
#[derive(Clone, Debug, TypeUlid)]
#[ulid = "01GP233N26N8DQAAS1WDGYM14X"]
pub struct PlayerInputs {
    pub players: Vec<PlayerInput>,
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
#[derive(Default, Clone, Debug, TypeUlid)]
#[ulid = "01GP2356AYJ5NW8GJ3WF0TCCY3"]
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
#[derive(Default, Clone, Debug)]
#[repr(C)]
pub struct PlayerControl {
    pub move_direction: Vec2,
    pub just_moved: bool,
    pub moving: bool,

    pub jump_pressed: bool,
    pub jump_just_pressed: bool,

    pub shoot_pressed: bool,
    pub shoot_just_pressed: bool,

    pub grab_pressed: bool,
    pub grab_just_pressed: bool,

    pub slide_pressed: bool,
    pub slide_just_pressed: bool,
}
