use std::collections::HashMap;

use macroquad::experimental::{coroutines::Coroutine, scene::Handle};

use crate::{Player, PlayerEventParams};

static mut PASSIVE_EFFECT_COROUTINES: Option<HashMap<String, PassiveEffectCoroutine>> = None;

unsafe fn get_passive_effects_coroutine_map() -> &'static mut HashMap<String, PassiveEffectCoroutine>
{
    PASSIVE_EFFECT_COROUTINES.get_or_insert(HashMap::new())
}

#[allow(dead_code)]
pub fn add_passive_effect_coroutine(id: &str, f: PassiveEffectCoroutine) {
    unsafe { get_passive_effects_coroutine_map() }.insert(id.to_string(), f);
}

pub fn try_get_passive_effect_coroutine(id: &str) -> Option<&PassiveEffectCoroutine> {
    unsafe { get_passive_effects_coroutine_map() }
        .get(id)
}

pub fn get_passive_effect_coroutine(id: &str) -> &PassiveEffectCoroutine {
    try_get_passive_effect_coroutine(id).unwrap()
}

/// This is implemented to create active effects and it will be called on every player event by
/// any `PassiveEffectInstance` that has its id as their `coroutine_id`.
/// To access the `PassiveEffectInstance` that is the caller, get the player node from the scene
/// and access the instance in the `passive_effects` map, using the `instance_id` parameter.
///
/// Any implementations must also be added to `init_passive_effects` function.
pub type PassiveEffectCoroutine = fn(
    instance_id: &str,
    player_handle: Handle<Player>,
    event_params: PlayerEventParams,
) -> Coroutine;
