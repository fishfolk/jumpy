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
    json::{self, GenericParam},
    math::{deg_to_rad, rotate_vector},
    ParticleEmitters, Player,
};

use super::AnyEffectParams;

pub mod projectiles;
pub mod triggered;

pub use triggered::{TriggeredEffectParams, TriggeredEffectTrigger, TriggeredEffects};

mod custom;

pub use custom::{add_custom_active_effect, get_custom_active_effect, CustomActiveEffectCoroutine};

pub use projectiles::{ProjectileKind, Projectiles};

/// This holds all the common parameters, available to all implementations, as well as specialized
/// parameters, in the `ActiveEffectKind`.
#[derive(Clone, Serialize, Deserialize)]
pub struct ActiveEffectParams {
    /// This holds all the specialized parameters for the effect, dependent on the implementation,
    /// specified by its variant. It is flattened into this struct in JSON.
    #[serde(flatten)]
    pub kind: Box<ActiveEffectKind>,
    /// This specifies the id of a particle effect to emit when the effect is instantiated.
    #[serde(
        default,
        rename = "particle_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub particle_effect_id: Option<String>,
    /// This specifies the id of a sound effect to play when the effect is instantiated.
    #[serde(
        default,
        rename = "sound_effect",
        skip_serializing_if = "Option::is_none"
    )]
    pub sound_effect_id: Option<String>,
    /// The delay between instantiation of the effect is requested and the actual instantiation.
    /// This will delay the entire effect, including sound effects, particle effects and anything
    /// drawn as a result of the effect, so in most cases it is probably better to use a
    /// `TriggeredEffect`, with a `timed_trigger` value, in stead of using this.
    #[serde(default)]
    pub delay: f32,
}

/// This should hold implementations of the commonly used weapon effects, that see usage spanning
/// many different weapon implementations. For more specialized effects, only likely to be used
/// for a single weapon implementation, `Custom` can be used. The main reason for adding `Custom`,
/// however, is to accommodate an eventual integration of a scripting API, so all effects,
/// specialized or not, can be implemented as a variant of this enum.
///
/// The effects that have the `Collider` suffix denote effects that do an immediate collider check,
/// upon attack, using the weapons `effect_offset` as origin.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActiveEffectKind {
    /// Custom effects are made by implementing `WeaponEffectCoroutine`, either directly in code or
    /// in scripts if/when we add a scripting API
    Custom {
        #[serde(rename = "id")]
        id: String,
        #[serde(default, rename = "params")]
        params: HashMap<String, GenericParam>,
    },
    /// Check for hits with a `Circle` collider.
    /// Can select a segment of the circle by setting `segment`. This can be either a quarter or a
    /// half of the circle, selected by setting `x` and `y` of `segment`.
    /// If `x` is one and `y` is zero, the forward-facing half of the circle will be used, if `x` is
    /// one and `y` is negative one, the upper forward-facing quarter of the circle will be used,
    /// if `x` is negative one and `y` is one, the lower backward-facing quarter of the circle will
    /// be used, and so on.
    CircleCollider {
        radius: f32,
        #[serde(default, with = "json::ivec2_opt")]
        segment: Option<IVec2>,
        #[serde(default)]
        is_explosion: bool,
    },
    /// Check for hits with a `Rect` collider
    RectCollider { width: f32, height: f32 },
    /// Spawn a trigger that will set of another effect if its trigger conditions are met.
    TriggeredEffect {
        #[serde(flatten)]
        params: TriggeredEffectParams,
    },
    /// Spawn a projectile.
    /// This would typically be used for things like a gun.
    Projectile {
        #[serde(rename = "projectile")]
        kind: ProjectileKind,
        speed: f32,
        range: f32,
        #[serde(default)]
        spread: f32,
    },
}

pub fn active_effect_coroutine(
    player_handle: Handle<Player>,
    origin: Vec2,
    params: ActiveEffectParams,
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

        match *params.kind {
            ActiveEffectKind::Custom { id, params } => {
                let f = get_custom_active_effect(&id);
                f(player_handle, params);
            }
            ActiveEffectKind::CircleCollider {
                radius,
                segment,
                is_explosion,
            } => {
                // borrow player so that it is excluded from hit check below
                let mut _player = None;
                if !is_explosion {
                    _player = scene::try_get_node(player_handle)
                }

                let circle = Circle::new(origin.x, origin.y, radius);
                for player in scene::find_nodes_by_type::<Player>() {
                    let collider = player.get_collider_rect();
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
                            let is_from_right = origin.x > player.body.position.x;
                            Player::on_receive_damage(
                                player.handle(),
                                is_from_right,
                                Some(player_handle),
                            );
                        }
                    }
                }

                if is_explosion {
                    let mut triggered_effects =
                        scene::find_node_by_type::<TriggeredEffects>().unwrap();
                    triggered_effects.check_triggers_circle(
                        TriggeredEffectTrigger::Explosion,
                        &circle,
                        None,
                    );
                }
            }
            ActiveEffectKind::RectCollider { width, height } => {
                // borrow player so that it is excluded from hit check below
                let _player = scene::try_get_node(player_handle);

                let mut rect = Rect::new(origin.x, origin.y, width, height);
                if !is_facing_right {
                    rect.x -= rect.w;
                }

                for player in scene::find_nodes_by_type::<Player>() {
                    if rect.overlaps(&player.get_collider_rect()) {
                        let is_from_right = origin.x > player.body.position.x;
                        Player::on_receive_damage(
                            player.handle(),
                            is_from_right,
                            Some(player_handle),
                        );
                    }
                }
            }
            ActiveEffectKind::TriggeredEffect { mut params } => {
                let mut triggered_effects = scene::find_node_by_type::<TriggeredEffects>().unwrap();

                if !is_facing_right {
                    params.velocity.x = -params.velocity.x;
                }

                triggered_effects.spawn(player_handle, origin, params)
            }
            ActiveEffectKind::Projectile {
                kind,
                speed,
                range,
                spread,
            } => {
                let rad = deg_to_rad(spread);
                let spread = rand::gen_range(-rad, rad);

                let mut velocity = Vec2::ZERO;
                if is_facing_right {
                    velocity.x = speed
                } else {
                    velocity.x = -speed
                }

                let mut projectiles = scene::find_node_by_type::<Projectiles>().unwrap();

                projectiles.spawn(
                    player_handle,
                    kind,
                    origin,
                    rotate_vector(velocity, spread),
                    range,
                );
            }
        }
    };

    start_coroutine(coroutine)
}
