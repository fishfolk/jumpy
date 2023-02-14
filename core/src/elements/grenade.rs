use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_lit_grenades)
        .add_system_to_stage(CoreStage::PostUpdate, update_idle_grenades);
}

#[derive(Clone, TypeUlid, Debug, Copy)]
#[ulid = "01GPRSBWQ3X0QJC37BDDQXDN84"]
pub struct IdleGrenade {
    /// The entity ID of the map element that spawned the grenade
    pub spawner: Entity,
}

#[derive(Clone, TypeUlid, Debug, Copy)]
#[ulid = "01GPY9N9CBR6EFJX0RS2H2K58J"]
pub struct LitGrenade {
    /// The entity ID of the map element that spawned the grenade.
    pub spawner: Entity,
    /// How long the grenade has been lit.
    pub age: f32,
}

fn hydrate(
    game_meta: Res<CoreMetaArc>,
    mut entities: ResMut<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut idle_grenades: CompMut<IdleGrenade>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut items: CompMut<Item>,
    mut respawn_points: CompMut<MapRespawnPoint>,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let spawners = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for spawner_ent in spawners {
        let transform = *transforms.get(spawner_ent).unwrap();
        let element_handle = element_handles.get(spawner_ent).unwrap();
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        if let BuiltinElementKind::Grenade {
            atlas,
            body_diameter,
            can_rotate,
            bounciness,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            idle_grenades.insert(
                entity,
                IdleGrenade {
                    spawner: spawner_ent,
                },
            );
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
            respawn_points.insert(entity, MapRespawnPoint(transform.translation));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle.clone());
            hydrated.insert(entity, MapElementHydrated);
            animated_sprites.insert(entity, default());
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Circle {
                        diameter: *body_diameter,
                    },
                    has_mass: true,
                    has_friction: true,
                    can_rotate: *can_rotate,
                    bounciness: *bounciness,
                    gravity: game_meta.physics.gravity,
                    ..default()
                },
            );
        }
    }
}

