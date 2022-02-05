use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::{Entity, World};

use serde::{Deserialize, Serialize};

use core::math::deg_to_rad;
use core::Result;

use crate::effects::active::spawn_active_effect;
use crate::particles::{ParticleEmitter, ParticleEmitterMetadata};
use crate::physics;
use crate::player::{Player, PlayerState};
use crate::{json, Drawable, PhysicsBodyParams};
use crate::{ActiveEffectMetadata, AnimatedSpriteMetadata, CollisionWorld, PhysicsBody, Transform};

const TRIGGERED_EFFECT_DRAW_ORDER: u32 = 5;

/// The various collision types that can trigger a `TriggeredEffect`.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TriggeredEffectTrigger {
    /// The player that deployed the effect
    Player,
    /// Enemy players
    Enemy,
    /// Ground tiles (all tiles with collision, except platforms)
    Ground,
    /// Explosion effects
    Explosion,
    /// Projectile hit
    Projectile,
}

pub struct TriggeredEffect {
    pub owner: Entity,
    pub trigger: Vec<TriggeredEffectTrigger>,
    pub effects: Vec<ActiveEffectMetadata>,
    pub activation_delay: f32,
    pub trigger_delay: f32,
    pub timed_trigger: Option<f32>,
    pub is_kickable: bool,
    /// This can be used to trigger the effect immediately, ignoring delay timers.
    /// Also requires `is_triggered` to be set to `true`, for this to work.
    pub should_override_delay: bool,
    pub should_collide_with_platforms: bool,
    pub is_triggered: bool,
    /// This holds a handle to the player that triggered the effect, if applicable.
    pub triggered_by: Option<Entity>,
    pub kick_delay_timer: f32,
    pub activation_timer: f32,
    pub trigger_delay_timer: f32,
    pub timed_trigger_timer: f32,
}

impl TriggeredEffect {
    pub fn new(owner: Entity, meta: TriggeredEffectMetadata) -> Self {
        TriggeredEffect {
            owner,
            trigger: meta.trigger,
            effects: meta.effects,
            activation_delay: meta.activation_delay,
            trigger_delay: meta.trigger_delay,
            timed_trigger: meta.timed_trigger,
            is_kickable: meta.is_kickable,
            should_override_delay: false,
            should_collide_with_platforms: meta.should_collide_with_platforms,
            is_triggered: false,
            triggered_by: None,
            kick_delay_timer: 0.0,
            activation_timer: 0.0,
            trigger_delay_timer: 0.0,
            timed_trigger_timer: 0.0,
        }
    }
}

pub fn spawn_triggered_effect(
    world: &mut World,
    owner: Entity,
    origin: Vec2,
    is_facing_left: bool,
    meta: TriggeredEffectMetadata,
) -> Result<Entity> {
    let mut velocity = meta.velocity;
    if is_facing_left {
        velocity.x = -velocity.x;
    }

    let offset = -meta.size / 2.0;

    let actor = {
        let mut collision_world = storage::get_mut::<CollisionWorld>();
        collision_world.add_actor(origin, meta.size.x as i32, meta.size.y as i32)
    };

    let rotation = deg_to_rad(meta.rotation);

    let entity = world.spawn((
        TriggeredEffect::new(owner, meta.clone()),
        Transform::new(origin, rotation),
        PhysicsBody::new(
            actor,
            velocity,
            PhysicsBodyParams {
                offset,
                size: meta.size,
                can_rotate: meta.can_rotate,
                gravity: meta.gravity,
                ..Default::default()
            },
        ),
    ));

    if let Some(meta) = meta.sprite.clone() {
        let animations = meta
            .animations
            .clone()
            .into_iter()
            .map(|a| a.into())
            .collect::<Vec<_>>();

        let mut drawable = Drawable::new_animated_sprite(
            TRIGGERED_EFFECT_DRAW_ORDER,
            &meta.texture_id,
            animations.as_slice(),
            meta.clone().into(),
        );

        {
            let sprite = drawable.get_animated_sprite_mut().unwrap();
            sprite.offset -= sprite.frame_size / 2.0;
        }

        world.insert_one(entity, drawable)?;
    }

    if !meta.effects.is_empty() {
        let mut particle_emitters = meta
            .particles
            .into_iter()
            .map(ParticleEmitter::new)
            .collect::<Vec<_>>();

        for emitter in &mut particle_emitters {
            emitter.activate();
        }

        world.insert_one(entity, particle_emitters)?
    }

    Ok(entity)
}

const KICK_FORCE: f32 = 15.0;
const KICK_DELAY: f32 = 0.22;

