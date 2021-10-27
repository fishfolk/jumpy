use serde::{Deserialize, Serialize};

pub mod active_effects;
pub mod passive_effects;

pub use passive_effects::{PassiveEffect, PassiveEffectParams};

pub use active_effects::{
    active_effect_coroutine, add_custom_active_effect, get_custom_active_effect, ActiveEffectKind,
    ActiveEffectParams, CustomActiveEffectCoroutine, Projectiles, TriggeredEffectTrigger,
    TriggeredEffects,
};

/// This is used to allow both active and passive effects to be used as values in JSON
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AnyEffectParams {
    Active(ActiveEffectParams),
    Passive(PassiveEffectParams),
}
