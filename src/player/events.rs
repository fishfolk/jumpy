use hecs::{Entity, World};
use macroquad::time::get_frame_time;

use crate::player::{Player, PlayerState};
use serde::{Deserialize, Serialize};

#[derive(Default)]
pub struct PlayerEventQueue {
    pub queue: Vec<PlayerEvent>,
}

impl PlayerEventQueue {
    pub fn new() -> Self {
        PlayerEventQueue { queue: Vec::new() }
    }
}

#[derive(Clone)]
pub enum PlayerEvent {
    Update {
        dt: f32,
    },
    ReceiveDamage {
        is_from_left: bool,
        damage_from: Option<Entity>,
    },
    GiveDamage {
        damage_to: Option<Entity>,
    },
    DamageBlocked {
        is_from_left: bool,
    },
    Incapacitated {
        incapacitated_by: Option<Entity>,
    },
    Collision {
        is_new: bool,
        collision_with: Entity,
    },
}

/// This is used in JSON to specify which event types an effect should apply to
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayerEventKind {
    Update,
    ReceiveDamage,
    GiveDamage,
    DamageBlocked,
    Incapacitated,
    Collision,
}

impl From<&PlayerEvent> for PlayerEventKind {
    fn from(params: &PlayerEvent) -> Self {
        use PlayerEvent::*;

        match params {
            Update { .. } => Self::Update,
            ReceiveDamage { .. } => Self::ReceiveDamage,
            GiveDamage { .. } => Self::GiveDamage,
            DamageBlocked { .. } => Self::DamageBlocked,
            Incapacitated { .. } => Self::Incapacitated,
            Collision { .. } => Self::Collision,
        }
    }
}

pub fn update_player_events(world: &mut World) {
    for (_, (player, events)) in world.query_mut::<(&mut Player, &mut PlayerEventQueue)>() {
        let dt = get_frame_time();

        events.queue.push(PlayerEvent::Update { dt });

        let mut damage_blocked_left = false;
        let mut damage_blocked_right = false;

        for event in events.queue.iter() {
            if let PlayerEvent::DamageBlocked { is_from_left } = *event {
                damage_blocked_left = damage_blocked_left || is_from_left;
                damage_blocked_right = damage_blocked_right || !is_from_left;
            }
        }

        while let Some(event) = events.queue.pop() {
            if let PlayerEvent::ReceiveDamage { is_from_left, .. } = event {
                if (is_from_left && !damage_blocked_left)
                    || (!is_from_left && !damage_blocked_right)
                {
                    player.state = PlayerState::Dead;
                    player.damage_from_left = is_from_left;
                }
            }
        }
    }
}