pub fn fixed_update_triggered_effects(world: &mut World) {
    let dt = get_frame_time();

    let mut to_trigger = Vec::new();

    let players = world
        .query::<(&Player, &Transform, &PhysicsBody)>()
        .iter()
        .filter_map(|(e, (player, transform, body))| {
            if player.state == PlayerState::Dead {
                None
            } else {
                Some((e, player.is_facing_left, transform.position, body.size))
            }
        })
        .collect::<Vec<_>>();

    for (entity, (effect, transform, body)) in world
        .query::<(&mut TriggeredEffect, &Transform, &mut PhysicsBody)>()
        .iter()
    {
        if !effect.should_collide_with_platforms {
            let mut collision_world = storage::get_mut::<CollisionWorld>();
            collision_world.descent(body.actor);
        }

        effect.timed_trigger_timer += dt;
        effect.kick_delay_timer += dt;
        effect.activation_timer += dt;

        if let Some(timed_trigger) = effect.timed_trigger {
            if effect.timed_trigger_timer >= timed_trigger {
                effect.is_triggered = true;
            }
        }

        if effect.is_triggered {
            effect.trigger_delay_timer += dt;
        }

        if !effect.is_triggered && effect.activation_timer >= effect.activation_delay {
            let collider = Rect::new(
                transform.position.x,
                transform.position.y,
                body.size.x,
                body.size.y,
            );

            let can_be_triggered_by_player =
                effect.trigger.contains(&TriggeredEffectTrigger::Player);
            let can_be_triggered_by_enemy = effect.trigger.contains(&TriggeredEffectTrigger::Enemy);
            let can_be_triggered_by_ground =
                effect.trigger.contains(&TriggeredEffectTrigger::Ground);

            if can_be_triggered_by_player || can_be_triggered_by_enemy {
                let should_exclude_owner = (effect.is_kickable
                    && effect.kick_delay_timer < KICK_DELAY)
                    || (!can_be_triggered_by_player && !effect.is_kickable);

                'players: for (pe, is_facing_left, position, size) in players.clone() {
                    if !should_exclude_owner || pe != effect.owner {
                        let player_collider = Rect::new(position.x, position.y, size.x, size.y);

                        if collider.overlaps(&player_collider) {
                            let mut should_trigger = false;

                            if effect.is_kickable && effect.kick_delay_timer >= KICK_DELAY {
                                if is_facing_left && transform.position.x < position.x + size.x {
                                    body.velocity.x = -KICK_FORCE;
                                } else if !is_facing_left && transform.position.x > position.x {
                                    body.velocity.x = KICK_FORCE;
                                } else {
                                    should_trigger = true;
                                }
                            } else {
                                should_trigger = true;
                            }

                            if should_trigger {
                                effect.is_triggered = true;
                                effect.triggered_by = Some(pe);
                            }

                            break 'players;
                        }
                    }
                }
            }

            if can_be_triggered_by_ground && body.is_on_ground {
                effect.is_triggered = true;
            }
        }

        if effect.is_triggered
            && (effect.should_override_delay || effect.trigger_delay_timer >= effect.trigger_delay)
        {
            let params = (
                entity,
                effect.triggered_by,
                effect.owner,
                transform.position,
                effect.effects.clone(),
            );
            to_trigger.push(params);
        }
    }

    for (e, _, owner, origin, effects) in to_trigger.drain(0..) {
        for params in effects {
            if let Err(err) = spawn_active_effect(world, owner, origin, params) {
                #[cfg(debug_assertions)]
                println!("WARNING: {}", err);
            }
        }

        if let Err(err) = world.despawn(e) {
            #[cfg(debug_assertions)]
            println!("WARNING: {}", err);
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TriggeredEffectMetadata {
    /// The effects to instantiate when the triggers condition is met
    #[serde(default, alias = "effect")]
    pub effects: Vec<ActiveEffectMetadata>,
    /// Particle effects that will be attached to the trigger
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub particles: Vec<ParticleEmitterMetadata>,
    /// This specifies the size of the trigger.
    #[serde(with = "json::vec2_def")]
    pub size: Vec2,
    /// This specifies the valid trigger conditions for the trigger.
    #[serde(default)]
    pub trigger: Vec<TriggeredEffectTrigger>,
    /// This specifies the velocity of the triggers body when it is instantiated.
    #[serde(default, with = "json::vec2_def")]
    pub velocity: Vec2,
    /// This specifies the initial rotation of the sprite.
    #[serde(default)]
    pub rotation: f32,
    /// This can be used to add an animated sprite to the trigger. If only a sprite is desired, an
    /// animation with only one frame can be used.
    #[serde(default, alias = "animation", skip_serializing_if = "Option::is_none")]
    pub sprite: Option<AnimatedSpriteMetadata>,
    /// This specifies the delay between the the trigger is instantiated and when it will be
    /// possible to trigger it.
    /// Explosions and projectiles, if in the list of valid trigger conditions, will ignore this
    /// and trigger the effect immediately.
    #[serde(default)]
    pub activation_delay: f32,
    /// This specifies the delay between the triggers conditions are met and the effect is triggered.
    /// Explosions and projectiles, if in the list of valid trigger conditions, will ignore this
    /// and trigger the effect immediately.
    #[serde(default)]
    pub trigger_delay: f32,
    /// If a value is specified the effect will trigger automatically after `value` time has passed.
    /// Explosions and projectiles, if in the list of valid trigger conditions, will ignore this
    /// and trigger the effect immediately.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timed_trigger: Option<f32>,
    /// If this is `true` the trigger is kicked by a player, if it hits him while he is facing it
    #[serde(default)]
    pub is_kickable: bool,
    /// If this is `true` the effect will collide with platforms. This will also trigger it on
    /// collisions with platforms, if `ground` is selected as one of the trigger criteria
    #[serde(default, rename = "collide_with_platforms")]
    pub should_collide_with_platforms: bool,
    /// If this is `true` the triggered physics body will rotate while in the air.
    #[serde(default)]
    pub can_rotate: bool,
    #[serde(default = "default_physics_gravity")]
    pub gravity: f32,
}

impl Default for TriggeredEffectMetadata {
    fn default() -> Self {
        TriggeredEffectMetadata {
            effects: Vec::new(),
            particles: Vec::new(),
            size: vec2(6.0, 6.0),
            trigger: Vec::new(),
            velocity: Vec2::ZERO,
            rotation: 0.0,
            sprite: None,
            activation_delay: 0.0,
            trigger_delay: 0.0,
            timed_trigger: None,
            is_kickable: false,
            should_collide_with_platforms: false,
            can_rotate: false,
            gravity: default_physics_gravity(),
        }
    }
}

fn default_physics_gravity() -> f32 {
    physics::GRAVITY
}
