use serde::{Deserialize, Serialize};

use ff_core::ecs::World;

use crate::Map;

use ff_core::prelude::*;

use crate::Result;

pub const GRAVITY: f32 = 2.5;
pub const TERMINAL_VELOCITY: f32 = 10.0;

const FRICTION_LERP: f32 = 0.96;
const STOP_THRESHOLD: f32 = 1.0;

#[derive(Debug, Clone)]
pub struct PhysicsBodyParams {
    pub size: Size<f32>,
    pub offset: Vec2,
    pub has_mass: bool,
    pub has_friction: bool,
    pub can_rotate: bool,
    pub bouncyness: f32,
    pub gravity: f32,
}

impl Default for PhysicsBodyParams {
    fn default() -> Self {
        PhysicsBodyParams {
            size: Size::new(16.0, 16.0),
            offset: Vec2::ZERO,
            has_mass: true,
            has_friction: true,
            can_rotate: true,
            bouncyness: 0.0,
            gravity: GRAVITY,
        }
    }
}

/// Regular simulated physics bodies.
/// Note that rotation is abstract, only set on the transform to be used for draws. The colliders
/// are axis-aligned and will not be affected by rotation.
pub struct PhysicsBody {
    pub actor: Actor,
    pub offset: Vec2,
    pub size: Size<f32>,
    pub velocity: Vec2,
    pub is_on_ground: bool,
    pub was_on_ground: bool,
    /// Will be `true` if the body is currently on top of a platform/jumpthrough tile
    pub is_on_platform: bool,
    /// If this is `true` the body will be affected by gravity
    pub has_mass: bool,
    pub has_friction: bool,
    pub can_rotate: bool,
    pub bouncyness: f32,
    pub is_deactivated: bool,
    pub gravity: f32,
}

impl PhysicsBody {
    pub fn new<V: Into<Option<Vec2>>>(
        actor: Actor,
        velocity: V,
        params: PhysicsBodyParams,
    ) -> Self {
        let velocity = velocity.into().unwrap_or_default();

        PhysicsBody {
            actor,
            offset: params.offset,
            size: params.size,
            velocity,
            is_on_ground: false,
            was_on_ground: false,
            is_on_platform: false,
            has_mass: params.has_mass,
            has_friction: params.has_friction,
            can_rotate: params.can_rotate,
            bouncyness: params.bouncyness,
            is_deactivated: false,
            gravity: params.gravity,
        }
    }

    pub fn as_rect(&self, position: Vec2) -> Rect {
        let position = position + self.offset;
        Rect::new(position.x, position.y, self.size.width, self.size.height)
    }
}

pub fn fixed_update_physics_bodies(
    world: &mut World,
    _delta_time: f32,
    _integration_factor: f32,
) -> Result<()> {
    let mut physics = physics_world();

    for (_, (transform, body)) in world.query_mut::<(&mut Transform, &mut PhysicsBody)>() {
        physics.set_actor_position(body.actor, transform.position + body.offset);

        if !body.is_deactivated {
            let position = physics.actor_position(body.actor);

            {
                let position = position + vec2(0.0, 1.0);

                body.was_on_ground = body.is_on_ground;

                body.is_on_ground = physics.collide_at(body.actor, position);

                // FIXME: Using this to set `is_on_ground` caused weird glitching behavior when jumping up through platforms
                let tile = physics.collide_solids_at(position, body.size);

                body.is_on_platform = tile == ColliderKind::Platform;
            }

            if !body.is_on_ground && body.has_mass {
                body.velocity.y += body.gravity;

                if body.velocity.y > TERMINAL_VELOCITY {
                    body.velocity.y = TERMINAL_VELOCITY;
                }
            }

            if !physics.move_actor_x(body.actor, body.velocity.x) {
                body.velocity.x *= -body.bouncyness;
            }

            if !physics.move_actor_y(body.actor, body.velocity.y) {
                body.velocity.y *= -body.bouncyness;
            }

            if body.can_rotate {
                apply_rotation(transform, &mut body.velocity, body.is_on_ground);
            }

            if body.is_on_ground && body.has_friction {
                body.velocity.x *= FRICTION_LERP;
                if body.velocity.x.abs() <= STOP_THRESHOLD {
                    body.velocity.x = 0.0;
                }
            }

            transform.position = physics.actor_position(body.actor) - body.offset;
        }
    }

    Ok(())
}

pub fn debug_draw_physics_bodies(world: &mut World, _delta_time: f32) -> Result<()> {
    for (_, (transform, body)) in world.query::<(&Transform, &PhysicsBody)>().iter() {
        if !body.is_deactivated {
            let rect = body.as_rect(transform.position);

            let color = if body.is_on_platform {
                colors::YELLOW
            } else if body.is_on_ground {
                colors::RED
            } else {
                colors::GREEN
            };

            draw_rectangle_outline(rect.x, rect.y, rect.width, rect.height, 2.0, color);
        }
    }

    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidBodyParams {
    #[serde(with = "ff_core::parsing::vec2_def")]
    pub offset: Vec2,
    pub size: Size<f32>,
    #[serde(default, skip_serializing_if = "ff_core::parsing::is_false")]
    pub can_rotate: bool,
}

impl Default for RigidBodyParams {
    fn default() -> Self {
        RigidBodyParams {
            offset: Vec2::ZERO,
            size: Size::new(16.0, 16.0),
            can_rotate: false,
        }
    }
}

/// Simple physics bodies that has a velocity and optional rotation.
/// Note that rotation is abstract, only set on the transform to be used for draws. The colliders
/// are axis-aligned and will not be affected by rotation.
pub struct RigidBody {
    pub offset: Vec2,
    pub size: Size<f32>,
    pub velocity: Vec2,
    pub can_rotate: bool,
}

impl RigidBody {
    pub fn new<V: Into<Option<Vec2>>>(velocity: V, params: RigidBodyParams) -> Self {
        let velocity = velocity.into().unwrap_or_default();

        RigidBody {
            offset: params.offset,
            size: params.size,
            velocity,
            can_rotate: params.can_rotate,
        }
    }

    pub fn as_rect(&self, position: Vec2) -> Rect {
        let position = position + self.offset;
        Rect::new(position.x, position.y, self.size.width, self.size.height)
    }
}

pub fn fixed_update_rigid_bodies(
    world: &mut World,
    _delta_time: f32,
    _integration_factor: f32,
) -> Result<()> {
    for (_, (transform, body)) in world.query_mut::<(&mut Transform, &mut RigidBody)>() {
        transform.position += body.velocity;

        if body.can_rotate {
            apply_rotation(transform, &mut body.velocity, false);
        }
    }

    Ok(())
}

pub fn debug_draw_rigid_bodies(world: &mut World, _delta_time: f32) -> Result<()> {
    for (_, (transform, body)) in world.query::<(&Transform, &RigidBody)>().iter() {
        let rect = body.as_rect(transform.position);

        draw_rectangle_outline(rect.x, rect.y, rect.width, rect.height, 2.0, colors::RED)
    }

    Ok(())
}

fn apply_rotation(transform: &mut Transform, velocity: &mut Vec2, is_on_ground: bool) {
    if !is_on_ground {
        transform.rotation += velocity.x.abs() * 0.00045 + velocity.y.abs() * 0.00015;
    } else {
        transform.rotation %= std::f32::consts::PI * 2.0;

        let goal = std::f32::consts::PI * 2.0;

        let rest = goal - transform.rotation;
        if rest.abs() >= 0.1 {
            transform.rotation += (rest * 0.1).max(0.1);
        }
    }
}
