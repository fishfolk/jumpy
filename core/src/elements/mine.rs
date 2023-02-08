use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_thrown_mines)
        .add_system_to_stage(CoreStage::PostUpdate, update_idle_mines);
}

#[derive(Clone, TypeUlid, Debug, Copy)]
#[ulid = "01GPRSBWQ3X0QJC37BDDQXDNF3"]
pub struct IdleMine {
    /// The entity ID of the map element that spawned the mine
    pub spawner: Entity,
}

#[derive(Clone, TypeUlid, Debug, Copy)]
#[ulid = "01GPRSBWQ3X0QJC37BDQXDNASF"]
pub struct ThrownMine {
    // The entity ID of the map element that spawned the mine.
    spawner: Entity,
    // How long the mine has been thrown.
    age: f32,
}

fn hydrate(
    game_meta: Res<CoreMetaArc>,
    mut entities: ResMut<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut idle_mines: CompMut<IdleMine>,
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

        if let BuiltinElementKind::Mine {
            atlas,
            body_size,
            bounciness,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            idle_mines.insert(
                entity,
                IdleMine {
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
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: true,
                    has_friction: true,
                    bounciness: *bounciness,
                    gravity: game_meta.physics.gravity,
                    ..default()
                },
            );
        }
    }
}

fn update_idle_mines(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut transforms: CompMut<Transform>,
    mut idle_mines: CompMut<IdleMine>,
    mut sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut items_used: CompMut<ItemUsed>,
    mut items_dropped: CompMut<ItemDropped>,
    player_inventories: PlayerInventories,
    mut attachments: CompMut<PlayerBodyAttachment>,
    mut player_layers: CompMut<PlayerLayers>,
    mut commands: Commands,
    mut player_events: ResMut<PlayerEvents>,
) {
    for (entity, (mine, element_handle)) in entities.iter_with((&mut idle_mines, &element_handles))
    {
        let spawner = mine.spawner;
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
        continue;
    };

        let BuiltinElementKind::Mine {
        grab_offset,
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
            if items_used.get(entity).is_some() {
                items_used.remove(entity);
                player_events.set_inventory(player, None);
                attachments.remove(entity);

                let player_velocity = bodies.get(player).unwrap().velocity;
                let player_sprite = sprites.get_mut(player).unwrap();
                let player_translation = transforms.get(player).unwrap().translation;

                let body = bodies.get_mut(entity).unwrap();

                let horizontal_flip_factor = if player_sprite.flip_x {
                    Vec2::new(-1.0, 1.0)
                } else {
                    Vec2::ONE
                };

                body.velocity = *throw_velocity * horizontal_flip_factor + player_velocity;
                body.is_deactivated = false;

                let transform = transforms.get_mut(entity).unwrap();
                transform.translation =
                    player_translation + (*grab_offset * horizontal_flip_factor).extend(0.0);

                commands.add(
                    move |mut idle: CompMut<IdleMine>, mut thrown: CompMut<ThrownMine>| {
                        idle.remove(entity);
                        thrown.insert(entity, ThrownMine { spawner, age: 0.0 });
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
            let player_sprite = sprites.get_mut(player).unwrap();

            let body = bodies.get_mut(entity).unwrap();

            // Re-activate physics
            body.is_deactivated = false;

            let horizontal_flip_factor = if player_sprite.flip_x {
                Vec2::new(-1.0, 1.0)
            } else {
                Vec2::ONE
            };

            if player_velocity != Vec2::ZERO {
                body.velocity = *throw_velocity * horizontal_flip_factor + player_velocity;
            }

            body.is_spawning = true;

            let transform = transforms.get_mut(entity).unwrap();
            transform.translation =
                player_translation + (*grab_offset * horizontal_flip_factor).extend(0.0);
        }
    }
}

fn update_thrown_mines(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut audio_events: ResMut<AudioEvents>,
    mut thrown_mines: CompMut<ThrownMine>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut hydrated: CompMut<MapElementHydrated>,
    player_indexes: Comp<PlayerIdx>,
    mut player_events: ResMut<PlayerEvents>,
    mut commands: Commands,
    collision_world: CollisionWorld,
) {
    let players = entities
        .iter_with(&player_indexes)
        .map(|x| x.0)
        .collect::<Vec<_>>();
    for (entity, (thrown_mine, element_handle, sprite)) in
        entities.iter_with((&mut thrown_mines, &element_handles, &mut animated_sprites))
    {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::Mine {
            explosion_fps,
            explosion_frames,
            explosion_sound,
            explosion_atlas,
            arm_delay,
            arm_sound,
            armed_frames,
            armed_fps,
            damage_region_size,
            damage_region_lifetime, explosion_volume, arm_sound_volume, explosion_lifetime, .. } = &element_meta.builtin else {
            unreachable!();
        };

        let frame_time = 1.0 / crate::FPS;
        thrown_mine.age += 1.0 / crate::FPS;

        if thrown_mine.age >= *arm_delay && thrown_mine.age - *arm_delay < frame_time {
            audio_events.play(arm_sound.clone(), *arm_sound_volume);

            sprite.frames = (0..*armed_frames).collect();
            sprite.fps = *armed_fps;
            sprite.repeat = true;
        }

        let colliding_with_players = collision_world
            .actor_collisions(entity)
            .into_iter()
            .filter(|&x| players.contains(&x))
            .collect::<Vec<_>>();

        if !colliding_with_players.is_empty() && thrown_mine.age >= *arm_delay {
            for player in &colliding_with_players {
                player_events.kill(*player);
            }

            audio_events.play(explosion_sound.clone(), *explosion_volume);

            hydrated.remove(thrown_mine.spawner);

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
                    let explosion_transform = *transforms.get(entity).unwrap();

                    // Despawn the grenade
                    entities.kill(entity);

                    // Spawn the damage region
                    let damage_ent = entities.create();
                    transforms.insert(damage_ent, explosion_transform);
                    damage_regions.insert(
                        damage_ent,
                        DamageRegion {
                            size: damage_region_size,
                        },
                    );
                    lifetimes.insert(damage_ent, Lifetime::new(damage_region_lifetime));

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
