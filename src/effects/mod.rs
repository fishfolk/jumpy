use serde::{Deserialize, Serialize};

pub mod active;
pub mod passive;

pub use passive::{PassiveEffect, PassiveEffectParams};

pub use active::{
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
