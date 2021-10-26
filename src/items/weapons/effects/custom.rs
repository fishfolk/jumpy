use std::collections::HashMap;

use macroquad::experimental::{coroutines::Coroutine, scene::Handle};

use crate::{json::GenericParam, Player};

static mut CUSTOM_WEAPON_EFFECTS: Option<HashMap<String, CustomWeaponEffectCoroutine>> = None;

unsafe fn get_custom_weapon_effects_map(
) -> &'static mut HashMap<String, CustomWeaponEffectCoroutine> {
    if CUSTOM_WEAPON_EFFECTS.is_none() {
        CUSTOM_WEAPON_EFFECTS = Some(HashMap::new());
    }

    CUSTOM_WEAPON_EFFECTS.as_mut().unwrap()
}

#[allow(dead_code)]
pub fn add_custom_weapon_effect(id: &str, f: CustomWeaponEffectCoroutine) {
    unsafe { get_custom_weapon_effects_map() }.insert(id.to_string(), f);
}

pub fn get_custom_weapon_effect(id: &str) -> &CustomWeaponEffectCoroutine {
    unsafe { get_custom_weapon_effects_map() }.get(id).unwrap()
}

// This is implemented for `Custom` effects (remember to also add it to the effects directory).
// This is not strictly necessary as of writing this, as there is no way of adding effects through
// scripts etc., so new effects can also be implemented by creating a new variant of
// `WeaponEffectKind` and implementing the effect directly in the `weapon_effect_coroutine` function
pub type CustomWeaponEffectCoroutine =
    fn(Handle<Player>, HashMap<String, GenericParam>) -> Coroutine;
