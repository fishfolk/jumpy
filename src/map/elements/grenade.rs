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
    age: f32,
}

impl Default for LitGrenade {
    fn default() -> Self {
        Self {
            spawner: crate::utils::invalid_entity(),
            age: 0.0,
        }
    }
}

impl Plugin for GrenadePlugin {
    fn build(&self, app: &mut App) {
        app.extend_rollback_schedule(|schedule| {
            schedule
                .add_system_to_stage(RollbackStage::PreUpdateInGame, pre_update_in_game)
                .add_system_to_stage(
                    RollbackStage::UpdateInGame,
                    update_lit_grenades.before(update_idle_grenades),
                )
                .add_system_to_stage(RollbackStage::UpdateInGame, update_idle_grenades);
        })
        .extend_rollback_plugin(|plugin| plugin.register_rollback_type::<IdleGrenade>());
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
    // Hydrate any newly-spawned grenades
    let mut elements = non_hydrated_map_elements.iter().collect::<Vec<_>>();
    elements.sort_by_key(|x| x.1);
    for (entity, _sort, map_element_handle, transform) in elements {
        let map_element = element_assets.get(map_element_handle).unwrap();
        if let BuiltinElementKind::Grenades {
            body_size,
            body_offset,
            atlas_handle,
            can_rotate,
            ..
        } = &map_element.builtin
        {
            commands.entity(entity).insert(MapElementHydrated);

            commands
                .spawn()
                .insert(Rollback::new(ridp.next_id()))
                .insert(Item {
                    script: "core:grenade".into(),
                })
                .insert(IdleGrenade { spawner: entity })
                .insert(EntityName("Item: Grenade".into()))
                .insert(AnimatedSprite {
                    start: 0,
                    end: 0,
                    atlas: atlas_handle.inner.clone(),
                    repeat: false,
                    ..default()
                })
                .insert(map_element_handle.clone())
                .insert_bundle(VisibilityBundle::default())
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
                    can_rotate: *can_rotate,
                    ..default()
                });
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
    mut ridp: ResMut<RollbackIdProvider>,
    element_assets: ResMut<Assets<MapElementMeta>>,
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
        let BuiltinElementKind::Grenades { grab_offset, atlas_handle, throw_velocity, .. } = &meta.builtin else {
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
                    .insert(Name::new("Grenade ( Lit )"))
                    .insert(Transform::from_translation(
                        player_transform.translation
                            + (*grab_offset * horizontal_flip_factor).extend(0.0),
                    ))
                    .insert(GlobalTransform::default())
                    .insert(Visibility::default())
                    .insert(ComputedVisibility::default())
                    .insert(AnimatedSprite {
                        start: 3,
                        end: 5,
                        repeat: true,
                        fps: 8.0,
                        atlas: atlas_handle.inner.clone(),
                        ..default()
                    })
                    .insert(meta_handle.clone())
                    .insert(body.clone())
                    .insert(LitGrenade {
                        spawner: grenade.spawner,
                        ..default()
                    })
                    .insert(KinematicBody {
                        velocity: *throw_velocity * horizontal_flip_factor + player_body.velocity,
                        is_deactivated: false,
                        ..body.clone()
                    });
            }
        }

        // If the item is dropped
        if let Some(dropped) = dropped {
            commands.entity(item_ent).remove::<ItemDropped>();
            let (.., player_transform, player_body) =
                players.get(dropped.player).expect("Parent is not a player");

            // Re-activate physics
            body.is_deactivated = false;

            // Put sword in rest position
            sprite.start = 0;
            sprite.end = 0;
            body.velocity = player_body.velocity;
            body.is_spawning = true;

            // Drop item at player position
            transform.translation = player_transform.translation + grab_offset.extend(0.0);
        }
    }
}

fn update_lit_grenades(
    mut commands: Commands,
    mut grenades: Query<
        (
            &Rollback,
            Entity,
            &mut LitGrenade,
            &Transform,
            &Handle<MapElementMeta>,
        ),
        Without<PlayerIdx>,
    >,
    mut ridp: ResMut<RollbackIdProvider>,
    element_assets: ResMut<Assets<MapElementMeta>>,
) {
    let mut items = grenades.iter_mut().collect::<Vec<_>>();
    items.sort_by_key(|x| x.0.id());
    for (_, item_ent, mut grenade, transform, meta_handle) in items {
        let meta = element_assets.get(meta_handle).unwrap();
        let BuiltinElementKind::Grenades {
            fuse_time,
            damage_region_size,
            damage_region_lifetime,
            explosion_atlas_handle,
            explosion_lifetime,
            explosion_fps,
            explosion_frames,
            ..
        } = &meta.builtin else {
            unreachable!();
        };

        grenade.age += 1.0 / crate::FPS as f32;

        if grenade.age >= *fuse_time {
            // Despawn the grenade
            commands.entity(item_ent).despawn();
            // Cause the item to re-spawn by re-triggering spawner hydration
            commands
                .entity(grenade.spawner)
                .remove::<MapElementHydrated>();

            // Spawn the damage region entity
            commands
                .spawn()
                .insert(Rollback::new(ridp.next_id()))
                .insert(*transform)
                .insert(GlobalTransform::default())
                .insert(Visibility::default())
                .insert(ComputedVisibility::default())
                .insert(DamageRegion {
                    size: *damage_region_size,
                })
                .insert(Lifetime::new(*damage_region_lifetime));
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
                    end: *explosion_frames,
                    atlas: explosion_atlas_handle.inner.clone(),
                    repeat: false,
                    fps: *explosion_fps,
                    ..default()
                })
                .insert(Lifetime::new(*explosion_lifetime));
        }
    }
}
