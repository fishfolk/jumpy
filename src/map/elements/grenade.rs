use bones_camera_shake::CameraTrauma;

use std::time::Duration;

use crate::networking::RollbackIdWrapper;

use super::*;

pub struct GrenadePlugin;

#[derive(Reflect, Component, Clone, Debug)]
#[reflect(Component)]
pub struct IdleGrenade {
    /// The entity ID of the map element that spawned the grenade
    spawner: Entity,
}

impl Default for IdleGrenade {
    fn default() -> Self {
        Self {
            spawner: crate::utils::invalid_entity(),
        }
    }
}

#[derive(Reflect, Component, Clone, Debug)]
#[reflect(Component, Default)]
pub struct LitGrenade {
    /// The entity ID of the map element that spawned the grenade
    spawner: Entity,
    fuse_sound: Handle<AudioInstance>,
    age: f32,
}

impl Default for LitGrenade {
    fn default() -> Self {
        Self {
            spawner: crate::utils::invalid_entity(),
            fuse_sound: default(),
            age: 0.0,
        }
    }
}

impl Plugin for GrenadePlugin {
    fn build(&self, app: &mut App) {
        app.add_rollback_system(RollbackStage::PreUpdate, pre_update_in_game)
            .add_rollback_system(RollbackStage::Update, update_lit_grenades)
            .add_rollback_system(RollbackStage::Update, update_idle_grenades)
            .extend_rollback_plugin(|plugin| {
                plugin
                    .register_rollback_type::<IdleGrenade>()
                    .register_rollback_type::<LitGrenade>()
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
    // Hydrate any newly-spawned grenades
    let mut elements = non_hydrated_map_elements.iter().collect::<Vec<_>>();
    elements.sort_by_key(|x| x.1);
    for (entity, _sort, map_element_handle, transform) in elements {
        let map_element = element_assets.get(map_element_handle).unwrap();
        if let BuiltinElementKind::Grenade {
            body_size,
            body_offset,
            atlas_handle,
            can_rotate,
            bouncyness,
            ..
        } = &map_element.builtin
        {
            commands.entity(entity).insert(MapElementHydrated);

            commands.spawn((
                Rollback::new(ridp.next_id()),
                Item {
                    script: "core:grenade".into(),
                },
                IdleGrenade { spawner: entity },
                Name::new("Item: Grenade"),
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
                    can_rotate: *can_rotate,
                    bouncyness: *bouncyness,
                    ..default()
                },
            ));
        }
    }
}

fn update_idle_grenades(
    mut commands: Commands,
    players: Query<(&AnimatedSprite, &Transform, &KinematicBody), With<PlayerIdx>>,
    mut grenades: Query<
        (
            &Rollback,
            Entity,
            &IdleGrenade,
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
    element_assets: ResMut<Assets<MapElementMeta>>,
    effects: Res<AudioChannel<EffectsChannel>>,
) {
    let mut items = grenades.iter_mut().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0.id());
    for (
        _,
        item_ent,
        grenade,
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
        let BuiltinElementKind::Grenade {
            grab_offset,
            fuse_sound_handle,
            angular_velocity,
            ..
        } = &meta.builtin else {
            unreachable!();
        };

        // If the item is being held
        if let Some(parent) = parent {
            let (player_sprite, ..) = players.get(parent.get()).expect("Parent is not player");

            // Deactivate items while held
            body.is_deactivated = true;

            // Flip the sprite to match the player orientation
            let flip = player_sprite.flip_x;
            sprite.flip_x = flip;
            let flip_factor = if flip { -1.0 } else { 1.0 };
            transform.translation.x = grab_offset.x * flip_factor;
            transform.translation.y = grab_offset.y;
            transform.translation.z = 0.0;

            // If the item is being used
            if used.is_some() {
                // Light the grenade
                sprite.start = 3;
                sprite.end = 5;
                sprite.repeat = true;
                sprite.fps = 8.0;
                body.angular_velocity = *angular_velocity;
                commands
                    .entity(item_ent)
                    .remove::<IdleGrenade>()
                    .insert(LitGrenade {
                        spawner: grenade.spawner,
                        fuse_sound: effects.play(fuse_sound_handle.clone_weak()).handle(),
                        ..default()
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

fn update_lit_grenades(
    mut commands: Commands,
    players: Query<(&AnimatedSprite, &Transform, &KinematicBody), With<PlayerIdx>>,
    mut grenades: Query<
        (
            &Rollback,
            Entity,
            &mut LitGrenade,
            &mut Transform,
            &GlobalTransform,
            &mut KinematicBody,
            &mut AnimatedSprite,
            &Handle<MapElementMeta>,
            Option<&Parent>,
            Option<&ItemDropped>,
        ),
        Without<PlayerIdx>,
    >,
    mut ridp: ResMut<RollbackIdWrapper>,
    element_assets: ResMut<Assets<MapElementMeta>>,
    player_inputs: Res<PlayerInputs>,
    effects: Res<AudioChannel<EffectsChannel>>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    mut shake_event: EventWriter<CameraTrauma>,
) {
    let mut items = grenades.iter_mut().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0.id());
    for (
        _,
        item_ent,
        mut grenade,
        mut transform,
        global_transform,
        mut body,
        mut sprite,
        meta_handle,
        parent,
        dropped,
    ) in items
    {
        let meta = element_assets.get(meta_handle).unwrap();
        let BuiltinElementKind::Grenade {
            fuse_time,
            damage_region_size,
            damage_region_lifetime,
            explosion_atlas_handle,
            explosion_lifetime,
            explosion_fps,
            explosion_frames,
            explosion_sound_handle,
            grab_offset,
            throw_velocity,
            ..
        } = &meta.builtin else {
            unreachable!();
        };

        grenade.age += 1.0 / crate::FPS as f32;

        if let Some(parent) = parent {
            let (player_sprite, ..) = players.get(parent.get()).expect("Parent is not player");

            // Deactivate items while held
            body.is_deactivated = true;

            // Flip the sprite to match the player orientation
            let flip = player_sprite.flip_x;
            sprite.flip_x = flip;
            let flip_factor = if flip { -1.0 } else { 1.0 };
            transform.translation.x = grab_offset.x * flip_factor;
            transform.translation.y = grab_offset.y;
            transform.translation.z = 0.0;
        }

        // If the item is dropped
        if let Some(dropped) = dropped {
            commands.entity(item_ent).remove::<ItemDropped>();
            let (player_sprite, player_transform, player_body) =
                players.get(dropped.player).expect("Parent is not a player");

            // Re-activate physics
            body.is_deactivated = false;

            let horizontal_flip_factor = if sprite.flip_x {
                Vec2::new(-1.0, 1.0)
            } else {
                Vec2::ONE
            };
            body.velocity = *throw_velocity * horizontal_flip_factor + player_body.velocity;
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

        if grenade.age >= *fuse_time {
            if player_inputs.is_confirmed {
                effects.play(explosion_sound_handle.clone_weak());
                audio_instances
                    .get_mut(&grenade.fuse_sound)
                    .map(|x| x.stop(AudioTween::linear(Duration::from_secs_f32(0.1))));
            }

            shake_event.send(CameraTrauma(0.5));

            // Despawn the grenade
            commands.entity(item_ent).despawn();
            // Cause the item to re-spawn by re-triggering spawner hydration
            commands
                .entity(grenade.spawner)
                .remove::<MapElementHydrated>();

            // Spawn the damage region entity
            let mut spawn_transform = global_transform.compute_transform();
            spawn_transform.rotation = Quat::IDENTITY;

            commands.spawn((
                Rollback::new(ridp.next_id()),
                spawn_transform,
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
                spawn_transform,
                GlobalTransform::default(),
                Visibility::default(),
                ComputedVisibility::default(),
                AnimatedSprite {
                    start: 0,
                    end: *explosion_frames,
                    atlas: explosion_atlas_handle.inner.clone(),
                    repeat: false,
                    fps: *explosion_fps,
                    ..default()
                },
                Lifetime::new(*explosion_lifetime),
            ));
        }
    }
}
