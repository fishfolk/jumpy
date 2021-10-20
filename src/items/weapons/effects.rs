use std::collections::HashMap;

use macroquad::{
    experimental::{
        scene::{
            Handle,
        },
        coroutines::{
            Coroutine,
            start_coroutine,
        }
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    Player,
    math::{deg_to_rad, rotate_vector},
    json,
};

pub mod projectiles;

pub use projectiles::Projectiles;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EffectTrigger {
    Player,
    Ground,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CustomWeaponEffectParam {
    Bool { value: bool },
    Int { value: i32 },
    Float { value: f32 },
    String { value: String },
    Color {
        #[serde(with = "json::ColorDef")]
        value: Color,
    },
    Vec2 {
        #[serde(with = "json::vec2_def")]
        value: Vec2,
    },
    UVec2 {
        #[serde(with = "json::uvec2_def")]
        value: UVec2,
    }
}

// This should hold implementations of the commonly used weapon effects, that see usage spanning
// many different weapon implementations. For more specialized effects, only likely to be used
// for a single weapon implementation, `Custom` can be used. The main reason for adding `Custom`,
// however, is to accommodate an eventual integration of a scripting API, so all effects,
// specialized or not, can be implemented as a variant of this enum.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "effect", rename_all = "snake_case")]
pub enum WeaponEffectKind {
    // Custom effects are made by implementing `WeaponEffectCoroutine`, either directly in code or
    // in scripts if/when we add a scripting API
    Custom {
        #[serde(rename = "custom_effect_id")]
        id: String,
        #[serde(default, rename = "custom_effect_params")]
        params: HashMap<String, CustomWeaponEffectParam>,
    },
    // This is used to add multiple effects to a weapon, without having to implement a custom effect
    Batch {
        #[serde(rename = "batch_effects")]
        effects: Vec<WeaponEffectParams>,
    },
    // Hit check using a cone.
    // This would typically be used for things like a flame thrower or any melee weapons.
    Cone {
        #[serde(default, rename = "cone_direction", with = "json::vec2_def")]
        direction: Vec2,
        #[serde(rename = "cone_length")]
        length: f32,
        #[serde(rename = "cone_angle")]
        angle: f32,
    },
    // Spawn a projectile..
    // This would typically be used for things like a gun.
    Projectile {
        #[serde(rename = "projectile_speed")]
        speed: f32,
        #[serde(rename = "projectile_range")]
        range: f32,
        #[serde(default, rename = "projectile_spread")]
        spread: f32,
        #[serde(rename = "projectile_size")]
        size: f32,
        #[serde(default = "default_projectile_color", with = "json::ColorDef")]
        color: Color,
    },
}

fn default_projectile_color() -> Color {
    Color::new(1.0, 1.0, 0.8, 1.0)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponEffectParams {
    #[serde(flatten)]
    pub kind: WeaponEffectKind,
}

static mut CUSTOM_WEAPON_EFFECTS: Option<HashMap<String, CustomWeaponEffectCoroutine>> = None;

unsafe fn get_custom_weapon_effects_map() -> &'static mut HashMap<String, CustomWeaponEffectCoroutine> {
    if CUSTOM_WEAPON_EFFECTS.is_none() {
        CUSTOM_WEAPON_EFFECTS = Some(HashMap::new());
    }

    CUSTOM_WEAPON_EFFECTS.as_mut().unwrap()
}

pub fn add_custom_weapon_effect(id: &str, f: CustomWeaponEffectCoroutine) {
    unsafe { get_custom_weapon_effects_map() }.insert(id.to_string(), f);
}

fn get_custom_weapon_effect(id: &str) -> CustomWeaponEffectCoroutine {
    let res = unsafe { get_custom_weapon_effects_map() }.get(id).unwrap();
    *res
}

// This is implemented for `Custom` effects (remember to also add it to the effects directory).
// This is not strictly necessary as of writing this, as there is no way of adding effects through
// scripts etc., so new effects can also be implemented by creating a new variant of
// `WeaponEffectKind` and implementing the effect directly in the match of `weapon_effect_coroutine`
pub type CustomWeaponEffectCoroutine = fn(Handle<Player>, HashMap<String, CustomWeaponEffectParam>) -> Coroutine;

pub fn weapon_effect_coroutine(player_handle: Handle<Player>, origin: Vec2, params: WeaponEffectParams) -> Coroutine {
    let coroutine = async move {
        match params.kind {
            WeaponEffectKind::Custom {
                id,
                params,
            } => {
                let f = get_custom_weapon_effect(&id);
                f(player_handle, params);
            }
            WeaponEffectKind::Batch {
                effects,
            } => {
                for params in effects {
                    weapon_effect_coroutine(player_handle, origin, params);
                }
            }
            WeaponEffectKind::Cone {
                direction: _,
                length: _,
                angle: _,
            } => {}
            WeaponEffectKind::Projectile {
                speed,
                range,
                spread,
                size,
                color,
            } => {
                let player = scene::get_node(player_handle);

                let rad = deg_to_rad(spread);
                let spread = rand::gen_range(-rad, rad);

                let velocity = rotate_vector(player.body.facing_dir() * speed, spread);

                let mut projectiles =
                    scene::find_node_by_type::<Projectiles>().unwrap();

                projectiles.spawn(
                    player_handle,
                    origin,
                    velocity,
                    range,
                    size,
                    color,
                );
            }
        }
    };

    start_coroutine(coroutine)
}