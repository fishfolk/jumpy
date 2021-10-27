use std::collections::HashMap;

use macroquad::experimental::{coroutines::Coroutine, scene::Handle};

use crate::{json::GenericParam, Player};

static mut CUSTOM_ACTIVE_EFFECTS: Option<HashMap<String, CustomActiveEffectCoroutine>> = None;

unsafe fn get_custom_active_effects_map(
) -> &'static mut HashMap<String, CustomActiveEffectCoroutine> {
    CUSTOM_ACTIVE_EFFECTS.get_or_insert(HashMap::new())
}

#[allow(dead_code)]
pub fn add_custom_active_effect(id: &str, f: CustomActiveEffectCoroutine) {
    unsafe { get_custom_active_effects_map() }.insert(id.to_string(), f);
}

pub fn get_custom_active_effect(id: &str) -> &CustomActiveEffectCoroutine {
    unsafe { get_custom_active_effects_map() }.get(id).unwrap()
}

// This is implemented for `Custom` effects (remember to also add it to the effects directory).
// This is not strictly necessary as of writing this, as there is no way of adding effects through
// scripts etc., so new effects can also be implemented by creating a new variant of
// `ActiveEffectKind` and implementing the effect directly in the `weapon_effect_coroutine` function
pub type CustomActiveEffectCoroutine =
    fn(Handle<Player>, HashMap<String, GenericParam>) -> Coroutine;
