use std::collections::HashMap;

use macroquad::{
    experimental::{
        coroutines::{start_coroutine, Coroutine},
        scene::Handle,
    },
    prelude::*,
};
use macroquad::prelude::coroutines::wait_seconds;

use serde::{Deserialize, Serialize};

use crate::{
    json,
    math::{deg_to_rad, rotate_vector},
    Player,
};

pub mod projectiles;

pub use projectiles::{
    Projectiles,
    default_projectile_color,
};

use crate::nodes::ParticleEmitters;

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
    Bool {
        value: bool,
    },
    Int {
        value: i32,
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
    UVec2 {
        #[serde(with = "json::uvec2_def")]
        value: UVec2,
    },
}

// This should hold implementations of the commonly used weapon effects, that see usage spanning
// many different weapon implementations. For more specialized effects, only likely to be used
// for a single weapon implementation, `Custom` can be used. The main reason for adding `Custom`,
// however, is to accommodate an eventual integration of a scripting API, so all effects,
// specialized or not, can be implemented as a variant of this enum.
//
// The effects that have the `Collider` suffix denote effects that do an immediate collider check,
// upon attack, using the weapons `effect_offset` as origin.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "effect", rename_all = "snake_case")]
pub enum WeaponEffectKind {
    // This is used to add multiple effects to a weapon, without having to implement a custom effect
    Batch {
        #[serde(rename = "batch_effects")]
        effects: Vec<WeaponEffectParams>,
    },
    // Custom effects are made by implementing `WeaponEffectCoroutine`, either directly in code or
    // in scripts if/when we add a scripting API
    Custom {
        #[serde(rename = "custom_effect_id")]
        id: String,
        #[serde(default, rename = "custom_effect_params")]
        params: HashMap<String, CustomWeaponEffectParam>,
    },
    CircleCollider {
        #[serde(rename = "circle_radius")]
        radius: f32,
    },
    RectCollider {
        #[serde(rename = "rect_width")]
        width: f32,
        #[serde(rename = "rect_height")]
        height: f32,
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

fn get_custom_weapon_effect(id: &str) -> CustomWeaponEffectCoroutine {
    let res = unsafe { get_custom_weapon_effects_map() }.get(id).unwrap();
    *res
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponEffectParams {
    #[serde(flatten)]
    pub kind: WeaponEffectKind,
    #[serde(default, rename = "effect_particle_effect_id", skip_serializing_if = "Option::is_none")]
    pub particle_effect_id: Option<String>,
    #[serde(default, rename = "effect_delay")]
    pub delay: f32,
}

// This is implemented for `Custom` effects (remember to also add it to the effects directory).
// This is not strictly necessary as of writing this, as there is no way of adding effects through
// scripts etc., so new effects can also be implemented by creating a new variant of
// `WeaponEffectKind` and implementing the effect directly in the `weapon_effect_coroutine` function
pub type CustomWeaponEffectCoroutine =
    fn(Handle<Player>, HashMap<String, CustomWeaponEffectParam>) -> Coroutine;

pub fn weapon_effect_coroutine(
    player_handle: Handle<Player>,
    origin: Vec2,
    params: WeaponEffectParams,
) -> Coroutine {
    let coroutine = async move {
        wait_seconds(params.delay).await;

        let is_facing_right = {
            let player = scene::get_node(player_handle);
            player.body.is_facing_right
        };

        if let Some(particle_effect_id) = &params.particle_effect_id {
            let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
            let emitter = particles.emitters
                .get_mut(particle_effect_id)
                .unwrap_or_else(|| panic!("Invalid particle effect emitter ID '{}'", particle_effect_id));
            emitter.spawn(origin);
        }

        match params.kind {
            WeaponEffectKind::Batch { effects } => {
                for params in effects {
                    weapon_effect_coroutine(player_handle, origin, params);
                }
            }
            WeaponEffectKind::Custom {
                id,
                params,
            } => {
                let f = get_custom_weapon_effect(&id);
                f(player_handle, params);
            }
            WeaponEffectKind::CircleCollider {
                radius,
            } => {
                // borrow player so that it is excluded from hit check below
                let _player = scene::get_node(player_handle);

                let circle = Circle::new(origin.x, origin.y, radius);
                for mut player in scene::find_nodes_by_type::<Player>() {
                    if circle.overlaps_rect(&player.get_hitbox()) {
                        println!("overlap");
                        let is_to_the_right = origin.x < player.body.pos.x;
                        player.kill(is_to_the_right);
                    }
                }
            }
            WeaponEffectKind::RectCollider {
                width,
                height,
            } => {
                // borrow player so that it is excluded from hit check below
                let _player = scene::get_node(player_handle);

                let mut rect = Rect::new(origin.x, origin.y, width, height);
                if !is_facing_right {
                    rect.x -= rect.w;
                }

                for mut player in scene::find_nodes_by_type::<Player>() {
                    if rect.overlaps(&player.get_hitbox()) {
                        let is_to_the_right = origin.x < player.body.pos.x;
                        player.kill(is_to_the_right);
                    }
                }
            }
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

                let mut projectiles = scene::find_node_by_type::<Projectiles>().unwrap();

                projectiles.spawn(player_handle, origin, velocity, range, size, color);
            }
        }
    };

    start_coroutine(coroutine)
}