fn update_idle_grenades(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut audio_events: ResMut<AudioEvents>,
    mut transforms: CompMut<Transform>,
    mut idle_grenades: CompMut<IdleGrenade>,
    mut sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut items_used: CompMut<ItemUsed>,
    mut items_dropped: CompMut<ItemDropped>,
    mut attachments: CompMut<PlayerBodyAttachment>,
    mut player_layers: CompMut<PlayerLayers>,
    player_inventories: PlayerInventories,
    mut commands: Commands,
) {
    for (entity, (grenade, element_handle)) in
        entities.iter_with((&mut idle_grenades, &element_handles))
    {
        let spawner = grenade.spawner;
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::Grenade {
            grab_offset,
            angular_velocity,
            fuse_sound,
            fuse_sound_volume,
            throw_velocity,
            fin_anim,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        // If the item is being held
        if let Some(inventory) = player_inventories
            .iter()
            .find_map(|x| x.filter(|x| x.inventory == entity))
        {
            let player = inventory.player;
            let player_layers = player_layers.get_mut(player).unwrap();
            player_layers.fin_anim = *fin_anim;
            let body = bodies.get_mut(entity).unwrap();

            // Deactivate held items
            body.is_deactivated = true;

            // Attach to the player
            attachments.insert(
                entity,
                PlayerBodyAttachment {
                    player,
                    offset: grab_offset.extend(0.1),
                    sync_animation: false,
                },
            );

            // If the item is being used
            let item_used = items_used.get(entity).is_some();
            if item_used {
                audio_events.play(fuse_sound.clone(), *fuse_sound_volume);
                items_used.remove(entity);
                let animated_sprite = animated_sprites.get_mut(entity).unwrap();
                animated_sprite.frames = Arc::from([3, 4, 5]);
                animated_sprite.repeat = true;
                animated_sprite.fps = 8.0;
                body.angular_velocity = *angular_velocity;
                commands.add(
                    move |mut idle: CompMut<IdleGrenade>, mut lit: CompMut<LitGrenade>| {
                        idle.remove(entity);
                        lit.insert(entity, LitGrenade { spawner, age: 0.0 });
                    },
                );
            }
        }

        // If the item was dropped
        if let Some(dropped) = items_dropped.get(entity).copied() {
            let player = dropped.player;

            items_dropped.remove(entity);
            attachments.remove(entity);
            let player_translation = transforms.get(dropped.player).unwrap().translation;
            let player_velocity = bodies.get(player).unwrap().velocity;

            let body = bodies.get_mut(entity).unwrap();
            let player_sprite = sprites.get_mut(player).unwrap();

            // Re-activate physics
            body.is_deactivated = false;

            let horizontal_flip_factor = if player_sprite.flip_x {
                Vec2::new(-1.0, 1.0)
            } else {
                Vec2::ONE
            };
            body.velocity = *throw_velocity * horizontal_flip_factor + player_velocity;
            body.angular_velocity =
                *angular_velocity * if player_sprite.flip_x { -1.0 } else { 1.0 };

            body.is_spawning = true;

            let transform = transforms.get_mut(entity).unwrap();
            transform.translation =
                player_translation + (*grab_offset * horizontal_flip_factor).extend(0.0);
        }
    }
}

fn update_lit_grenades(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut audio_events: ResMut<AudioEvents>,
    mut transforms: CompMut<Transform>,
    mut lit_grenades: CompMut<LitGrenade>,
    mut sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut items_dropped: CompMut<ItemDropped>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut attachments: CompMut<PlayerBodyAttachment>,
    mut emote_regions: CompMut<EmoteRegion>,
    mut player_layers: CompMut<PlayerLayers>,
    player_inventories: PlayerInventories,
    mut commands: Commands,
) {
    for (entity, (grenade, element_handle)) in
        entities.iter_with((&mut lit_grenades, &element_handles))
    {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::Grenade {
            grab_offset,
            angular_velocity,
            explosion_sound,
            explosion_volume,
            throw_velocity,
            fuse_time,
            damage_region_lifetime,
            damage_region_size,
            explosion_lifetime,
            explosion_atlas,
            explosion_fps,
            explosion_frames,
            fin_anim,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        grenade.age += 1.0 / crate::FPS;
        let spawner = grenade.spawner;

        if !emote_regions.contains(entity) {
            emote_regions.insert(
                entity,
                EmoteRegion {
                    direction_sensitive: true,
                    size: *damage_region_size * 2.0,
                    emote: Emote::Alarm,
                    active: true,
                },
            );
        }
        let emote_region = emote_regions.get_mut(entity).unwrap();

        // If the item is being held
        if let Some(inventory) = player_inventories
            .iter()
            .find_map(|x| x.filter(|x| x.inventory == entity))
        {
            let player = inventory.player;
            let layers = player_layers.get_mut(player).unwrap();
            layers.fin_anim = *fin_anim;
            let body = bodies.get_mut(entity).unwrap();

            // Deactivate held items
            body.is_deactivated = true;

            // Attach to the player
            attachments.insert(
                entity,
                PlayerBodyAttachment {
                    player,
                    offset: grab_offset.extend(1.0),
                    sync_animation: false,
                },
            );

            emote_region.active = false;

        // If the item is not being held
        } else {
            emote_region.active = true;
        }

        // If the item was dropped
        if let Some(dropped) = items_dropped.get(entity).copied() {
            let player = dropped.player;

            items_dropped.remove(entity);
            attachments.remove(entity);
            let player_translation = transforms.get(dropped.player).unwrap().translation;
            let player_velocity = bodies.get(player).unwrap().velocity;

            let body = bodies.get_mut(entity).unwrap();
            let player_sprite = sprites.get_mut(player).unwrap();

            // Re-activate physics
            body.is_deactivated = false;

            let horizontal_flip_factor = if player_sprite.flip_x {
                Vec2::new(-1.0, 1.0)
            } else {
                Vec2::ONE
            };
            body.velocity = *throw_velocity * horizontal_flip_factor + player_velocity;
            body.angular_velocity =
                *angular_velocity * if player_sprite.flip_x { -1.0 } else { 1.0 };

            body.is_spawning = true;

            let transform = transforms.get_mut(entity).unwrap();
            transform.translation =
                player_translation + (*grab_offset * horizontal_flip_factor).extend(0.0);
        }

        // If it's time to explode
        if grenade.age >= *fuse_time {
            audio_events.play(explosion_sound.clone(), *explosion_volume);

            // Cause the item to respawn by un-hydrating it's spawner.
            hydrated.remove(spawner);
            let mut explosion_transform = *transforms.get(entity).unwrap();
            explosion_transform.translation.z += 1.0;

            // Clone types for move into closure
            let damage_region_size = *damage_region_size;
            let damage_region_lifetime = *damage_region_lifetime;
            let explosion_lifetime = *explosion_lifetime;
            let explosion_atlas = explosion_atlas.clone();
            let explosion_fps = *explosion_fps;
            let explosion_frames = *explosion_frames;
            commands.add(
                move |mut entities: ResMut<Entities>,
                      mut transforms: CompMut<Transform>,
                      mut damage_regions: CompMut<DamageRegion>,
                      mut lifetimes: CompMut<Lifetime>,
                      mut sprites: CompMut<AtlasSprite>,
                      mut animated_sprites: CompMut<AnimatedSprite>| {
                    // Despawn the grenade
                    entities.kill(entity);

                    // Spawn the damage region
                    let ent = entities.create();
                    transforms.insert(ent, explosion_transform);
                    damage_regions.insert(
                        ent,
                        DamageRegion {
                            size: damage_region_size,
                        },
                    );
                    lifetimes.insert(ent, Lifetime::new(damage_region_lifetime));

                    // Spawn the explosion animation
                    let ent = entities.create();
                    transforms.insert(ent, explosion_transform);
                    sprites.insert(
                        ent,
                        AtlasSprite {
                            atlas: explosion_atlas.clone(),
                            ..default()
                        },
                    );
                    animated_sprites.insert(
                        ent,
                        AnimatedSprite {
                            frames: (0..explosion_frames).collect(),
                            fps: explosion_fps,
                            repeat: false,
                            ..default()
                        },
                    );
                    lifetimes.insert(ent, Lifetime::new(explosion_lifetime));
                },
            );
        }
    }
}
