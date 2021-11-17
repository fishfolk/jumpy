use macroquad::{
    experimental::{coroutines::Coroutine, scene::Handle},
    prelude::*,
};

use serde::{Deserialize, Serialize};

pub mod coroutines;

mod turtle_shell;

pub use coroutines::PassiveEffectCoroutine;

use crate::effects::passive::coroutines::{
    add_passive_effect_coroutine, get_passive_effect_coroutine,
};
use crate::json::OneOrMany;
use crate::{player::PlayerEvent, ParticleEmitters, Player, PlayerEventParams};

#[derive(Clone, Serialize, Deserialize)]
pub struct PassiveEffectParams {
    /// This is the id used by instances of the effect. Unless a `coroutine_id` is specified, this
    /// will be used to retrieve the coroutine that will be called by the effect instance when
    /// triggered by a valid event
    pub id: String,
    /// This map holds the id's of the coroutines that are called by the event instance, mapped
    /// to the `PlayerEvent` they should be called on. If no `coroutine_id` is specified, the
    /// effect instance `id` will be used.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coroutine_id: Option<String>,
    /// This specifies the player events that will trigger a call of the effects coroutine.
    pub events: OneOrMany<PlayerEvent>,
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

pub struct PassiveEffectInstance {
    pub id: String,
    pub coroutine_id: Option<String>,
    pub events: Vec<PlayerEvent>,
    pub particle_effect_id: Option<String>,
    pub event_particle_effect_id: Option<String>,
    pub blocks_damage: bool,
    pub uses: Option<u32>,
    pub item_id: Option<String>,
    use_cnt: u32,
    duration: Option<f32>,
    duration_timer: f32,
}

impl PassiveEffectInstance {
    pub fn new(item_id: Option<&str>, params: PassiveEffectParams) -> Self {
        PassiveEffectInstance {
            id: params.id,
            coroutine_id: params.coroutine_id,
            events: params.events.into(),
            particle_effect_id: params.particle_effect_id,
            event_particle_effect_id: params.event_particle_effect_id,
            blocks_damage: params.blocks_damage,
            uses: params.uses,
            item_id: item_id.map(|str| str.to_string()),
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
        if self.events.contains(&PlayerEvent::from(&params)) {
            let coroutine_id = self.coroutine_id.as_ref().unwrap_or(&self.id);

            if self.uses.is_some() {
                self.use_cnt += 1;
            }

            if let Some(particle_effect_id) = &self.event_particle_effect_id {
                let mut particle_emitters = scene::find_node_by_type::<ParticleEmitters>().unwrap();
                particle_emitters.spawn(particle_effect_id, position);
            }

            let coroutine = get_passive_effect_coroutine(coroutine_id);
            let res = coroutine(&self.id, self.item_id.as_deref(), player_handle, params);

            return Some(res);
        }

        None
    }

    pub fn default_events() -> Vec<PlayerEvent> {
        vec![PlayerEvent::Update]
    }
}

/// This adds all coroutine implementations to the directory, so that they can be accessed by
/// passive effect instances.
pub fn init_passive_effects() {
    add_passive_effect_coroutine(turtle_shell::COROUTINE_ID, turtle_shell::coroutine);
}
