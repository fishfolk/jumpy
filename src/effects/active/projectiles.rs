use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use std::f32::consts::PI;

use hecs::{Entity, World};
use macroquad_platformer::Tile;

use serde::{Deserialize, Serialize};

use crate::effects::active::triggered::TriggeredEffect;
use crate::effects::TriggeredEffectTrigger;
use crate::json;
use crate::particles::{ParticleEmitter, ParticleEmitterParams};
use crate::player::PlayerState;
use crate::{
    CollisionWorld, PhysicsBody, Resources, RigidBody, RigidBodyParams, Sprite, SpriteMetadata,
    Transform,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProjectileKind {
    Circle {
        radius: f32,
        #[serde(with = "json::ColorDef")]
        color: Color,
    },
    Rect {
        width: f32,
        height: f32,
        #[serde(with = "json::ColorDef")]
        color: Color,
    },
    Sprite {
        #[serde(rename = "sprite")]
        params: SpriteMetadata,
        /// If yes, the sprite would be rotated by angle between Vec2(1, 0) (most likely will be changed in the future) and velocity vector.
        /// This, for example, used for machine gun bullets rotation.
        #[serde(default)]
        can_rotate: bool,
    },
}

pub struct Projectile {
    pub kind: ProjectileKind,
    pub owner: Entity,
    pub origin: Vec2,
    pub range: f32,
}

impl Projectile {
    pub fn new(owner: Entity, kind: ProjectileKind, origin: Vec2, range: f32) -> Self {
        Projectile {
            owner,
            kind,
            origin,
            range,
        }
    }
}

pub fn spawn_projectile(
    world: &mut World,
    owner: Entity,
    kind: ProjectileKind,
    origin: Vec2,
    velocity: Vec2,
    range: f32,
    particles: Vec<ParticleEmitterParams>,
) -> Entity {
    let entity = world.spawn((Projectile::new(owner, kind.clone(), origin, range),));

    let mut transform = Transform::from(origin);

    let body_params = match kind {
        ProjectileKind::Rect { width, height, .. } => RigidBodyParams {
            offset: vec2(-width, -height) / 2.0,
            size: vec2(width, height),
            can_rotate: false,
        },
        ProjectileKind::Circle { radius, .. } => RigidBodyParams {
            size: vec2(radius * 2.0, radius * 2.0),
            can_rotate: false,
            ..Default::default()
        },
        ProjectileKind::Sprite { params, can_rotate } => {
            let resources = storage::get::<Resources>();
            let texture_res = resources.textures.get(&params.texture_id).unwrap();

            let size = params
                .size
                .unwrap_or_else(|| texture_res.meta.frame_size.unwrap_or(texture_res.meta.size));

            let offset = params.offset - (vec2(size.x, size.y) / 2.0);

            let meta = SpriteMetadata {
                offset,
                ..params.clone()
            };

            if can_rotate {
                let mut direction = Vec2::ZERO;

                if velocity.x < 0.0 {
                    direction.x = 1.0;
                } else {
                    direction.x = -1.0;
                }

                transform.rotation = (velocity.y - direction.y).atan2(velocity.x - direction.x);

                if velocity.x < 0.0 {
                    transform.rotation += PI;
                }
            }

            world.insert_one(entity, Sprite::from(meta)).unwrap();

            RigidBodyParams {
                offset,
                size,
                ..Default::default()
            }
        }
    };

    world
        .insert(entity, (transform, RigidBody::new(velocity, body_params)))
        .unwrap();

    let mut particle_emitters = Vec::new();
    for params in particles {
        let mut emitter = ParticleEmitter::from(params);
        emitter.is_active = true;

        particle_emitters.push(emitter);
    }

    if !particle_emitters.is_empty() {
        world.insert_one(entity, particle_emitters).unwrap();
    }

    entity
}

enum ProjectileCollision {
    Player(Entity),
    Trigger(Entity),
    Map,
}

pub fn update_projectiles(world: &mut World) {
    let bodies = world
        .query::<(&Transform, &PhysicsBody)>()
        .iter()
        .map(|(e, (transform, body))| (e, body.as_rect(transform.position)))
        .collect::<Vec<_>>();

    let collision_world = storage::get::<CollisionWorld>();

    let mut events = Vec::new();

    'projectiles: for (e, (projectile, transform, body)) in world
        .query::<(&Projectile, &Transform, &RigidBody)>()
        .iter()
    {
        if projectile.origin.distance(transform.position) >= projectile.range {
            events.push((projectile.owner, e, None));
            continue 'projectiles;
        }

        let size = body.size.as_i32();
        let map_collision = collision_world.collide_solids(transform.position, size.x, size.y);
        if map_collision == Tile::Solid {
            let res = (projectile.owner, e, Some(ProjectileCollision::Map));
            events.push(res);
            continue 'projectiles;
        }

        let rect = body.as_rect(transform.position);
        for (other, other_rect) in &bodies {
            if rect.overlaps(other_rect) {
                if let Ok(state) = world.get::<PlayerState>(*other) {
                    if !state.is_dead {
                        let res = (
                            projectile.owner,
                            e,
                            Some(ProjectileCollision::Player(*other)),
                        );
                        events.push(res);
                        continue 'projectiles;
                    }
                } else if let Ok(effect) = world.get::<TriggeredEffect>(*other) {
                    if effect.trigger.contains(&TriggeredEffectTrigger::Projectile) {
                        let res = (
                            projectile.owner,
                            e,
                            Some(ProjectileCollision::Trigger(*other)),
                        );
                        events.push(res);
                    }
                }
            }
        }
    }

    for (owner_entity, projectile_entity, collision) in events {
        if let Some(collision_kind) = collision {
            match collision_kind {
                ProjectileCollision::Player(player_entity) => {
                    let mut state = world.get_mut::<PlayerState>(player_entity).unwrap();
                    state.is_dead = true;
                }
                ProjectileCollision::Trigger(trigger_entity) => {
                    let mut effect = world.get_mut::<TriggeredEffect>(trigger_entity).unwrap();
                    if !effect.should_override_delay {
                        effect.is_triggered = true;
                        effect.should_override_delay = true;
                        effect.triggered_by = Some(owner_entity);
                    }
                }
                _ => {}
            }
        }

        let _ = world.despawn(projectile_entity);
    }
}

pub fn draw_projectiles(world: &mut World) {
    for (_, (projectile, transform)) in world.query::<(&Projectile, &Transform)>().iter() {
        match projectile.kind {
            ProjectileKind::Rect {
                width,
                height,
                color,
            } => {
                draw_rectangle(
                    transform.position.x,
                    transform.position.y,
                    width,
                    height,
                    color,
                );
            }
            ProjectileKind::Circle { radius, color } => {
                draw_circle(transform.position.x, transform.position.y, radius, color);
            }
            _ => {}
        }
    }
}
