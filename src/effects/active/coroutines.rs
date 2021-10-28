use std::collections::HashMap;

use macroquad::experimental::{coroutines::Coroutine, scene::Handle};

use crate::{json::GenericParam, Player};

static mut ACTIVE_EFFECT_COROUTINES: Option<HashMap<String, ActiveEffectCoroutine>> = None;

unsafe fn get_active_effects_coroutine_map() -> &'static mut HashMap<String, ActiveEffectCoroutine>
{
    ACTIVE_EFFECT_COROUTINES.get_or_insert(HashMap::new())
}

#[allow(dead_code)]
pub fn add_active_effect_coroutine(id: &str, f: ActiveEffectCoroutine) {
    unsafe { get_active_effects_coroutine_map() }.insert(id.to_string(), f);
}

pub fn try_get_active_effect_coroutine(id: &str) -> Option<&ActiveEffectCoroutine> {
    unsafe { get_active_effects_coroutine_map() }.get(id)
}

pub fn get_active_effect_coroutine(id: &str) -> &ActiveEffectCoroutine {
    try_get_active_effect_coroutine(id).unwrap()
}

// This is implemented for `Custom` effects (remember to also add it to the effects directory).
// This is not strictly necessary as of writing this, as there is no way of adding effects through
// scripts etc., so new effects can also be implemented by creating a new variant of
// `ActiveEffectKind` and implementing the effect directly in the `weapon_effect_coroutine` function
pub type ActiveEffectCoroutine = fn(Handle<Player>, HashMap<String, GenericParam>) -> Coroutine;
