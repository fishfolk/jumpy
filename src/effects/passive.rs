use macroquad::{
    experimental::{
        coroutines::{start_coroutine, Coroutine},
        scene::Handle,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

pub mod custom;

use crate::{Player, PlayerEvent};

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
    pub id: String,
    pub particle_effect_id: Option<String>,
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

    pub fn on_player_event(
        &mut self,
        _player_handle: Handle<Player>,
        _event: PlayerEvent,
    ) -> Coroutine {
        let coroutine = async move {
            //
        };

        start_coroutine(coroutine)
    }
}
