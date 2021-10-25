use std::collections::HashMap;

use macroquad::{
    experimental::{
        coroutines::{start_coroutine, wait_seconds, Coroutine},
        scene::Handle,
    },
    prelude::*,
};

use serde::{Deserialize, Serialize};

use crate::{
    json,
    math::{deg_to_rad, rotate_vector},
    nodes::{ParticleEmitters, Player},
};

pub mod projectiles;
pub mod triggered;

pub use triggered::{TriggeredEffectParams, TriggeredEffects};

mod custom;

pub use custom::{
    add_custom_weapon_effect, get_custom_weapon_effect, CustomWeaponEffectCoroutine,
    CustomWeaponEffectParam,
};

pub use projectiles::{ProjectileKind, Projectiles};

use crate::components::AnimationParams;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaponEffectParams {
    #[serde(flatten)]
    pub kind: WeaponEffectKind,
    #[serde(
        default,
        rename = "particle_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub particle_effect_id: Option<String>,
    #[serde(
        default,
        rename = "sound_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub sound_effect_id: Option<String>,
    #[serde(default)]
    pub delay: f32,
    #[serde(default)]
    pub is_friendly_fire: bool,
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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WeaponEffectKind {
    // This is used to add multiple effects to a weapon, without having to implement a custom effect
    Batch {
        effects: Vec<WeaponEffectParams>,
    },
    // Custom effects are made by implementing `WeaponEffectCoroutine`, either directly in code or
    // in scripts if/when we add a scripting API
    Custom {
        #[serde(rename = "id")]
        id: String,
        #[serde(default, rename = "params")]
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
        radius: f32,
        #[serde(default, with = "json::ivec2_opt")]
        segment: Option<IVec2>,
    },
    // Check for hits with a `Rect` collider
    RectCollider {
        width: f32,
        height: f32,
    },
    // Spawn a trigger that will set of another effect if its trigger conditions are met.
    TriggeredEffect {
        #[serde(rename = "trigger")]
        kind: WeaponEffectTriggerKind,
        #[serde(with = "json::vec2_def")]
        size: Vec2,
        #[serde(default, with = "json::vec2_def")]
        offset: Vec2,
        #[serde(default, with = "json::vec2_def")]
        velocity: Vec2,
        #[serde(rename = "triggered_effect")]
        effect: Box<WeaponEffectParams>,
        #[serde(default)]
        animation: Option<AnimationParams>,
        #[serde(default)]
        activation_delay: f32,
        #[serde(default)]
        trigger_delay: f32,
        #[serde(default)]
        timed_trigger: Option<f32>,
        #[serde(default)]
        is_kickable: bool,
    },
    // Spawn a projectile.
    // This would typically be used for things like a gun.
    Projectile {
        #[serde(rename = "projectile")]
        kind: ProjectileKind,
        speed: f32,
        range: f32,
        #[serde(default)]
        spread: f32,
    },
}

pub fn weapon_effect_coroutine(
    player_handle: Handle<Player>,
    origin: Vec2,
    params: WeaponEffectParams,
) -> Coroutine {
    let coroutine = async move {
        wait_seconds(params.delay).await;

        if let Some(particle_effect_id) = &params.particle_effect_id {
            let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
            particles.spawn(particle_effect_id, origin);
        }

        let is_facing_right = {
            if let Some(player) = scene::try_get_node(player_handle) {
                player.body.is_facing_right
            } else {
                true
            }
        };

        match params.kind {
            WeaponEffectKind::Batch { effects } => {
                for params in effects {
                    weapon_effect_coroutine(player_handle, origin, params);
                }
            }
            WeaponEffectKind::Custom { id, params } => {
                let f = get_custom_weapon_effect(&id);
                f(player_handle, params);
            }
            WeaponEffectKind::CircleCollider { radius, segment } => {
                // borrow player so that it is excluded from hit check below
                let _player = if params.is_friendly_fire {
                    None
                } else {
                    scene::try_get_node(player_handle)
                };

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
                                is_killed =
                                    is_killed && collider.y + collider.h <= circle.point().y;
                            } else if segment.y == -1 {
                                is_killed = is_killed && collider.y >= circle.point().y;
                            }
                        } else {
                            is_killed = true;
                        }

                        if is_killed {
                            let is_to_the_right = origin.x < player.body.position.x;
                            player.kill(!is_to_the_right);
                        }
                    }
                }
            }
            WeaponEffectKind::RectCollider { width, height } => {
                // borrow player so that it is excluded from hit check below
                let _player = if params.is_friendly_fire {
                    None
                } else {
                    scene::try_get_node(player_handle)
                };

                let mut rect = Rect::new(origin.x, origin.y, width, height);
                if !is_facing_right {
                    rect.x -= rect.w;
                }

                for mut player in scene::find_nodes_by_type::<Player>() {
                    if rect.overlaps(&player.get_collider()) {
                        let is_to_the_right = origin.x < player.body.position.x;
                        player.kill(!is_to_the_right);
                    }
                }
            }
            WeaponEffectKind::TriggeredEffect {
                kind,
                size,
                offset,
                velocity,
                effect,
                animation,
                activation_delay,
                trigger_delay,
                timed_trigger,
                is_kickable,
            } => {
                let mut triggered_effects = scene::find_node_by_type::<TriggeredEffects>().unwrap();

                let mut velocity = velocity;
                if !is_facing_right {
                    velocity.x = -velocity.x;
                }

                let params = TriggeredEffectParams {
                    offset,
                    velocity,
                    animation,
                    is_friendly_fire: params.is_friendly_fire,
                    activation_delay,
                    trigger_delay,
                    timed_trigger,
                    is_kickable,
                };

                triggered_effects.spawn(player_handle, kind, origin, size, *effect, params)
            }
            WeaponEffectKind::Projectile {
                kind,
                speed,
                range,
                spread,
            } => {
                let facing_dir = {
                    if let Some(player) = scene::try_get_node(player_handle) {
                        player.body.facing_dir()
                    } else {
                        vec2(1.0, 0.0)
                    }
                };

                let rad = deg_to_rad(spread);
                let spread = rand::gen_range(-rad, rad);

                let velocity = rotate_vector(facing_dir * speed, spread);

                let mut projectiles = scene::find_node_by_type::<Projectiles>().unwrap();

                projectiles.spawn(player_handle, kind, origin, velocity, range);
            }
        }
    };

    start_coroutine(coroutine)
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WeaponEffectTriggerKind {
    None,
    Player,
    Ground,
    Both,
}
