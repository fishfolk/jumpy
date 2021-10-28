use macroquad::experimental::scene::Handle;

use serde::{Deserialize, Serialize};

use crate::Player;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerEvent {
    Update,
    ReceiveDamage,
    GiveDamage,
    Incapacitated,
    Collision,
}

impl From<&PlayerEventParams> for PlayerEvent {
    fn from(params: &PlayerEventParams) -> Self {
        use PlayerEventParams::*;

        match params {
            Update { .. } => Self::Update,
            ReceiveDamage { .. } => Self::ReceiveDamage,
            GiveDamage { .. } => Self::GiveDamage,
            Incapacitated { .. } => Self::Incapacitated,
            Collision { .. } => Self::Collision,
        }
    }
}

/// This holds the parameters for each `PlayerEvent` variant
pub enum PlayerEventParams {
    Update {
        dt: f32,
    },
    ReceiveDamage {
        is_from_right: bool,
        damage_from: Option<Handle<Player>>,
    },
    GiveDamage {
        damage_to: Handle<Player>,
    },
    Incapacitated {
        incapacitated_by: Option<Handle<Player>>,
    },
    Collision {
        collision_with: Handle<Player>,
    },
}
