use macroquad::{
    experimental::{
        coroutines::{start_coroutine, Coroutine},
        scene::Handle,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

pub mod custom;

use crate::Player;

// static mut EQUIPMENT_EFFECTS: Option<HashMap<String, PassiveEffect>> = None;
//
// unsafe fn get_passive_effects_map() -> &'static mut HashMap<String, PassiveEffect> {
//     if EQUIPMENT_EFFECTS.is_none() {
//         EQUIPMENT_EFFECTS = Some(HashMap::new());
//     }
//
//     EQUIPMENT_EFFECTS.as_mut().unwrap()
// }
//
// #[allow(dead_code)]
// pub fn add_passive_effect_constructor(id: &str, effect: PassiveEffect) {
//     unsafe { get_passive_effects_map() }.insert(id.to_string(), effect);
// }
//
// pub fn get_passive_effect(id: &str) -> PassiveEffect {
//     unsafe { get_passive_effects_map() }.get(id).cloned().unwrap()
// }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveEffectParams {
    pub id: String,
    #[serde(
        default,
        rename = "particle_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub particle_effect_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
}

pub struct PassiveEffect {
    id: String,
    particle_effect_id: Option<String>,
    duration: Option<f32>,
    duration_timer: f32,
}

impl PassiveEffect {
    pub fn new(params: PassiveEffectParams) -> Self {
        PassiveEffect {
            id: params.id,
            particle_effect_id: params.particle_effect_id,
            duration: params.duration,
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

        false
    }

    pub fn on_player_update(&mut self, _player_handle: Handle<Player>, _dt: f32) -> Coroutine {
        let coroutine = async move {
            //
        };

        start_coroutine(coroutine)
    }

    pub fn on_player_receive_damage(&mut self, _player_handle: Handle<Player>, _is_from_right: bool, _damage_from: Option<Handle<Player>>) -> Coroutine {
        let coroutine = async move {
            //
        };

        start_coroutine(coroutine)
    }

    pub fn on_player_give_damage(&mut self, _player_handle: Handle<Player>, _damage_to: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            //
        };

        start_coroutine(coroutine)
    }
}