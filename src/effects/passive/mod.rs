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
    Shield(ShieldEffectOptions),
    MovementMultiplier(MovementMultiplierEffectOptions),
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ShieldEffectOptions {
    pub front: bool,
    pub back: bool,
}

impl Default for ShieldEffectOptions {
    fn default() -> Self {
        Self {
            front: true,
            back: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MovementMultiplierEffectOptions {
    pub x: f32,
    pub y: f32,
}

impl Default for MovementMultiplierEffectOptions {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

impl PassiveEffectKind {
    /// Returns true if this passive effect is activated on the given player event
    pub fn activated_on(&self, event: &PlayerEvent) -> bool {
        match self {
            PassiveEffectKind::Shield(_) => matches!(event, PlayerEvent::ReceiveDamage { .. }),
            PassiveEffectKind::MovementMultiplier(_) => false,
        }
    }

    /// Returns a funciton pointer to the player event handler for the this passive effect kind
    pub fn handle_player_event(
        &self,
        world: &mut World,
        player_entity: Entity,
        item_entity: Option<Entity>,
        event: PlayerEvent,
    ) {
        match self {
            PassiveEffectKind::Shield(opts) => {
                self.handle_shield(opts, world, player_entity, item_entity, event)
            }
            PassiveEffectKind::MovementMultiplier(_) => (),
        }
    }

    fn handle_shield(
        &self,
        opts: &ShieldEffectOptions,
        world: &mut World,
        player_entity: Entity,
        _item_entity: Option<Entity>,
        event: PlayerEvent,
    ) {
        if let PlayerEvent::ReceiveDamage { is_from_left, .. } = event {
            let player = world.get::<Player>(player_entity).unwrap();
            let mut events = world.get_mut::<PlayerEventQueue>(player_entity).unwrap();

            let damage_is_from_front = player.is_facing_left == is_from_left;
            let damage_is_from_back = !damage_is_from_front;

            if damage_is_from_front && opts.front || damage_is_from_back && opts.back {
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
    /// This is the particle effect that will be spawned, each time a player event triggers the
    /// effect action ( if any ).
    /// 
    /// For instance, the shield effect is triggered every time it blocks an attack.
    #[serde(
        default,
        rename = "event_particle_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub event_particle_effect_id: Option<String>,
    /// This is the amount of times the effect action ( if any ) can be triggered, before the effect
    /// is depleted.
    /// 
    /// For instance, the shield effect is triggered every time it blocks an attack, and setting a
    /// use-limit will expire the effect after it has blocked that many attacks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    /// This is the duration of the effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
}
