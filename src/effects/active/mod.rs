use hecs::{Entity, World};
use macroquad::audio::play_sound_once;
use macroquad::color;

use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use core::math::{deg_to_rad, rotate_vector, IsZero};
use core::Result;

use crate::{json, Resources};
use crate::{PassiveEffectInstance, PassiveEffectMetadata};

pub mod projectiles;
pub mod triggered;

pub use triggered::{TriggeredEffectMetadata, TriggeredEffectTrigger};

use crate::effects::active::projectiles::{spawn_projectile, ProjectileParams};
use crate::effects::active::triggered::{spawn_triggered_effect, TriggeredEffect};
use crate::particles::ParticleEmitterMetadata;
use crate::player::{on_player_damage, Player};
use crate::{PhysicsBody, Transform};
pub use projectiles::ProjectileKind;

const COLLIDER_DEBUG_DRAW_TTL: f32 = 0.5;

struct CircleCollider {
    r: f32,
    ttl_timer: f32,
}

struct RectCollider {
    w: f32,
    h: f32,
    ttl_timer: f32,
}

pub fn spawn_active_effect(
    world: &mut World,
    owner: Entity,
    origin: Vec2,
    params: ActiveEffectMetadata,
) -> Result<()> {
    let is_facing_left = {
        let player = world.get::<Player>(owner).unwrap();
        player.is_facing_left
    };

    if let Some(id) = &params.sound_effect_id {
        let resources = storage::get::<Resources>();
        let sound = resources.sounds.get(id).unwrap();

        play_sound_once(*sound);
    }

    let mut damage = Vec::new();

    match *params.kind {
        ActiveEffectKind::CircleCollider {
            radius,
            passive_effects,
            is_lethal,
            is_explosion,
        } => {
            let circle = Circle::new(origin.x, origin.y, radius);

            #[cfg(debug_assertions)]
            {
                world.spawn((
                    Transform::new(origin, 0.0),
                    CircleCollider {
                        r: radius,
                        ttl_timer: 0.0,
                    },
                ));
            }

            for (e, (transform, body)) in world.query::<(&Transform, &PhysicsBody)>().iter() {
                let other_rect = body.as_rect(transform.position);
                if circle.overlaps_rect(&other_rect) {
                    if let Ok(mut player) = world.get_mut::<Player>(e) {
                        if is_explosion || e != owner {
                            if is_lethal {
                                damage.push((owner, e));
                            }

                            for meta in passive_effects.clone().into_iter() {
                                let effect_instance = PassiveEffectInstance::new(None, meta);
                                player.passive_effects.push(effect_instance);
                            }
                        }
                    } else if is_explosion {
                        if let Ok(mut effect) = world.get_mut::<TriggeredEffect>(e) {
                            if effect.trigger.contains(&TriggeredEffectTrigger::Explosion) {
                                effect.is_triggered = true;
                                effect.triggered_by = Some(owner);
                                effect.should_override_delay = true;
                            }
                        }
                    }
                }
            }
        }
        ActiveEffectKind::RectCollider {
            width,
            height,
            is_lethal,
            passive_effects,
        } => {
            let mut rect = Rect::new(origin.x, origin.y, width, height);
            if is_facing_left {
                rect.x -= rect.w;
            }

            #[cfg(debug_assertions)]
            {
                world.spawn((
                    Transform::new(origin, 0.0),
                    RectCollider {
                        w: rect.w,
                        h: rect.h,
                        ttl_timer: 0.0,
                    },
                ));
            }

            for (e, (transform, player, body)) in
                world.query_mut::<(&Transform, &mut Player, &PhysicsBody)>()
            {
                if owner != e {
                    let other_rect = body.as_rect(transform.position);
                    if rect.overlaps(&other_rect) {
                        if is_lethal {
                            damage.push((owner, e));
                        }

                        for meta in passive_effects.clone().into_iter() {
                            let effect_instance = PassiveEffectInstance::new(None, meta);
                            player.passive_effects.push(effect_instance);
                        }
                    }
                }
            }
        }
        ActiveEffectKind::TriggeredEffect { meta } => {
            spawn_triggered_effect(world, owner, origin, is_facing_left, *meta)?;
        }
        ActiveEffectKind::Projectile {
            kind,
            speed,
            range,
            spread,
            is_lethal,
            passive_effects,
            particles,
        } => {
            let mut velocity = Vec2::ZERO;
            if is_facing_left {
                velocity.x = -speed
            } else {
                velocity.x = speed
            }

            if spread != 0.0 {
                let rad = deg_to_rad(spread);
                let spread = rand::gen_range(-rad, rad);

                velocity = rotate_vector(velocity, spread);
            }

            spawn_projectile(
                world,
                owner,
                kind,
                origin,
                velocity,
                range,
                ProjectileParams {
                    is_lethal,
                    passive_effects,
                    particle_effects: particles,
                },
            );
        }
    }

    for (damage_from_entity, damage_to_entity) in damage.drain(0..) {
        on_player_damage(world, damage_from_entity, damage_to_entity);
    }

    Ok(())
}

