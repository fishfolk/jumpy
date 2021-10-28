use std::collections::HashMap;

use macroquad::{
    experimental::{
        coroutines::Coroutine,
        scene::Handle,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

pub mod coroutines;

mod turtle_shell;

pub use coroutines::PassiveEffectCoroutine;

use crate::effects::passive::coroutines::{add_passive_effect_coroutine, get_passive_effect_coroutine};
use crate::{player::PlayerEvent, Player, PlayerEventParams, ParticleEmitters};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassiveEffectParams {
    /// This is the id used by instances of the effect. It should not be confused with
    /// coroutine id's, which are the id's used to access the coroutines that are the actual
    /// implementations of the effect. This should be unique, as it will not be possible
    /// for more than one effect with a certain id to be active on a player at the same
    /// time. It is possible to have more than one effect instance with the same coroutine
    /// id active at the same time, however, as long as the instances has different id's.
    pub id: String,
    /// This map holds the id's of the coroutines that are called by the event instance, mapped
    /// to the `PlayerEvent` they should be called on.
    pub coroutines: HashMap<PlayerEvent, String>,
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
    /// This is the amount of times the coroutine can be called, before the effect is depleted
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uses: Option<u32>,
    /// This is the duration of the effect.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
}

pub struct PassiveEffectInstance {
    pub id: String,
    pub coroutines: HashMap<PlayerEvent, PassiveEffectCoroutine>,
    pub particle_effect_id: Option<String>,
    pub event_particle_effect_id: Option<String>,
    pub uses: Option<u32>,
    use_cnt: u32,
    duration: Option<f32>,
    duration_timer: f32,
}

impl PassiveEffectInstance {
    pub fn new(params: PassiveEffectParams) -> Self {
        let mut coroutines = HashMap::new();
        for (event, id) in params.coroutines {
            let coroutine = get_passive_effect_coroutine(&id);
            coroutines.insert(event, *coroutine);
        }

        PassiveEffectInstance {
            id: params.id,
            coroutines,
            particle_effect_id: params.particle_effect_id,
            event_particle_effect_id: params.event_particle_effect_id,
            uses: params.uses,
            use_cnt: 0,
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

        if let Some(uses) = self.uses {
            if self.use_cnt >= uses {
                return true;
            }
        }

        false
    }

    pub fn on_player_event(
        &mut self,
        player_handle: Handle<Player>,
        position: Vec2,
        params: PlayerEventParams,
    ) -> Option<Coroutine> {
        if let Some(coroutine) = self.coroutines.get(&(&params).into()) {
            if self.uses.is_some() {
                self.use_cnt += 1;
            }

            if let Some(particle_effect_id) = &self.event_particle_effect_id {
                let mut particle_emitters = scene::find_node_by_type::<ParticleEmitters>().unwrap();
                particle_emitters.spawn(particle_effect_id, position);
            }

            let res = coroutine(&self.id, player_handle, params);

            Some(res)
        } else {
            None
        }
    }
}

/// This adds all coroutine implementations to the directory, so that they can be accessed by
/// passive effect instances.
pub fn init_passive_effects() {
    add_passive_effect_coroutine(turtle_shell::COROUTINE_ID, turtle_shell::coroutine);
}