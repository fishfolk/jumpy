pub mod passive_effects;
pub mod active_effects;

pub use passive_effects::{PassiveEffect, PassiveEffectParams};

pub use active_effects::{
    add_custom_active_effect, get_custom_active_effect, active_effect_coroutine,
    CustomActiveEffectCoroutine, Projectiles, TriggeredEffectTrigger, ActiveEffectKind,
    ActiveEffectParams,
};