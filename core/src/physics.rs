use crate::prelude::*;

use self::collisions::{Actor, Collider, TileCollision};
pub use collisions::CollisionWorld;

pub mod collisions;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PostUpdate, hydrate_physics_bodies)
        .add_system_to_stage(CoreStage::PostUpdate, update_kinematic_bodies);
}

/// A kinematic physics body
///
/// Used primarily for players and things that need to walk around, detect what kind of platform
/// they are standing on, etc.
///
/// For now, all kinematic bodies have axis-aligned, rectangular colliders. This may or may not change in the future.
#[derive(Default, Debug, Clone, Copy, TypeUlid)]
#[ulid = "01GNF5NRJR7CXZ9NKEBQEGN8R1"]
#[repr(C)]
pub struct KinematicBody {
    pub velocity: Vec2,
    pub offset: Vec2,
    pub size: Vec2,
    /// Angular velocity in degrees per second
    pub angular_velocity: f32,
    pub gravity: f32,
    pub bounciness: f32,
    pub is_on_ground: bool,
    pub was_on_ground: bool,
    /// Will be `true` if the body is currently on top of a platform/jumpthrough tile
    pub is_on_platform: bool,
    /// If this is `true` the body will be affected by gravity
    pub has_mass: bool,
    pub has_friction: bool,
    pub can_rotate: bool,
    pub is_deactivated: bool,
    /// Whether or not the body should fall through jump_through platforms
    pub fall_through: bool,
    /// Indicates that we should reset the collider like it was just added to the world.
    ///
    /// This is important to make sure that it falls through JumpThrough platforms if it happens to
    /// spawn inside of one.
    pub is_spawning: bool,
}

impl KinematicBody {
    pub fn collider_rect(&self, position: Vec3) -> Rect {
        Rect::new(
            position.x + self.offset.x,
            position.y + self.offset.y,
            self.size.x,
            self.size.y,
        )
    }
}

fn hydrate_physics_bodies(
    entities: Res<Entities>,
    transforms: Comp<Transform>,
    bodies: Comp<KinematicBody>,
    mut colliders: CompMut<Collider>,
    mut actors: CompMut<Actor>,
) {
    let mut needs_hydrate = colliders.bitset().clone();
    needs_hydrate.bit_not();
    needs_hydrate.bit_and(bodies.bitset());

    for entity in entities.iter_with_bitset(&needs_hydrate) {
        let transform = transforms.get(entity).unwrap();
        let body = bodies.get(entity).unwrap();

        if body.size.x.round() as i32 % 2 != 0 || body.size.y.round() as i32 % 2 != 0 {
            warn!(
                "TODO: Non-even widths and heights for colliders may currently \
                behave incorrectly, getting stuck in walls."
            );
        }

        colliders.insert(
            entity,
            Collider {
                pos: transform.translation.truncate() + body.offset,
                width: body.size.x,
                height: body.size.y,
                ..default()
            },
        );
        actors.insert(entity, Actor);
    }
}

fn update_kinematic_bodies(
    game: Res<CoreMetaArc>,
    entities: Res<Entities>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut collision_world: CollisionWorld,
) {
    for (entity, (body, transform)) in entities.iter_with((&mut bodies, &mut transforms)) {
        if body.is_deactivated {
            continue;
        }

        let mut position = transform.translation.truncate() + body.offset;

        if body.has_mass {
            // Shove objects out of walls
            while collision_world.collide_solids(position, body.size.x, body.size.y)
                == TileCollision::SOLID
            {
                let rect = Rect::new(position.x, position.y, body.size.x, body.size.y);

                match (
                    collision_world.collide_tag(default(), rect.top_left(), 0.0, 0.0)
                        == TileCollision::SOLID,
                    collision_world.collide_tag(default(), rect.top_right(), 0.0, 0.0)
                        == TileCollision::SOLID,
                    collision_world.collide_tag(default(), rect.bottom_right(), 0.0, 0.0)
                        == TileCollision::SOLID,
                    collision_world.collide_tag(default(), rect.bottom_left(), 0.0, 0.0)
                        == TileCollision::SOLID,
                ) {
                    // Check for collisions on each side of the rectangle
                    (false, false, _, _) => position.y += 1.0,
                    (_, false, false, _) => position.x += 1.0,
                    (_, _, false, false) => position.y -= 1.0,
                    (false, _, _, false) => position.x -= 1.0,
                    // If none of the sides of the rectangle are un-collided, then we don't know
                    // which direction to move to get out of the wall, and we just give up.
                    _ => {
                        break;
                    }
                }
            }
        }

        if body.is_spawning {
            collision_world.add_actor(entity, position, body.size.x, body.size.y);
            body.is_spawning = false;
        }

        if body.fall_through {
            collision_world.descent(entity);
        }

        collision_world.set_actor_position(entity, position);

        {
            let position = position + vec2(0.0, -1.0);

            body.was_on_ground = body.is_on_ground;

            body.is_on_ground = collision_world.collide_check(entity, position);

            let tile = collision_world.collide_solids(position, body.size.x, body.size.y);
            body.is_on_platform = tile == TileCollision::JUMP_THROUGH;
        }

        if !collision_world.move_h(entity, body.velocity.x) {
            body.velocity.x *= -body.bounciness;
        }

        if !body.is_on_ground && body.has_mass {
            body.velocity.y -= body.gravity;

            if body.velocity.y < -game.physics.terminal_velocity {
                body.velocity.y = -game.physics.terminal_velocity;
            }
        }

        if !collision_world.move_v(entity, body.velocity.y) {
            body.velocity.y *= -body.bounciness;
        }

        if body.can_rotate {
            apply_rotation(
                transform,
                body.velocity,
                body.angular_velocity,
                body.is_on_ground,
            );
        }

        if body.is_on_ground && body.has_friction {
            body.velocity.x *= game.physics.friction_lerp;
            if body.velocity.x.abs() <= game.physics.stop_threshold {
                body.velocity.x = 0.0;
            }
        }

        transform.translation =
            (collision_world.actor_pos(entity) - body.offset).extend(transform.translation.z);
    }
}

fn apply_rotation(
    transform: &mut Transform,
    velocity: Vec2,
    angular_velocity: f32,
    is_on_ground: bool,
) {
    let mut angle = transform.rotation.to_euler(EulerRot::XYZ).2;

    if is_on_ground {
        angle += velocity.x.abs() * angular_velocity;
    } else {
        angle += (angular_velocity * crate::FPS).to_radians();
    }

    transform.rotation = Quat::from_rotation_z(angle);
}
