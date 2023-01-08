//! The crate item.
//!
//! This module is inconsistently named with the rest of the modules ( i.e. has an `_item` suffix )
//! because `crate` is a Rust keyword.

use crate::{networking::RollbackIdWrapper, player::PlayerKillCommand};

use super::*;

pub struct MinePlugin;

#[derive(Reflect, Component, Clone, Debug)]
#[reflect(Component)]
pub struct IdleMine {
    /// The entity ID of the map element that spawned the crate
    spawner: Entity,
}

impl Default for IdleMine {
    fn default() -> Self {
        Self {
            spawner: crate::utils::invalid_entity(),
        }
    }
}

#[derive(Reflect, Component, Clone, Debug)]
#[reflect(Component, Default)]
pub struct ThrownMine {
    /// The entity ID of the map element that spawned the crate
    spawner: Entity,
    age: f32,
}

impl Default for ThrownMine {
    fn default() -> Self {
        Self {
            spawner: crate::utils::invalid_entity(),
            age: 0.0,
        }
    }
}

impl Plugin for MinePlugin {
    fn build(&self, app: &mut App) {
        app.add_rollback_system(RollbackStage::PreUpdate, pre_update_in_game)
            .add_rollback_system(RollbackStage::Update, update_thrown_mines)
            .add_rollback_system(RollbackStage::Update, update_idle_mines)
            .extend_rollback_plugin(|plugin| {
                plugin
                    .register_rollback_type::<IdleMine>()
                    .register_rollback_type::<ThrownMine>()
            });
    }
}

fn pre_update_in_game(
    mut commands: Commands,
    non_hydrated_map_elements: Query<
        (Entity, &Sort, &Handle<MapElementMeta>, &Transform),
        Without<MapElementHydrated>,
    >,
    mut ridp: ResMut<RollbackIdWrapper>,
    element_assets: ResMut<Assets<MapElementMeta>>,
) {
    // Hydrate any newly-spawned crates
    let mut elements = non_hydrated_map_elements.iter().collect::<Vec<_>>();
    elements.sort_by_key(|x| x.1);
    for (entity, _sort, map_element_handle, transform) in elements {
        let map_element = element_assets.get(map_element_handle).unwrap();
        if let BuiltinElementKind::Mine {
            body_size,
            body_offset,
            atlas_handle,
            ..
        } = &map_element.builtin
        {
            commands.entity(entity).insert(MapElementHydrated);

            commands.spawn((
                Rollback::new(ridp.next_id()),
                Item {
                    script: "core:mine".into(),
                },
                IdleMine { spawner: entity },
                Name::new("Item: Mine"),
                AnimatedSprite {
                    start: 0,
                    end: 0,
                    atlas: atlas_handle.inner.clone(),
                    repeat: false,
                    ..default()
                },
                map_element_handle.clone_weak(),
                VisibilityBundle::default(),
                MapRespawnPoint(transform.translation),
                TransformBundle {
                    local: *transform,
                    ..default()
                },
                KinematicBody {
                    size: *body_size,
                    offset: *body_offset,
                    gravity: 1.0,
                    has_mass: true,
                    has_friction: true,
                    ..default()
                },
            ));
        }
    }
}

