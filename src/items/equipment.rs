use macroquad::{experimental::scene::Handle, prelude::*};

use serde::{Deserialize, Serialize};

use crate::{
    components::{AnimationParams, AnimationPlayer},
    json::OneOrMany,
    Player,
};

use crate::effects::{PassiveEffect, PassiveEffectParams};

#[derive(Clone, Serialize, Deserialize)]
pub struct EquipmentParams {
    pub effects: OneOrMany<PassiveEffectParams>,
    pub animation: AnimationParams,
}

pub struct Equipment {
    pub id: String,
    pub effects: Vec<PassiveEffect>,
    pub sprite_animation: AnimationPlayer,
}

impl Equipment {
    pub fn new(id: &str, params: EquipmentParams) -> Self {
        let mut effects = Vec::new();
        for effect_params in params.effects.into_vec() {
            effects.push(PassiveEffect::new(effect_params));
        }

        let sprite_animation = AnimationPlayer::new(params.animation);

        Equipment {
            id: id.to_string(),
            effects,
            sprite_animation,
        }
    }

    pub fn update(&mut self, dt: f32, player_handle: Handle<Player>) {
        let mut i = 0;
        while i < self.effects.len() {
            let effect = &mut self.effects[i];

            if let Some(duration) = effect.duration {
                effect.duration_timer += dt;

                if effect.duration_timer >= duration {
                    self.effects.remove(i);
                    continue;
                }

                effect.update_coroutine(player_handle);
            }

            i += 1;
        }
    }
}
