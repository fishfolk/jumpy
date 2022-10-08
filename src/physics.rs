use bevy_inspector_egui_rapier::InspectableRapierPlugin;
use bevy_rapier2d::prelude as rapier;

use crate::{config::ENGINE_CONFIG, metadata::GameMeta, prelude::*};

mod debug;

pub struct PhysicsPlugin;

#[derive(StageLabel)]
pub enum GamePhysicsStages {
    /// Runs before [`GamePhysicsStages::PreSync`] to make sure that all entities with a
    /// [`KinematicBody`] or a [`RigidBody`] have the required rapier components.
    Hydrate,
    /// Runs before rapier's [`rapier::PhysicsStages::SyncBackend`] to update the rapier components with the
    /// data form our custom physics components before rapier performs the next simulation step.
    PreSync,
    /// Runs after rapier's [`rapier::PhysicsStages::Writeback`] to update the data in our custom physics
    /// components with the collisions detected by rapier.
    PostWriteback,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(rapier::RapierPhysicsPlugin::<rapier::NoUserData>::default())
            .register_type::<KinematicBody>()
            .add_stage_before(
                rapier::PhysicsStages::SyncBackend,
                GamePhysicsStages::PreSync,
                SystemStage::parallel().with_system(
                    sync_kinematic_bodies
                        .run_in_state(GameState::InGame)
                        .run_not_in_state(InGameState::Paused),
                ),
            )
            .add_stage_before(
                GamePhysicsStages::PreSync,
                GamePhysicsStages::Hydrate,
                SystemStage::parallel().with_system(
                    hydrate_physics_bodies
                        .run_in_state(GameState::InGame)
                        .run_not_in_state(InGameState::Paused),
                ),
            )
            .add_stage_after(
                rapier::PhysicsStages::Writeback,
                GamePhysicsStages::PostWriteback,
                SystemStage::parallel().with_system(
                    writeback_kinematic_bodies
                        .run_in_state(GameState::InGame)
                        .run_not_in_state(InGameState::Paused),
                ),
            );

        // Add debug plugins if enabled
        if ENGINE_CONFIG.debug_tools {
            app.insert_resource(rapier::DebugRenderContext {
                enabled: false,
                ..default()
            })
            .add_plugin(InspectableRapierPlugin)
            .add_plugin(debug::DebugRenderPlugin);
        }
    }
}

/// A kinematic physics body
///
/// Used primarily for players and things that need to walk around, detect what kind of platform
/// they are standing on, etc.
///
/// For now, all kinematic bodies have axis-aligned, rectangular colliders. This may or may not change in the future.
#[derive(Reflect, Component, Default, Debug, Clone)]
#[reflect(Component, Default)]
pub struct KinematicBody {
    pub offset: Vec2,
    pub size: Vec2,
    pub velocity: Vec2,
    pub is_on_ground: bool,
    pub was_on_ground: bool,
    // TODO: Platform handling
    // /// Will be `true` if the body is currently on top of a platform/jumpthrough tile
    // pub is_on_platform: bool,
    /// If this is `true` the body will be affected by gravity
    pub has_mass: bool,
    pub has_friction: bool,
    // TODO: Rotations and angular velocity
    // pub can_rotate: bool,
    pub bouncyness: f32,
    pub is_deactivated: bool,
    pub gravity: f32,
}

impl KinematicBody {
    fn collider(&self) -> rapier::Collider {
        rapier::Collider::cuboid(self.size.x / 2.0, self.size.y / 2.0)
    }
}

#[derive(Component)]
struct KinematicBodyCollider;

fn hydrate_physics_bodies(
    mut commands: Commands,
    bodies: Query<Entity, (With<KinematicBody>, Without<rapier::RigidBody>)>,
) {
    for entity in &bodies {
        commands
            .entity(entity)
            .insert(rapier::RigidBody::KinematicPositionBased)
            .insert(rapier::LockedAxes::ROTATION_LOCKED)
            .with_children(|parent| {
                parent
                    .spawn()
                    .insert_bundle(TransformBundle::default())
                    .insert(rapier::Collider::default())
                    .insert(KinematicBodyCollider);
            });
    }
}

fn sync_kinematic_bodies(
    mut bodies: Query<(&KinematicBody, &Children)>,
    mut colliders: Query<(&mut rapier::Collider, &mut Transform), With<KinematicBodyCollider>>,
) {
    for (body, children) in &mut bodies {
        if let Some((mut collider, mut collider_transform)) = children
            .get(0)
            .and_then(|entity| colliders.get_mut(*entity).ok())
        {
            collider_transform.translation.x = body.offset.x;
            collider_transform.translation.y = body.offset.y;
            *collider = body.collider();
        }
    }
}

fn writeback_kinematic_bodies(
    game: Res<GameMeta>,
    ctx: Res<rapier::RapierContext>,
    mut bodies: Query<(&mut KinematicBody, &mut Transform)>,
    time: Res<Time>,
) {
    let dt = time.delta_seconds();
    for (mut body, mut transform) in &mut bodies {
        if body.is_deactivated {
            continue;
        }

        let pos = &mut transform.translation;

        let collider = body.collider();

        body.was_on_ground = body.is_on_ground;

        let ground_collision = ctx.cast_shape(
            pos.truncate() + body.offset,
            default(),
            Vec2::NEG_Y,
            &collider,
            1.0,
            rapier::QueryFilter {
                flags: rapier::QueryFilterFlags::ONLY_FIXED,
                ..default()
            },
        );

        if let Some((_, toi)) = ground_collision {
            body.is_on_ground = true;

            // Move the body flush with the ground
            pos.y -= toi.toi;
        }

        if !body.is_on_ground && body.has_mass {
            body.velocity.y -= body.gravity * dt;

            if body.velocity.y < -game.physics.terminal_velocity {
                body.velocity.y = -game.physics.terminal_velocity;
            }
        }

        let horizontal_collision = ctx.cast_shape(
            pos.truncate() + body.offset,
            default(),
            Vec2::new(body.velocity.x.signum(), 0.0),
            &collider,
            body.velocity.x.abs() * dt,
            rapier::QueryFilter {
                flags: rapier::QueryFilterFlags::ONLY_FIXED,
                ..default()
            },
        );
        if let Some((_, toi)) = horizontal_collision {
            body.velocity.x *= -body.bouncyness;
            pos.x += toi.toi * body.velocity.x.signum();
        } else {
            pos.x += body.velocity.x * dt;
        }

        let vertical_collision = ctx.cast_shape(
            pos.truncate() + body.offset,
            default(),
            Vec2::new(body.velocity.y.signum(), 0.0),
            &collider,
            body.velocity.y.abs() * dt,
            rapier::QueryFilter {
                flags: rapier::QueryFilterFlags::ONLY_FIXED,
                ..default()
            },
        );
        if let Some((_, toi)) = vertical_collision {
            body.velocity.y *= -body.bouncyness;
            pos.y += toi.toi * body.velocity.y.signum();
        } else {
            pos.y += body.velocity.y * dt;
        }

        if body.is_on_ground && body.has_friction {
            body.velocity.x *= game.physics.friction_lerp;
            if body.velocity.x.abs() <= game.physics.stop_threshold {
                body.velocity.x = 0.0;
            }
        }
    }
}
