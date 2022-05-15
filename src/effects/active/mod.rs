use ff_core::ecs::{Entity, World};

use ff_core::prelude::*;

use serde::{Deserialize, Serialize};

use ff_core::prelude::*;

use crate::items::get_item;
use crate::items::spawn_item;
use crate::PassiveEffect;
use crate::PassiveEffectMetadata;

pub mod projectiles;
pub mod triggered;

pub use triggered::{TriggeredEffectMetadata, TriggeredEffectTrigger};

use crate::effects::active::projectiles::{spawn_projectile, ProjectileParams};
use crate::effects::active::triggered::{spawn_triggered_effect, TriggeredEffect};
use crate::player::{on_player_damage, Player};
use crate::PhysicsBody;
use ff_core::particles::ParticleEmitterMetadata;

use ff_core::prelude::*;

pub use projectiles::ProjectileKind;

const COLLIDER_DEBUG_DRAW_FRAMES: u32 = 120;

struct CircleCollider {
    r: f32,
    frame_cnt: u32,
}

struct RectCollider {
    w: f32,
    h: f32,
    frame_cnt: u32,
}

pub fn spawn_active_effect(
    world: &mut World,
    owner: Entity,
    spawner: Entity,
    origin: Vec2,
    params: ActiveEffectMetadata,
) -> Result<()> {
    let is_facing_left = {
        let player = world.get::<Player>(owner).unwrap();
        player.is_facing_left
    };

    if let Some(id) = &params.sound_effect_id {
        play_sound(id, false);
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
                        frame_cnt: 0,
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
                                let effect = PassiveEffect::new(None, meta);
                                player.passive_effects.push(effect);
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
                rect.x -= rect.width;
            }

            #[cfg(debug_assertions)]
            {
                world.spawn((
                    Transform::new(origin, 0.0),
                    RectCollider {
                        w: rect.width,
                        h: rect.height,
                        frame_cnt: 0,
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
                            let effect = PassiveEffect::new(None, meta);
                            player.passive_effects.push(effect);
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
                let spread = ff_core::rand::gen_range(-rad, rad);

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
        ActiveEffectKind::SpawnItem {
            item,
            offset,
            inherit_spawner_velocity,
        } => {
            let item_meta = get_item(&item);

            match spawn_item(world, origin + offset, item_meta.clone()) {
                Ok(entity) => {
                    if inherit_spawner_velocity {
                        let spawner_velocity = {
                            let mut spawner_body_query =
                                world.query_one::<&PhysicsBody>(spawner).unwrap();
                            let spawner_body = spawner_body_query.get().unwrap();
                            spawner_body.velocity
                        };

                        let mut entity_body =
                            world.query_one_mut::<&mut PhysicsBody>(entity).unwrap();
                        entity_body.velocity = spawner_velocity;
                    }
                }
                Err(e) => {
                    println!("WARNING: {:?}", e);
                }
            }
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
        #[serde(
            default = "ff_core::parsing::default_true",
            skip_serializing_if = "ff_core::parsing::is_true"
        )]
        is_lethal: bool,
        #[serde(default, skip_serializing_if = "ff_core::parsing::is_false")]
        is_explosion: bool,
    },
    /// Check for hits with a `Rect` collider
    RectCollider {
        width: f32,
        height: f32,
        /// If `true` the effect will do damage to any player it hits
        #[serde(
            default = "ff_core::parsing::default_true",
            skip_serializing_if = "ff_core::parsing::is_true"
        )]
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
        #[serde(
            default = "ff_core::parsing::default_true",
            skip_serializing_if = "ff_core::parsing::is_true"
        )]
        is_lethal: bool,
        /// This contains any passive effects that will be spawned on collision
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        passive_effects: Vec<PassiveEffectMetadata>,
        /// Particle effects that will be attached to the projectile
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        particles: Vec<ParticleEmitterMetadata>,
    },
    SpawnItem {
        item: String,
        #[serde(default, with = "ff_core::parsing::vec2_def")]
        offset: Vec2,
        #[serde(default)]
        inherit_spawner_velocity: bool,
    },
}

pub fn debug_draw_active_effects(world: &mut World, _delta_time: f32) -> Result<()> {
    let mut to_remove = Vec::new();

    for (e, (transform, collider)) in world.query_mut::<(&Transform, &mut CircleCollider)>() {
        collider.frame_cnt += 1;

        draw_circle_outline(
            transform.position.x,
            transform.position.y,
            collider.r,
            2.0,
            colors::RED,
        );

        if collider.frame_cnt >= COLLIDER_DEBUG_DRAW_FRAMES {
            to_remove.push(e);
        }
    }

    for (e, (transform, collider)) in world.query_mut::<(&Transform, &mut RectCollider)>() {
        collider.frame_cnt += 1;

        draw_rectangle_outline(
            transform.position.x,
            transform.position.y,
            collider.w,
            collider.h,
            2.0,
            colors::RED,
        );

        if collider.frame_cnt >= COLLIDER_DEBUG_DRAW_FRAMES {
            to_remove.push(e);
        }
    }

    for (_, (transform, body, effect)) in
        world.query_mut::<(&Transform, &PhysicsBody, &TriggeredEffect)>()
    {
        if let Some(opts) = &effect.grab_options {
            let rect = opts.get_collider_rect(transform.position, body.velocity);
            draw_rectangle_outline(rect.x, rect.y, rect.width, rect.height, 2.0, colors::ORANGE);
        }
    }

    for e in to_remove.drain(0..) {
        world.despawn(e).unwrap();
    }

    Ok(())
}
