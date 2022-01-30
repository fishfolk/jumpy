use hecs::{Entity, World};
use macroquad::audio::play_sound_once;
use macroquad::color;

use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use crate::math::{deg_to_rad, rotate_vector, IsZero};
use crate::Result;
use crate::{json, Resources};

pub mod projectiles;
pub mod triggered;

pub use triggered::{TriggeredEffectMetadata, TriggeredEffectTrigger};

use crate::effects::active::projectiles::spawn_projectile;
use crate::effects::active::triggered::{spawn_triggered_effect, TriggeredEffect};
use crate::particles::ParticleEmitterMetadata;
use crate::player::{on_player_damage, Player, PlayerState};
use crate::{PhysicsBody, Transform};
pub use projectiles::ProjectileKind;

const COLLIDER_DEBUG_DRAW_FRAMES: u32 = 40;

struct CircleCollider {
    x: f32,
    y: f32,
    r: f32,
    frame_counter: u32,
}

struct RectCollider {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    frame_counter: u32,
}

static mut CIRCLE_COLLIDERS: Option<Vec<CircleCollider>> = None;
static mut RECT_COLLIDERS: Option<Vec<RectCollider>> = None;

unsafe fn get_circle_colliders() -> &'static mut Vec<CircleCollider> {
    CIRCLE_COLLIDERS.get_or_insert_with(Vec::new)
}

unsafe fn get_rect_colliders() -> &'static mut Vec<RectCollider> {
    RECT_COLLIDERS.get_or_insert_with(Vec::new)
}

pub fn spawn_active_effect(
    world: &mut World,
    owner: Entity,
    origin: Vec2,
    params: ActiveEffectMetadata,
) -> Result<()> {
    let is_facing_left = {
        let state = world.get::<PlayerState>(owner).unwrap();
        state.is_facing_left
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
            is_explosion,
        } => {
            let circle = Circle::new(origin.x, origin.y, radius);

            unsafe {
                get_circle_colliders().push(CircleCollider {
                    x: origin.x,
                    y: origin.y,
                    r: radius,
                    frame_counter: 0,
                });
            }

            for (e, (transform, body)) in world.query::<(&Transform, &PhysicsBody)>().iter() {
                let other_rect = body.as_rect(transform.position);
                if circle.overlaps_rect(&other_rect) {
                    if world.get_mut::<Player>(e).is_ok() {
                        if is_explosion || e != owner {
                            damage.push((owner, e))
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
        ActiveEffectKind::RectCollider { width, height } => {
            let mut rect = Rect::new(origin.x, origin.y, width, height);
            if is_facing_left {
                rect.x -= rect.w;
            }

            unsafe {
                get_rect_colliders().push(RectCollider {
                    x: rect.x,
                    y: rect.y,
                    w: rect.w,
                    h: rect.h,
                    frame_counter: 0,
                });
            }

            for (e, (_, transform, body)) in
                world.query::<(&Player, &Transform, &PhysicsBody)>().iter()
            {
                if owner != e {
                    let other_rect = body.as_rect(transform.position);
                    if rect.overlaps(&other_rect) {
                        damage.push((owner, e));
                    }
                }
            }
        }
        ActiveEffectKind::TriggeredEffect { mut params } => {
            if is_facing_left {
                params.velocity.x = -params.velocity.x;
            }

            spawn_triggered_effect(world, owner, origin, is_facing_left, *params)?;
        }
        ActiveEffectKind::Projectile {
            kind,
            speed,
            range,
            spread,
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

            spawn_projectile(world, owner, kind, origin, velocity, range, particles);
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
        #[serde(default, skip_serializing_if = "json::is_false")]
        is_explosion: bool,
    },
    /// Check for hits with a `Rect` collider
    RectCollider { width: f32, height: f32 },
    /// Spawn a trigger that will set of another effect if its trigger conditions are met.
    TriggeredEffect {
        #[serde(flatten)]
        params: Box<TriggeredEffectMetadata>,
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
        /// Particle effects that will be attached to the projectile
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        particles: Vec<ParticleEmitterMetadata>,
    },
}

pub fn debug_draw_active_effects(_world: &mut World) {
    {
        let circle_colliders = unsafe { get_circle_colliders() };

        let mut i = 0;
        while i < circle_colliders.len() {
            let mut circle = circle_colliders.get_mut(i).unwrap();

            circle.frame_counter += 1;

            draw_circle_lines(circle.x, circle.y, circle.r, 2.0, color::RED);

            if circle.frame_counter >= COLLIDER_DEBUG_DRAW_FRAMES {
                circle_colliders.remove(i);
            } else {
                i += 1;
            }
        }
    }

    {
        let rect_colliders = unsafe { get_rect_colliders() };

        let mut i = 0;
        while i < rect_colliders.len() {
            let mut rect = rect_colliders.get_mut(i).unwrap();

            rect.frame_counter += 1;

            draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 2.0, color::RED);

            if rect.frame_counter >= COLLIDER_DEBUG_DRAW_FRAMES {
                rect_colliders.remove(i);
            } else {
                i += 1;
            }
        }
    }
}
