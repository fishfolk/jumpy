//! The crate item.
//!
//! This module is inconsistently named with the rest of the modules ( i.e. has an `_item` suffix )
//! because `crate` is a Rust keyword.

use crate::{physics::collisions::TileCollision, player::PlayerKillCommand};

use super::*;

pub struct CrateItemPlugin;

#[derive(Reflect, Component, Clone, Debug)]
#[reflect(Component)]
pub struct IdleCrateItem {
    /// The entity ID of the map element that spawned the crate
    spawner: Entity,
}

impl Default for IdleCrateItem {
    fn default() -> Self {
        Self {
            spawner: crate::utils::invalid_entity(),
        }
    }
}

#[derive(Reflect, Component, Clone, Debug)]
#[reflect(Component, Default)]
pub struct ThrownCrateItem {
    /// The entity ID of the map element that spawned the crate
    spawner: Entity,
    /// The entity ID of the player that threw the box
    owner: Entity,
    age: f32,
}

impl Default for ThrownCrateItem {
    fn default() -> Self {
        Self {
            spawner: crate::utils::invalid_entity(),
            owner: crate::utils::invalid_entity(),
            age: 0.0,
        }
    }
}

impl Plugin for CrateItemPlugin {
    fn build(&self, app: &mut App) {
        app.add_rollback_system(RollbackStage::PreUpdate, pre_update_in_game)
            .add_rollback_system(RollbackStage::Update, update_thrown_crates)
            .add_rollback_system(RollbackStage::Update, update_idle_crates)
            .extend_rollback_plugin(|plugin| {
                plugin
                    .register_rollback_type::<IdleCrateItem>()
                    .register_rollback_type::<ThrownCrateItem>()
            });
    }
}

fn pre_update_in_game(
    mut commands: Commands,
    non_hydrated_map_elements: Query<
        (Entity, &Sort, &Handle<MapElementMeta>, &Transform),
        Without<MapElementHydrated>,
    >,
    mut ridp: ResMut<RollbackIdProvider>,
    element_assets: ResMut<Assets<MapElementMeta>>,
) {
    // Hydrate any newly-spawned crates
    let mut elements = non_hydrated_map_elements.iter().collect::<Vec<_>>();
    elements.sort_by_key(|x| x.1);
    for (entity, _sort, map_element_handle, transform) in elements {
        let map_element = element_assets.get(map_element_handle).unwrap();
        if let BuiltinElementKind::Crate {
            body_size,
            body_offset,
            atlas_handle,
            ..
        } = &map_element.builtin
        {
            commands.entity(entity).insert(MapElementHydrated);

            commands
                .spawn()
                .insert(Rollback::new(ridp.next_id()))
                .insert(Item {
                    script: "core:crate".into(),
                })
                .insert(IdleCrateItem { spawner: entity })
                .insert(Name::new("Item: Crate"))
                .insert(AnimatedSprite {
                    start: 0,
                    end: 0,
                    atlas: atlas_handle.inner.clone(),
                    repeat: false,
                    ..default()
                })
                .insert(map_element_handle.clone_weak())
                .insert_bundle(VisibilityBundle::default())
                .insert(MapRespawnPoint(transform.translation))
                .insert_bundle(TransformBundle {
                    local: *transform,
                    ..default()
                })
                .insert(KinematicBody {
                    size: *body_size,
                    offset: *body_offset,
                    gravity: 1.0,
                    has_mass: true,
                    has_friction: true,
                    ..default()
                });
        }
    }
}