/// This holds all the common parameters, available to all implementations, as well as specialized
/// parameters, in the `ActiveEffectKind`.
#[derive(Clone, Serialize, Deserialize)]
pub struct ActiveEffectMetadata {
    /// This holds all the specialized parameters for the effect, dependent on the implementation,
    /// specified by its variant. It is flattened into this struct in JSON.
    #[serde(flatten)]
    pub kind: Box<ActiveEffectKind>,
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
/// many different weapon implementations.
///
/// The effects that have the `Collider` suffix denote effects that do an immediate collider check,
/// upon attack, using the weapons `effect_offset` as origin.
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActiveEffectKind {
    /// Check for hits with a `Circle` collider.
    CircleCollider {
        radius: f32,
        /// This contains any passive effects that will be spawned on collision
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        passive_effects: Vec<PassiveEffectMetadata>,
        /// If `true` the effect will do damage to any player it hits
        #[serde(default = "json::default_true", skip_serializing_if = "json::is_true")]
        is_lethal: bool,
        #[serde(default, skip_serializing_if = "json::is_false")]
        is_explosion: bool,
    },
    /// Check for hits with a `Rect` collider
    RectCollider {
        width: f32,
        height: f32,
        /// If `true` the effect will do damage to any player it hits
        #[serde(default = "json::default_true", skip_serializing_if = "json::is_true")]
        is_lethal: bool,
        /// This contains any passive effects that will be spawned on collision
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        passive_effects: Vec<PassiveEffectMetadata>,
    },
    /// Spawn a trigger that will set of another effect if its trigger conditions are met.
    TriggeredEffect {
        #[serde(flatten)]
        meta: Box<TriggeredEffectMetadata>,
    },
    /// Spawn a projectile.
    /// This would typically be used for things like a gun.
    Projectile {
        #[serde(rename = "projectile")]
        kind: ProjectileKind,
        speed: f32,
        range: f32,
        #[serde(default, skip_serializing_if = "f32::is_zero")]
        spread: f32,
        /// If `true` the effect will do damage to any player it hits
        #[serde(default = "json::default_true", skip_serializing_if = "json::is_true")]
        is_lethal: bool,
        /// This contains any passive effects that will be spawned on collision
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        passive_effects: Vec<PassiveEffectMetadata>,
        /// Particle effects that will be attached to the projectile
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        particles: Vec<ParticleEmitterMetadata>,
    },
}

pub fn debug_draw_active_effects(world: &mut World) {
    let mut to_remove = Vec::new();

    let dt = get_frame_time();

    for (e, (transform, collider)) in world.query_mut::<(&Transform, &mut CircleCollider)>() {
        collider.ttl_timer += dt;

        draw_circle_lines(
            transform.position.x,
            transform.position.y,
            collider.r,
            2.0,
            color::RED,
        );

        if collider.ttl_timer >= COLLIDER_DEBUG_DRAW_TTL {
            to_remove.push(e);
        }
    }

    for (e, (transform, collider)) in world.query_mut::<(&Transform, &mut RectCollider)>() {
        collider.ttl_timer += dt;

        draw_rectangle_lines(
            transform.position.x,
            transform.position.y,
            collider.w,
            collider.h,
            2.0,
            color::RED,
        );

        if collider.ttl_timer >= COLLIDER_DEBUG_DRAW_TTL {
            to_remove.push(e);
        }
    }

    for e in to_remove.drain(0..) {
        world.despawn(e).unwrap();
    }
}
