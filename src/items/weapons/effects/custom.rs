use std::collections::HashMap;

use macroquad::{
    experimental::{
        scene::{
            Handle,
        },
        coroutines::Coroutine,
    },
    prelude::*,
};

use serde::{
    Serialize,
    Deserialize,
};

use crate::{
    Player,
    json,
};

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
pub type CustomWeaponEffectCoroutine = fn(Handle<Player>, HashMap<String, CustomWeaponEffectParam>) -> Coroutine;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustomWeaponEffectParam {
    Bool {
        value: bool,
    },
    Int {
        value: i32,
    },
    UInt {
        value: u32,
    },
    Float {
        value: f32,
    },
    String {
        value: String,
    },
    Color {
        #[serde(with = "json::ColorDef")]
        value: Color,
    },
    Vec2 {
        #[serde(with = "json::vec2_def")]
        value: Vec2,
    },
    IVec2 {
        #[serde(with = "json::ivec2_def")]
        value: IVec2,
    },
    UVec2 {
        #[serde(with = "json::uvec2_def")]
        value: UVec2,
    },
    Vec {
        value: Vec<Self>,
    },
    HashMap {
        value: HashMap<String, Self>,
    }
}

impl CustomWeaponEffectParam {
    pub fn get_value<T: CustomWeaponEffectParamType>(&self) -> Option<&T> {
        T::from_param(self)
    }
}

pub trait CustomWeaponEffectParamType: Clone {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self>;
}

impl CustomWeaponEffectParamType for bool {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::Bool { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for i32 {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::Int { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for u32 {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::UInt { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for f32 {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::Float { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for String {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::String { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for Color {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::Color { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for Vec2 {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::Vec2 { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for IVec2 {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::IVec2 { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for UVec2 {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::UVec2 { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for Vec<CustomWeaponEffectParam> {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::Vec { value } = param {
            Some(value)
        } else {
            None
        }
    }
}

impl CustomWeaponEffectParamType for HashMap<String, CustomWeaponEffectParam> {
    fn from_param(param: &CustomWeaponEffectParam) -> Option<&Self> {
        if let CustomWeaponEffectParam::HashMap { value } = param {
            Some(value)
        } else {
            None
        }
    }
}
