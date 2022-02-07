use macroquad::experimental::collections::storage;
use macroquad::prelude::*;
use std::f32::consts::PI;

use hecs::{Entity, World};
use macroquad_platformer::Tile;

use serde::{Deserialize, Serialize};

use crate::effects::active::triggered::TriggeredEffect;
use crate::effects::TriggeredEffectTrigger;
use crate::particles::{ParticleEmitter, ParticleEmitterMetadata};
use crate::player::{on_player_damage, Player, PlayerState};
use crate::{json, Drawable, PassiveEffectInstance, PassiveEffectMetadata, SpriteParams};
use crate::{
    CollisionWorld, PhysicsBody, Resources, RigidBody, RigidBodyParams, SpriteMetadata, Transform,
};

const PROJECTILE_DRAW_ORDER: u32 = 1;

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
    pub is_lethal: bool,
    pub passive_effects: Vec<PassiveEffectMetadata>,
}

impl Projectile {
    pub fn new(
        owner: Entity,
        kind: ProjectileKind,
        origin: Vec2,
        range: f32,
        is_lethal: bool,
        passive_effects: &[PassiveEffectMetadata],
    ) -> Self {
        Projectile {
            owner,
            kind,
            origin,
            range,
            is_lethal,
            passive_effects: passive_effects.to_vec(),
        }
    }
}

#[derive(Clone)]
pub struct ProjectileParams {
    pub is_lethal: bool,
    pub passive_effects: Vec<PassiveEffectMetadata>,
    pub particle_effects: Vec<ParticleEmitterMetadata>,
}

impl Default for ProjectileParams {
    fn default() -> Self {
        ProjectileParams {
            is_lethal: true,
            passive_effects: Vec::new(),
            particle_effects: Vec::new(),
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
    params: ProjectileParams,
) -> Entity {
    let entity = world.spawn(());
    world
        .insert_one(
            entity,
            Projectile::new(
                owner,
                kind.clone(),
                origin,
                range,
                params.is_lethal,
                &params.passive_effects,
            ),
        )
        .unwrap();

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
        ProjectileKind::Sprite {
            params: meta,
            can_rotate,
        } => {
            let resources = storage::get::<Resources>();
            let texture_res = resources.textures.get(&meta.texture_id).unwrap();

            let size = meta
                .size
                .unwrap_or_else(|| texture_res.meta.frame_size.unwrap_or(texture_res.meta.size));

            let offset = meta.offset - (vec2(size.x, size.y) / 2.0);

            let is_flipped_x = velocity.x < 0.0;

            if can_rotate {
                let mut direction = Vec2::ZERO;

                if is_flipped_x {
                    direction.x = 1.0;
                } else {
                    direction.x = -1.0;
                }

                transform.rotation = (velocity.y - direction.y).atan2(velocity.x - direction.x);

                if is_flipped_x {
                    transform.rotation += PI;
                }
            }

            world
                .insert_one(
                    entity,
                    Drawable::new_sprite(
                        PROJECTILE_DRAW_ORDER,
                        &meta.texture_id,
                        SpriteParams {
                            is_flipped_x,
                            offset,
                            ..meta.clone().into()
                        },
                    ),
                )
                .unwrap();

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
    for params in params.particle_effects {
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

pub fn fixed_update_projectiles(world: &mut World) {
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
                if let Ok(mut player) = world.get_mut::<Player>(*other) {
                    if player.state != PlayerState::Dead {
                        for meta in projectile.passive_effects.clone().into_iter() {
                            let effect_instance = PassiveEffectInstance::new(None, meta);

                            player.passive_effects.push(effect_instance);
                        }

                        if projectile.is_lethal {
                            let res = (
                                projectile.owner,
                                e,
                                Some(ProjectileCollision::Player(*other)),
                            );

                            events.push(res);
                        }

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

    for (damage_from_entity, projectile_entity, collision) in events {
        if let Some(collision_kind) = collision {
            match collision_kind {
                ProjectileCollision::Player(damage_to_entity) => {
                    on_player_damage(world, damage_from_entity, damage_to_entity);
                }
                ProjectileCollision::Trigger(trigger_entity) => {
                    let mut effect = world.get_mut::<TriggeredEffect>(trigger_entity).unwrap();
                    if !effect.should_override_delay {
                        effect.is_triggered = true;
                        effect.should_override_delay = true;
                        effect.triggered_by = Some(damage_from_entity);
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
