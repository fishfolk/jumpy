use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::{Entity, World};
use macroquad_platformer::Tile;

use serde::{Deserialize, Serialize};

use crate::json;
use crate::particles::{ParticleEmitter, ParticleEmitterParams};
use crate::player::{Player, PlayerState};
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
    let entity = world.spawn((
        Projectile::new(owner, kind.clone(), origin, range),
        Transform::from(origin),
    ));

    let body_params = match kind {
        ProjectileKind::Rect { width, height, .. } => RigidBodyParams {
            offset: vec2(-width, -height) / 2.0,
            size: vec2(width, height),
            can_rotate: false,
            ..Default::default()
        },
        ProjectileKind::Circle { radius, .. } => RigidBodyParams {
            size: vec2(radius * 2.0, radius * 2.0),
            can_rotate: false,
            ..Default::default()
        },
        ProjectileKind::Sprite { mut params, can_rotate } => {
            let resources = storage::get::<Resources>();
            let texture_res = resources.textures.get(&params.texture_id).unwrap();

            let size = params
                .size
                .unwrap_or_else(|| texture_res.meta.frame_size.unwrap_or(texture_res.meta.size));

            params.offset.x -= size.x / 2.0;
            params.offset.y -= size.y / 2.0;

            world
                .insert_one(entity, Sprite::from(params.clone()))
                .unwrap();

            RigidBodyParams {
                offset: vec2(-size.x, -size.y) / 2.0,
                size,
                can_rotate,
                ..Default::default()
            }
        }
    };

    world
        .insert_one(entity, RigidBody::new(velocity, body_params))
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
    Map,
}

pub fn update_projectiles(world: &mut World) {
    let players = world
        .query::<(&Player, &Transform, &PhysicsBody)>()
        .iter()
        .map(|(e, (_, transform, body))| (e, body.as_rect(transform.position)))
        .collect::<Vec<_>>();

    let collision_world = storage::get::<CollisionWorld>();

    let mut to_remove = Vec::new();

    'projectiles: for (e, (projectile, transform, body)) in world
        .query::<(&Projectile, &Transform, &RigidBody)>()
        .iter()
    {
        if projectile.origin.distance(transform.position) >= projectile.range {
            to_remove.push((e, None));
            continue 'projectiles;
        }

        let size = body.size.as_i32();
        let map_collision = collision_world.collide_solids(transform.position, size.x, size.y);
        if map_collision == Tile::Solid {
            let res = (e, Some(ProjectileCollision::Map));
            to_remove.push(res);
            continue 'projectiles;
        }

        let rect = body.as_rect(transform.position);
        for (p, player_rect) in &players {
            if rect.overlaps(player_rect) {
                let res = (e, Some(ProjectileCollision::Player(*p)));
                to_remove.push(res);
                continue 'projectiles;
            }
        }
    }

    for (e, c) in to_remove {
        if let Some(ProjectileCollision::Player(p)) = c {
            let mut state = world.get_mut::<PlayerState>(p).unwrap();

            state.is_dead = true;
        }

        let _ = world.despawn(e);
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
