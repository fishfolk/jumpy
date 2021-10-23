use std::collections::HashMap;

use macroquad::{
    experimental::{
        coroutines::{
            Coroutine,
            start_coroutine,
            wait_seconds,
        },
        scene::Handle,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    nodes::{
        ParticleEmitters,
        Player,
    },
    math::{
        deg_to_rad,
        rotate_vector,
    },
    json,
};

pub mod projectiles;

mod custom;

pub use custom::{
    CustomWeaponEffectParam,
    CustomWeaponEffectCoroutine,
    add_custom_weapon_effect,
    get_custom_weapon_effect,
};

pub use projectiles::{
    Projectiles,
    default_projectile_color,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponEffectParams {
    #[serde(flatten)]
    pub kind: WeaponEffectKind,
    #[serde(default, rename = "effect_particle_effect_id", skip_serializing_if = "Option::is_none")]
    pub particle_effect_id: Option<String>,
    #[serde(default, rename = "effect_delay")]
    pub delay: f32,
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
    // Check for hits with a `Circle` collider.
    // Can select a segment of the circle by setting `segment`. This can be either a quarter or a
    // half of the circle, selected by setting `x` and `y` of `segment`.
    // If `x` is one and `y` is zero, the forward-facing half of the circle will be used, if `x` is
    // one and `y` is negative one, the upper forward-facing quarter of the circle will be used,
    // if `x` is negative one and `y` is one, the lower backward-facing quarter of the circle will
    // be used, and so on.
    CircleCollider {
        #[serde(rename = "circle_collider_radius")]
        radius: f32,
        #[serde(default, rename = "circle_collider_segment", with = "json::ivec2_opt")]
        segment: Option<IVec2>,
    },
    RectCollider {
        #[serde(rename = "rect_collider_width")]
        width: f32,
        #[serde(rename = "rect_collider_height")]
        height: f32,
    },
    // Spawn a projectile.
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

pub fn weapon_effect_coroutine(player_handle: Handle<Player>, origin: Vec2, params: WeaponEffectParams) -> Coroutine {
    let coroutine = async move {
        wait_seconds(params.delay).await;

        if let Some(particle_effect_id) = &params.particle_effect_id {
            let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
            let emitter = particles.emitters
                .get_mut(particle_effect_id)
                .unwrap_or_else(|| panic!("Invalid particle effect emitter ID '{}'", particle_effect_id));

            emitter.spawn(origin);
        }

        let is_facing_right = {
            let player = scene::get_node(player_handle);
            player.body.is_facing_right
        };

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
                segment,
            } => {
                // borrow player so that it is excluded from hit check below
                let _player = scene::get_node(player_handle);

                let circle = Circle::new(origin.x, origin.y, radius);
                for mut player in scene::find_nodes_by_type::<Player>() {
                    let collider = player.get_collider();
                    if circle.overlaps_rect(&collider) {
                        let mut is_killed = false;

                        if let Some(mut segment) = segment {
                            if !is_facing_right {
                                segment.x = -segment.x;
                            }

                            if segment.x == 1 {
                                is_killed = collider.x + collider.w >= circle.point().x;
                            } else if segment.x == -1 {
                                is_killed = collider.x <= circle.point().x;
                            }

                            if segment.y == 1 {
                                is_killed = is_killed && collider.y + collider.h <= circle.point().y;
                            } else if segment.y == -1 {
                                is_killed = is_killed && collider.y >= circle.point().y;
                            }
                        } else {
                            is_killed = true;
                        }

                        if is_killed {
                            let is_to_the_right = origin.x < player.body.pos.x;
                            player.kill(is_to_the_right);
                        }
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
                    if rect.overlaps(&player.get_collider()) {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponEffectTrigger {
    Player,
    Ground,
    Both,
}