fn update_idle_crates(
    mut commands: Commands,
    players: Query<(&AnimatedSprite, &Transform, &KinematicBody), With<PlayerIdx>>,
    mut grenades: Query<
        (
            &Rollback,
            Entity,
            &IdleCrateItem,
            &mut Transform,
            &mut AnimatedSprite,
            &mut KinematicBody,
            &Handle<MapElementMeta>,
            Option<&Parent>,
            Option<&ItemUsed>,
            Option<&ItemDropped>,
        ),
        Without<PlayerIdx>,
    >,
    mut ridp: ResMut<RollbackIdProvider>,
    element_assets: ResMut<Assets<MapElementMeta>>,
) {
    let mut items = grenades.iter_mut().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0.id());
    for (
        _,
        item_ent,
        crate_item,
        mut transform,
        mut sprite,
        mut body,
        meta_handle,
        parent,
        used,
        dropped,
    ) in items
    {
        let meta = element_assets.get(meta_handle).unwrap();
        let BuiltinElementKind::Crate {
            grab_offset,
            atlas_handle,
            throw_velocity,
            ..
        } = &meta.builtin else {
            unreachable!();
        };

        // If the item is being held
        if let Some(parent) = parent {
            let (player_sprite, player_transform, player_body) =
                players.get(parent.get()).expect("Parent is not player");

            // Deactivate items while held
            body.is_deactivated = true;

            // Flip the sprite to match the player orientation
            let flip = player_sprite.flip_x;
            sprite.flip_x = flip;
            let flip_factor = if flip { -1.0 } else { 1.0 };
            let horizontal_flip_factor = Vec2::new(flip_factor, 1.0);
            transform.translation.x = grab_offset.x * flip_factor;
            transform.translation.y = grab_offset.y;
            transform.translation.z = 0.0;

            // If the item is being used
            if used.is_some() {
                // Despawn the item from the player's hand
                commands.entity(item_ent).despawn();

                // Spawn a new, lit grenade
                commands
                    .spawn()
                    .insert(Rollback::new(ridp.next_id()))
                    .insert(Name::new("Crate ( Thrown )"))
                    .insert(Transform::from_translation(
                        player_transform.translation
                            + (*grab_offset * horizontal_flip_factor).extend(0.0),
                    ))
                    .insert(GlobalTransform::default())
                    .insert(Visibility::default())
                    .insert(ComputedVisibility::default())
                    .insert(AnimatedSprite {
                        atlas: atlas_handle.inner.clone(),
                        ..default()
                    })
                    .insert(meta_handle.clone_weak())
                    .insert(body.clone())
                    .insert(ThrownCrateItem {
                        spawner: crate_item.spawner,
                        owner: parent.get(),
                        ..default()
                    })
                    .insert(KinematicBody {
                        velocity: *throw_velocity * horizontal_flip_factor + player_body.velocity,
                        is_deactivated: false,
                        fall_through: true,
                        ..body.clone()
                    });
            }
        }

        // If the item is dropped
        if let Some(dropped) = dropped {
            commands.entity(item_ent).remove::<ItemDropped>();
            let (player_sprite, player_transform, player_body) =
                players.get(dropped.player).expect("Parent is not a player");

            // Re-activate physics
            body.is_deactivated = false;

            // Put sword in rest position
            sprite.start = 0;
            sprite.end = 0;
            body.velocity = player_body.velocity;
            body.is_spawning = true;

            let horizontal_flip_factor = if player_sprite.flip_x {
                Vec2::new(-1.0, 1.0)
            } else {
                Vec2::ONE
            };

            // Drop item at player position
            transform.translation =
                player_transform.translation + (*grab_offset * horizontal_flip_factor).extend(0.0);
        }
    }
}

fn update_thrown_crates(
    mut commands: Commands,
    players: Query<Entity, With<PlayerIdx>>,
    mut crates: Query<
        (
            &Rollback,
            Entity,
            &mut ThrownCrateItem,
            &Transform,
            &Handle<MapElementMeta>,
        ),
        Without<PlayerIdx>,
    >,
    mut ridp: ResMut<RollbackIdProvider>,
    element_assets: ResMut<Assets<MapElementMeta>>,
    player_inputs: Res<PlayerInputs>,
    effects: Res<AudioChannel<EffectsChannel>>,
    collision_world: CollisionWorld,
) {
    let mut items = crates.iter_mut().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0.id());
    for (_, item_ent, mut crate_item, transform, meta_handle) in items {
        let meta = element_assets.get(meta_handle).unwrap();
        let BuiltinElementKind::Crate {
            breaking_atlas_handle,
            breaking_anim_fps,
            breaking_anim_frames: breaking_anim_length,
            break_sound_handle,
            break_timeout,
            ..
        } = &meta.builtin else {
            unreachable!();
        };
        let frame_time = 1.0 / crate::FPS as f32;

        crate_item.age += frame_time;

        let colliding_with_wall = {
            let collider = collision_world.get_collider(item_ent);
            let width = collider.width + 2.0;
            let height = collider.width + 2.0;
            let pos = transform.translation.truncate();
            collision_world.collide_solids(pos, width, height) == TileCollision::Solid
        };

        let colliding_with_players = collision_world
            .actor_collisions(item_ent)
            .into_iter()
            .filter(|&x| x != crate_item.owner && players.contains(x))
            .collect::<Vec<_>>();

        for &player in &colliding_with_players {
            commands.add(PlayerKillCommand::new(player));
        }

        if !colliding_with_players.is_empty()
            || colliding_with_wall
            || crate_item.age > *break_timeout
        {
            if player_inputs.is_confirmed {
                effects.play(break_sound_handle.clone_weak());
            }

            // Despawn the grenade
            commands.entity(item_ent).despawn();
            // Cause the item to re-spawn by re-triggering spawner hydration
            commands
                .entity(crate_item.spawner)
                .remove::<MapElementHydrated>();

            // Spawn the explosion sprite entity
            commands
                .spawn()
                .insert(Rollback::new(ridp.next_id()))
                .insert(*transform)
                .insert(GlobalTransform::default())
                .insert(Visibility::default())
                .insert(ComputedVisibility::default())
                .insert(AnimatedSprite {
                    start: 0,
                    end: *breaking_anim_length,
                    atlas: breaking_atlas_handle.inner.clone(),
                    repeat: false,
                    fps: *breaking_anim_fps,
                    ..default()
                })
                .insert(Lifetime::new(
                    *breaking_anim_fps * *breaking_anim_length as f32,
                ));
        }
    }
}
