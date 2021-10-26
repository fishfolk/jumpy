use macroquad::{
    experimental::{
        coroutines::{start_coroutine, Coroutine},
        scene::Handle,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

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
    #[serde(default)]
    pub delay: f32,
}

pub struct PassiveEffect {
    pub particle_effect_id: Option<String>,
    pub duration: Option<f32>,
    pub duration_timer: f32,
}

impl PassiveEffect {
    pub fn new(params: PassiveEffectParams) -> Self {
        PassiveEffect {
            particle_effect_id: params.particle_effect_id,
            duration: params.duration,
            duration_timer: 0.0,
        }
    }

    pub fn update_coroutine(&self, player_handle: Handle<Player>) -> Coroutine {
        let coroutine = async move {
            let _player = scene::get_node(player_handle);
        };

        start_coroutine(coroutine)
    }
}
