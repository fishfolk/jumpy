use std::collections::HashMap;

use ff_core::prelude::*;

use serde::{Deserialize, Serialize};

use ff_core::ecs::{Entity, World};

use crate::player::{DamageDirection, PlayerEventKind};
use crate::PlayerEvent;

#[derive(Resource, Clone, Serialize, Deserialize)]
#[resource(name = "passive_effect", path_index = true, crate_name = "ff_core")]
pub struct PassiveEffectMetadata {
    pub id: String,
    pub name: String,
    /// This holds particle effects that will be spawned when the effect begins.
    #[serde(
        default,
        rename = "particle_effects",
        skip_serializing_if = "Vec::is_empty"
    )]
    pub particle_effect_ids: Vec<String>,
    /// This determines if damage should be blocked by the effect and how this will be handled
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub damage_block: Option<PassiveEffectDamageBlock>,
    /// This is the amount of times the coroutine can be called, before the effect is depleted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    /// This is the duration of the effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
    /// If defined, this factor will be applied to the affected players move speed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub move_speed_factor: Option<f32>,
    /// If defined, this factor will be applied to the affected players jump force
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub jump_force_factor: Option<f32>,
    /// If defined, this factor will be applied to the affected players slide speed
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slide_speed_factor: Option<f32>,
    /// If defined, this factor will be applied to the affected players incapacitation duration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incapacitation_duration_factor: Option<f32>,
    /// If defined, this factor will be applied to the affected players float gravity
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub float_gravity_factor: Option<f32>,
    #[serde(
        default,
        rename = "on_begin_function",
        skip_serializing_if = "Option::is_none"
    )]
    pub on_begin_function_id: Option<String>,
    #[serde(
        default,
        rename = "on_event_functions",
        skip_serializing_if = "HashMap::is_empty"
    )]
    pub on_event_function_ids: HashMap<PlayerEventKind, Vec<String>>,
    #[serde(
        default,
        rename = "on_begin_function",
        skip_serializing_if = "Option::is_none"
    )]
    pub on_end_function_id: Option<String>,
}

static mut PASSIVE_EFFECT_FUNCS: Option<HashMap<String, PassiveEffectFn>> = None;

unsafe fn get_passive_effect_fn_map() -> &'static mut HashMap<String, PassiveEffectFn> {
    PASSIVE_EFFECT_FUNCS.get_or_insert(HashMap::new())
}

#[allow(dead_code)]
fn add_passive_effect_fn(id: &str, f: PassiveEffectFn) {
    unsafe { get_passive_effect_fn_map() }.insert(id.to_string(), f);
}

fn try_get_passive_effect_fn(id: &str) -> Option<&PassiveEffectFn> {
    unsafe { get_passive_effect_fn_map() }.get(id)
}

fn get_passive_effect_fn(id: &str) -> &PassiveEffectFn {
    try_get_passive_effect_fn(id).unwrap()
}

pub type PassiveEffectFn = fn(
    world: &mut World,
    player_entity: Entity,
    item_entity: Option<Entity>,
    event: Option<PlayerEvent>,
);

pub fn init_passive_effects() {
    let _effects = unsafe { get_passive_effect_fn_map() };
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PassiveEffectDamageBlockKind {
    Normal,
    IncrementUses,
    EndEffect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveEffectDamageBlock {
    pub kind: PassiveEffectDamageBlockKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub direction: Option<DamageDirection>,
}

pub struct PassiveEffect {
    pub id: String,
    pub name: String,
    pub damage_block: Option<PassiveEffectDamageBlock>,
    pub uses: Option<u32>,
    pub use_cnt: u32,
    pub duration: Option<f32>,
    pub duration_timer: f32,
    pub move_speed_factor: Option<f32>,
    pub jump_force_factor: Option<f32>,
    pub slide_speed_factor: Option<f32>,
    pub on_begin_fn: Option<PassiveEffectFn>,
    pub on_event_fn: HashMap<PlayerEventKind, Vec<PassiveEffectFn>>,
    pub on_end_fn: Option<PassiveEffectFn>,
    pub item: Option<Entity>,
    pub should_begin: bool,
    pub should_end: bool,
    pub should_remove: bool,
}

impl PassiveEffect {
    pub fn new(item: Option<Entity>, meta: PassiveEffectMetadata) -> Self {
        let on_begin_fn = meta
            .on_begin_function_id
            .map(|id| *get_passive_effect_fn(&id));

        let on_event_fn =
            HashMap::from_iter(meta.on_event_function_ids.into_iter().map(|(event, ids)| {
                let funcs = ids
                    .into_iter()
                    .map(|id| *get_passive_effect_fn(&id))
                    .collect::<Vec<_>>();
                (event, funcs)
            }));

        let on_end_fn = meta
            .on_end_function_id
            .map(|id| *get_passive_effect_fn(&id));

        PassiveEffect {
            id: meta.id,
            name: meta.name,
            on_begin_fn,
            on_event_fn,
            on_end_fn,
            damage_block: meta.damage_block,
            uses: meta.uses,
            item,
            use_cnt: 0,
            duration: meta.duration,
            duration_timer: 0.0,
            move_speed_factor: meta.move_speed_factor,
            jump_force_factor: meta.jump_force_factor,
            slide_speed_factor: meta.slide_speed_factor,
            should_begin: true,
            should_end: false,
            should_remove: false,
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
