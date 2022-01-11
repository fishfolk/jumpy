use hecs::{Entity, World};
use macroquad::audio::play_sound_once;

use macroquad::prelude::*;
use macroquad::experimental::collections::storage;

use serde::{Deserialize, Serialize};

use crate::{json, Resources};
use crate::math::{deg_to_rad, rotate_vector, IsZero};
use crate::Result;

pub mod projectiles;
pub mod triggered;

pub use triggered::{TriggeredEffectMetadata, TriggeredEffectTrigger};

use crate::effects::active::projectiles::spawn_projectile;
use crate::effects::active::triggered::{spawn_triggered_effect, TriggeredEffect};
use crate::particles::ParticleEmitterParams;
use crate::player::PlayerState;
use crate::{PhysicsBody, Transform};
pub use projectiles::ProjectileKind;

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

    match *params.kind {
        ActiveEffectKind::CircleCollider {
            radius,
            segment,
            is_explosion,
        } => {
            let circle = Circle::new(origin.x, origin.y, radius);

            for (e, (state, transform, body)) in
                world.query_mut::<(&mut PlayerState, &Transform, &PhysicsBody)>()
            {
                if is_explosion || e != owner {
                    let player_rect = body.as_rect(transform.position);
                    if circle.overlaps_rect(&player_rect) {
                        let mut is_hit = false;

                        if let Some(mut segment) = segment {
                            if is_facing_left {
                                segment.x = -segment.x;
                            }

                            if segment.x == 1 {
                                is_hit = player_rect.x + player_rect.w >= circle.point().x;
                            } else if segment.x == -1 {
                                is_hit = player_rect.x <= circle.point().x;
                            }

                            if segment.y == 1 {
                                is_hit =
                                    is_hit && player_rect.y + player_rect.h <= circle.point().y;
                            } else if segment.y == -1 {
                                is_hit = is_hit && player_rect.y >= circle.point().y;
                            }
                        } else {
                            is_hit = true;
                        }

                        if is_hit {
                            state.is_dead = true;
                        }
                    }
                }
            }

            if is_explosion {
                for (_, (effect, transform, body)) in
                    world.query_mut::<(&mut TriggeredEffect, &Transform, &PhysicsBody)>()
                {
                    let trigger_rect = body.as_rect(transform.position);
                    if circle.overlaps_rect(&trigger_rect)
                        && effect.trigger.contains(&TriggeredEffectTrigger::Explosion)
                    {
                        effect.is_triggered = true;
                        effect.should_override_delay = true;
                    }
                }
            }
        }
        ActiveEffectKind::RectCollider { width, height } => {
            let mut rect = Rect::new(origin.x, origin.y, width, height);
            if is_facing_left {
                rect.x -= rect.w;
            }

            for (_, (state, transform, body)) in
                world.query_mut::<(&mut PlayerState, &Transform, &PhysicsBody)>()
            {
                let player_rect = body.as_rect(transform.position);
                if rect.overlaps(&player_rect) {
                    state.is_dead = true;
                }
            }
        }
        ActiveEffectKind::TriggeredEffect { mut params } => {
            if is_facing_left {
                params.velocity.x = -params.velocity.x;
            }

            spawn_triggered_effect(world, owner, origin, *params)?;
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

            spawn_projectile(
                world,
                owner,
                kind,
                origin,
                velocity,
                range,
                particles,
            );
        }
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
    /// Can select a segment of the circle by setting `segment`. This can be either a quarter or a
    /// half of the circle, selected by setting `x` and `y` of `segment`.
    /// If `x` is one and `y` is zero, the forward-facing half of the circle will be used, if `x` is
    /// one and `y` is negative one, the upper forward-facing quarter of the circle will be used,
    /// if `x` is negative one and `y` is one, the lower backward-facing quarter of the circle will
    /// be used, and so on.
    CircleCollider {
        radius: f32,
        #[serde(
        default,
        with = "json::ivec2_opt",
        skip_serializing_if = "Option::is_none"
        )]
        segment: Option<IVec2>,
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
        particles: Vec<ParticleEmitterParams>,
    }
}