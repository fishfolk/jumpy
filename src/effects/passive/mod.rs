use std::collections::HashMap;

use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use hecs::{Entity, World};

mod turtle_shell;

use crate::player::PlayerEventKind;
use crate::{AnimatedSprite, AnimatedSpriteMetadata, PlayerEvent};

static mut PASSIVE_EFFECT_FUNCS: Option<HashMap<String, PassiveEffectFn>> = None;

unsafe fn get_passive_effects_map() -> &'static mut HashMap<String, PassiveEffectFn> {
    PASSIVE_EFFECT_FUNCS.get_or_insert(HashMap::new())
}

#[allow(dead_code)]
pub fn add_passive_effect(id: &str, f: PassiveEffectFn) {
    unsafe { get_passive_effects_map() }.insert(id.to_string(), f);
}

pub fn try_get_passive_effect(id: &str) -> Option<&PassiveEffectFn> {
    unsafe { get_passive_effects_map() }.get(id)
}

pub fn get_passive_effect(id: &str) -> &PassiveEffectFn {
    try_get_passive_effect(id).unwrap()
}

pub type PassiveEffectFn =
    fn(world: &mut World, player_entity: Entity, item_entity: Option<Entity>, event: PlayerEvent);

pub fn init_passive_effects() {
    let effects = unsafe { get_passive_effects_map() };

    effects.insert(
        turtle_shell::EFFECT_FUNCTION_ID.to_string(),
        turtle_shell::effect_function,
    );
}

pub struct PassiveEffectInstance {
    pub name: String,
    pub function: Option<PassiveEffectFn>,
    pub activated_on: Vec<PlayerEventKind>,
    pub sprite: Option<AnimatedSprite>,
    pub sprite_entity: Option<Entity>,
    pub particle_effect_id: Option<String>,
    pub event_particle_effect_id: Option<String>,
    pub blocks_damage: bool,
    pub uses: Option<u32>,
    pub item: Option<Entity>,
    pub use_cnt: u32,
    pub duration: Option<f32>,
    pub duration_timer: f32,
}

impl PassiveEffectInstance {
    pub fn new(item: Option<Entity>, meta: PassiveEffectMetadata) -> Self {
        let function = meta.function_id.map(|id| *get_passive_effect(&id));

        PassiveEffectInstance {
            name: meta.name,
            function,
            activated_on: meta.activated_on,
            sprite: meta.sprite.map(Into::into),
            sprite_entity: None,
            particle_effect_id: meta.particle_effect_id,
            event_particle_effect_id: meta.event_particle_effect_id,
            blocks_damage: meta.blocks_damage,
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
pub struct PassiveEffectMetadata {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_id: Option<String>,
    /// This specifies the player events that will trigger an activation of the event
    pub activated_on: Vec<PlayerEventKind>,
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

    /// An optional sprite to add to the player along with the effect
    #[serde(alias = "animation")]
    pub sprite: Option<AnimatedSpriteMetadata>,
}
