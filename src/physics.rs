use bevy::math::vec2;

use crate::{config::ENGINE_CONFIG, metadata::GameMeta, prelude::*};

use self::collisions::{Actor, Collider, CollisionWorld, Rect, TileCollision};

pub mod collisions;
pub mod debug;

pub struct PhysicsPlugin;

#[derive(StageLabel)]
pub enum PhysicsStages {
    Hydrate,
    UpdatePhysics,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<KinematicBody>()
            .register_type::<Collider>()
            .extend_rollback_schedule(|schedule| {
                schedule
                    .add_stage_after(
                        RollbackStage::PostUpdate,
                        PhysicsStages::Hydrate,
                        SystemStage::parallel().with_system(hydrate_physics_bodies),
                    )
                    .add_stage_after(
                        PhysicsStages::Hydrate,
                        PhysicsStages::UpdatePhysics,
                        SystemStage::parallel().with_system(
                            update_kinematic_bodies
                                .run_in_state(GameState::InGame)
                                .run_not_in_state(InGameState::Paused),
                        ),
                    );
            });

        if ENGINE_CONFIG.debug_tools {
            app.add_plugin(debug::PhysicsDebugRenderPlugin);
        }
    }
}

/// A kinematic physics body
///
/// Used primarily for players and things that need to walk around, detect what kind of platform
/// they are standing on, etc.
///
/// For now, all kinematic bodies have axis-aligned, rectangular colliders. This may or may not change in the future.
#[derive(Reflect, Component, Default, Debug, Clone, Serialize, Deserialize)]
#[reflect(Component, Default)]
pub struct KinematicBody {
    pub velocity: Vec2,
    pub offset: Vec2,
    pub size: Vec2,
    /// Angular velocity in degrees per second
    pub angular_velocity: f32,
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

#[derive(Component)]
struct KinematicBodyCollider;

fn hydrate_physics_bodies(
    mut commands: Commands,
    bodies: Query<(Entity, &Transform, &KinematicBody), Without<Collider>>,
) {
    for (entity, transform, body) in &bodies {
        if body.size.x.round() as i32 % 2 != 0 || body.size.y.round() as i32 % 2 != 0 {
            warn!(
                "TODO: Non-even widths and heights for colliders may currently \
                behave incorrectly, getting stuck in walls."
            );
        }
        commands
            .entity(entity)
            .insert(Collider {
                pos: transform.translation.truncate() + body.offset,
                width: body.size.x,
                height: body.size.y,
                ..default()
            })
            .insert(Actor);
    }
}

fn update_kinematic_bodies(
    game: Res<GameMeta>,
    mut collision_world: CollisionWorld,
    mut bodies: Query<(Entity, &mut KinematicBody, &mut Transform)>,
) {
    for (actor, mut body, mut transform) in &mut bodies {
        if body.is_deactivated {
            continue;
        }

        let mut position = transform.translation.truncate() + body.offset;

        // Shove objects out of walls
        while collision_world.collide_solids(position, body.size.x, body.size.y)
            == TileCollision::Solid
        {
            let rect = collisions::Rect::new(position.x, position.y, body.size.x, body.size.y);

            match (
                collision_world.collide_tag(1, rect.top_left(), 0.0, 0.0) == TileCollision::Solid,
                collision_world.collide_tag(1, rect.top_right(), 0.0, 0.0) == TileCollision::Solid,
                collision_world.collide_tag(1, rect.bottom_right(), 0.0, 0.0)
                    == TileCollision::Solid,
                collision_world.collide_tag(1, rect.bottom_left(), 0.0, 0.0)
                    == TileCollision::Solid,
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

        if body.is_spawning {
            collision_world.add_actor(actor, position, body.size.x, body.size.y);
            body.is_spawning = false;
        }

        if body.fall_through {
            collision_world.descent(actor);
        }

        collision_world.set_actor_position(actor, position);

        {
            let position = position + vec2(0.0, -1.0);

            body.was_on_ground = body.is_on_ground;

            body.is_on_ground = collision_world.collide_check(actor, position);

            let tile = collision_world.collide_solids(position, body.size.x, body.size.y);
            body.is_on_platform = tile == TileCollision::JumpThrough;
        }

        if !collision_world.move_h(actor, body.velocity.x) {
            body.velocity.x *= -body.bouncyness;
        }

        if !collision_world.move_v(actor, body.velocity.y) {
            body.velocity.y *= -body.bouncyness;
        }

        if !body.is_on_ground && body.has_mass {
            body.velocity.y -= body.gravity;

            if body.velocity.y < -game.physics.terminal_velocity {
                body.velocity.y = -game.physics.terminal_velocity;
            }
        }

        if body.can_rotate {
            apply_rotation(
                &mut transform,
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
            (collision_world.actor_pos(actor) - body.offset).extend(transform.translation.z);
    }
}

fn apply_rotation(
    transform: &mut Transform,
    velocity: Vec2,
    angular_velocity: f32,
    is_on_ground: bool,
) {
    let mut angle = transform.rotation.to_euler(EulerRot::XYZ).2;
    if angular_velocity != 0.0 {
        angle += (angular_velocity * crate::FPS as f32).to_radians();
    } else if !is_on_ground {
        angle += velocity.x.abs() * 0.00045 + velocity.y.abs() * 0.00015;
    } else {
        angle %= std::f32::consts::PI * 2.0;

        let goal = std::f32::consts::PI * 2.0;

        let rest = goal - angle;
        if rest.abs() >= 0.1 {
            angle += (rest * 0.1).max(0.1);
        }
    }

    transform.rotation = Quat::from_rotation_z(angle);
}
