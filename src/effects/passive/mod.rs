use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use hecs::{Entity, World};

use crate::{
    player::{Player, PlayerEventQueue},
    PlayerEvent,
};

pub struct PassiveEffectInstance {
    pub name: String,
    pub kind: Option<PassiveEffectKind>,
    pub particle_effect_id: Option<String>,
    pub event_particle_effect_id: Option<String>,
    pub uses: Option<u32>,
    pub item: Option<Entity>,
    pub use_cnt: u32,
    pub duration: Option<f32>,
    pub duration_timer: f32,
}

impl PassiveEffectInstance {
    pub fn new(item: Option<Entity>, meta: PassiveEffectMetadata) -> Self {
        PassiveEffectInstance {
            name: meta.name,
            kind: meta.kind,
            particle_effect_id: meta.particle_effect_id,
            event_particle_effect_id: meta.event_particle_effect_id,
            uses: meta.uses,
            item,
            use_cnt: 0,
            duration: meta.duration,
            duration_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        self.duration_timer += dt;
    }

    pub fn is_depleted(&self) -> bool {
        if let Some(duration) = self.duration {
            if self.duration_timer >= duration {
                return true;
            }
        }

        if let Some(uses) = self.uses {
            if self.use_cnt >= uses {
                return true;
            }
        }

        false
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PassiveEffectKind {
    TurtleShell,
}

impl PassiveEffectKind {
    /// Returns true if this passive effect is activated on the given player event
    pub fn activated_on(&self, event: &PlayerEvent) -> bool {
        match self {
            PassiveEffectKind::TurtleShell => matches!(event, PlayerEvent::ReceiveDamage { .. }),
        }
    }

    /// Returns a funciton pointer to the player event handler for the this passive effect kind
    pub fn player_event_handler(&self) -> fn(&mut World, Entity, Option<Entity>, PlayerEvent) {
        match self {
            PassiveEffectKind::TurtleShell => Self::handle_turtle_shell_player_event,
        }
    }

    fn handle_turtle_shell_player_event(
        world: &mut World,
        player_entity: Entity,
        _item_entity: Option<Entity>,
        event: PlayerEvent,
    ) {
        if let PlayerEvent::ReceiveDamage { is_from_left, .. } = event {
            let player = world.get::<Player>(player_entity).unwrap();
            let mut events = world.get_mut::<PlayerEventQueue>(player_entity).unwrap();

            if player.is_facing_left != is_from_left {
                events
                    .queue
                    .push(PlayerEvent::DamageBlocked { is_from_left });
            }
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PassiveEffectMetadata {
    pub name: String,
    #[serde(default, flatten, skip_serializing_if = "Option::is_none")]
    pub kind: Option<PassiveEffectKind>,
    /// This is the particle effect that will be spawned when the effect become active.
    #[serde(
        default,
        rename = "particle_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub particle_effect_id: Option<String>,
    /// This is the particle effect that will be spawned, each time a player event leads to the
    /// effect coroutine being called.
    #[serde(
        default,
        rename = "event_particle_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub event_particle_effect_id: Option<String>,
    /// If this is true damage will be blocked on a player that has the item equipped
    #[serde(default)]
    pub blocks_damage: bool,
    /// This is the amount of times the coroutine can be called, before the effect is depleted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    /// This is the duration of the effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
}
