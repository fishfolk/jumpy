use serde::{Deserialize, Serialize};

pub mod active;
pub mod passive;

pub use passive::{PassiveEffectInstance, PassiveEffectParams};

pub use active::{
    active_effect_coroutine, add_active_effect_coroutine, get_active_effect_coroutine,
    ActiveEffectCoroutine, ActiveEffectKind, ActiveEffectParams, ParticleControllers, Projectiles,
    TriggeredEffectTrigger, TriggeredEffects,
};

/// This is used to allow both active and passive effects to be used as values in JSON
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnyEffectParams {
    Active(ActiveEffectParams),
    Passive(PassiveEffectParams),
}