fn update_idle_mines(
    mut commands: Commands,
    players: Query<(&AnimatedSprite, &Transform, &KinematicBody), With<PlayerIdx>>,
    mut grenades: Query<
        (
            &Rollback,
            Entity,
            &IdleMine,
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
    mut ridp: ResMut<RollbackIdWrapper>,
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
        let BuiltinElementKind::Mine {
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

                // Spawn a new, thrown mine
                let pos = player_transform.translation
                    + (*grab_offset * horizontal_flip_factor).extend(0.0);
                commands.spawn((
                    Rollback::new(ridp.next_id()),
                    Name::new("Mine ( Thrown )"),
                    MapRespawnPoint(pos),
                    Transform::from_translation(pos),
                    GlobalTransform::default(),
                    Visibility::default(),
                    ComputedVisibility::default(),
                    AnimatedSprite {
                        atlas: atlas_handle.inner.clone(),
                        ..default()
                    },
                    meta_handle.clone_weak(),
                    body.clone(),
                    ThrownMine {
                        spawner: crate_item.spawner,
                        ..default()
                    },
                    KinematicBody {
                        velocity: *throw_velocity * horizontal_flip_factor + player_body.velocity,
                        is_deactivated: false,
                        ..body.clone()
                    },
                ));
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

fn update_thrown_mines(
    mut commands: Commands,
    players: Query<Entity, With<PlayerIdx>>,
    mut mines: Query<
        (
            &Rollback,
            Entity,
            &mut ThrownMine,
            &Transform,
            &mut AnimatedSprite,
            &Handle<MapElementMeta>,
        ),
        Without<PlayerIdx>,
    >,
    mut ridp: ResMut<RollbackIdWrapper>,
    element_assets: ResMut<Assets<MapElementMeta>>,
    player_inputs: Res<PlayerInputs>,
    effects: Res<AudioChannel<EffectsChannel>>,
    collision_world: CollisionWorld,
) {
    let mut items = mines.iter_mut().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0.id());
    for (_, item_ent, mut crate_item, transform, mut sprite, meta_handle) in items {
        let meta = element_assets.get(meta_handle).unwrap();
        let BuiltinElementKind::Mine {
            explosion_atlas_handle,
            explosion_anim_fps,
            explosion_anim_frames,
            explosion_sound_handle,
            arm_delay,
            arm_sound_handle,
            armed_anim_start,
            armed_anim_end,
            armed_anim_fps,
            damage_region_size,
            damage_region_lifetime,
            ..
        } = &meta.builtin else {
            unreachable!();
        };
        let frame_time = 1.0 / crate::FPS as f32;

        crate_item.age += frame_time;

        if crate_item.age >= *arm_delay && crate_item.age - *arm_delay < frame_time {
            if player_inputs.is_confirmed {
                effects.play(arm_sound_handle.clone_weak());
            }

            sprite.start = *armed_anim_start;
            sprite.end = *armed_anim_end;
            sprite.fps = *armed_anim_fps;
            sprite.repeat = true;
        }

        let colliding_with_players = collision_world
            .actor_collisions(item_ent)
            .into_iter()
            .filter(|&x| players.contains(x))
            .collect::<Vec<_>>();

        if !colliding_with_players.is_empty() && crate_item.age >= *arm_delay {
            for &player in &colliding_with_players {
                commands.add(PlayerKillCommand::new(player));
            }

            if player_inputs.is_confirmed {
                effects.play(explosion_sound_handle.clone_weak());
            }

            // Despawn the grenade
            commands.entity(item_ent).despawn();
            // Cause the item to re-spawn by re-triggering spawner hydration
            commands
                .entity(crate_item.spawner)
                .remove::<MapElementHydrated>();

            // Spawn the damage region entity
            commands.spawn((
                Rollback::new(ridp.next_id()),
                *transform,
                GlobalTransform::default(),
                Visibility::default(),
                ComputedVisibility::default(),
                DamageRegion {
                    size: *damage_region_size,
                },
                Lifetime::new(*damage_region_lifetime),
            ));

            // Spawn the explosion sprite entity
            commands.spawn((
                Rollback::new(ridp.next_id()),
                *transform,
                GlobalTransform::default(),
                Visibility::default(),
                ComputedVisibility::default(),
                AnimatedSprite {
                    start: 0,
                    end: *explosion_anim_frames,
                    atlas: explosion_atlas_handle.inner.clone(),
                    repeat: false,
                    fps: *explosion_anim_fps,
                    ..default()
                },
                Lifetime::new(*explosion_anim_fps * *explosion_anim_frames as f32),
            ));
        }
    }
}
