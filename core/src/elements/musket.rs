use crate::{
    physics::collisions::{Actor, Collider, TileCollision},
    prelude::*,
};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, bullet_update)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, TypeUlid, Default)]
#[ulid = "01GQWRRV9HV52X9JAYYF1AFFS7"]
pub struct Musket;

#[derive(Clone, Debug, TypeUlid, Default)]
#[ulid = "01GQX3KM2A4WPV2NKJNG85TJ3P"]
pub struct MusketBullet {
    size: Vec2,
    velocity: Vec2,
    direction: f32,
    explosion_fps: f32,
    explosion_volume: f32,
    explosion_lifetime: f32,
    explosion_frames: usize,
    explosion_atlas: Handle<Atlas>,
    explosion_sound: Handle<AudioSource>,
}

fn hydrate(
    game_meta: Res<CoreMetaArc>,
    mut entities: ResMut<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut muskets: CompMut<Musket>,
    mut atlas_sprites: CompMut<AtlasSprite>,
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

        if let BuiltinElementKind::Musket {
            atlas,
            body_size,
            body_offset,
            can_rotate,
            bounciness,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            muskets.insert(entity, Musket::default());
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
            respawn_points.insert(entity, MapRespawnPoint(transform.translation));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle.clone());
            hydrated.insert(entity, MapElementHydrated);
            bodies.insert(
                entity,
                KinematicBody {
                    size: *body_size,
                    offset: *body_offset,
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

fn update(
    entities: Res<Entities>,
    mut commands: Commands,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,

    mut muskets: CompMut<Musket>,
    mut sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut audio_events: ResMut<AudioEvents>,

    player_inventories: PlayerInventories,
    mut items_used: CompMut<ItemUsed>,
    mut attachments: CompMut<Attachment>,
    mut items_dropped: CompMut<ItemDropped>,
) {
    for (entity, (_musket, element_handle)) in entities.iter_with((&mut muskets, &element_handles))
    {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::Musket {
            shoot_fps,
            shoot_atlas,
            shoot_frames,
            shoot_lifetime,

            bullet_atlas,
            bullet_velocity,
            bullet_body_size,
            bullet_body_offset,

            explosion_fps,
            explosion_volume,
            explosion_sound,
            explosion_atlas,
            explosion_frames,
            explosion_lifetime,

            grab_offset,
            shoot_sound,
            throw_velocity,
            angular_velocity,
            shoot_sound_volume,
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
            let body = bodies.get_mut(entity).unwrap();

            // Deactivate collisions while being held
            body.is_deactivated = true;

            // Attach to the player
            attachments.insert(
                entity,
                Attachment {
                    entity: player,
                    offset: grab_offset.extend(0.1),
                },
            );

            // If the item is being used
            let item_used = items_used.get(entity).is_some();
            if item_used {
                items_used.remove(entity);
                audio_events.play(shoot_sound.clone(), *shoot_sound_volume);

                let player_sprite = sprites.get_mut(player).unwrap();
                let player_flip_x = player_sprite.flip_x;

                let mut shoot_animation_transform = *transforms.get(entity).unwrap();
                shoot_animation_transform.translation.z += 1.0;
                shoot_animation_transform.translation.x +=
                    if player_sprite.flip_x { -15.0 } else { 15.0 };

                let shoot_fps = *shoot_fps;
                let shoot_frames = *shoot_frames;
                let shoot_lifetime = *shoot_lifetime;
                let shoot_atlas = shoot_atlas.clone();

                let bullet_velocity = *bullet_velocity;
                let bullet_atlas = bullet_atlas.clone();
                let bullet_body_size = *bullet_body_size;
                let bullet_body_offset = *bullet_body_offset;

                let explosion_fps = *explosion_fps;
                let explosion_volume = *explosion_volume;
                let explosion_frames = *explosion_frames;
                let explosion_lifetime = *explosion_lifetime;
                let explosion_atlas = explosion_atlas.clone();
                let explosion_sound = explosion_sound.clone();

                commands.add(
                    move |mut actors: CompMut<Actor>,
                          mut colliders: CompMut<Collider>,
                          mut entities: ResMut<Entities>,
                          mut lifetimes: CompMut<Lifetime>,
                          mut sprites: CompMut<AtlasSprite>,
                          mut bullets: CompMut<MusketBullet>,
                          mut transforms: CompMut<Transform>,
                          mut animated_sprites: CompMut<AnimatedSprite>| {
                        // spawn fire animation
                        {
                            let ent = entities.create();
                            transforms.insert(ent, shoot_animation_transform);
                            sprites.insert(
                                ent,
                                AtlasSprite {
                                    flip_x: player_flip_x,
                                    atlas: shoot_atlas.clone(),
                                    ..default()
                                },
                            );

                            animated_sprites.insert(
                                ent,
                                AnimatedSprite {
                                    start: 0,
                                    end: shoot_frames,
                                    fps: shoot_fps,
                                    repeat: false,
                                    ..default()
                                },
                            );
                            lifetimes.insert(ent, Lifetime::new(shoot_lifetime));
                        }

                        // spawn bullet
                        {
                            let ent = entities.create();
                            bullets.insert(
                                ent,
                                MusketBullet {
                                    explosion_fps,
                                    explosion_volume,
                                    explosion_frames,
                                    explosion_lifetime,
                                    size: bullet_body_size,
                                    velocity: bullet_velocity,
                                    direction: if player_flip_x { -1.0 } else { 1.0 },
                                    explosion_atlas: explosion_atlas.clone(),
                                    explosion_sound: explosion_sound.clone(),
                                },
                            );
                            transforms.insert(ent, shoot_animation_transform);
                            sprites.insert(
                                ent,
                                AtlasSprite {
                                    atlas: bullet_atlas.clone(),
                                    ..default()
                                },
                            );

                            // Setup custom collider
                            colliders.insert(
                                ent,
                                Collider {
                                    pos: shoot_animation_transform.translation.truncate()
                                        + bullet_body_offset,
                                    width: bullet_body_size.x,
                                    height: bullet_body_size.y,
                                    ..default()
                                },
                            );
                            actors.insert(ent, Actor);

                            lifetimes.insert(ent, Lifetime::new(1.0));
                        }
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

            if player_velocity != Vec2::ZERO {
                let horizontal_flip_factor = if player_sprite.flip_x {
                    Vec2::new(-1.0, 1.0)
                } else {
                    Vec2::ONE
                };

                body.velocity = *throw_velocity * horizontal_flip_factor + player_velocity;
                body.angular_velocity =
                    *angular_velocity * if player_sprite.flip_x { -1.0 } else { 1.0 };
            }

            body.is_spawning = true;

            let transform = transforms.get_mut(entity).unwrap();
            transform.translation = player_translation;
        }
    }
}

fn bullet_update(
    entities: Res<Entities>,
    mut commands: Commands,

    mut collision_world: CollisionWorld,
    player_indexes: Comp<PlayerIdx>,
    mut player_events: ResMut<PlayerEvents>,

    mut bullets: CompMut<MusketBullet>,
    mut transforms: CompMut<Transform>,
    mut audio_events: ResMut<AudioEvents>,
) {
    for (entity, bullet) in entities.iter_with(&mut bullets) {
        let MusketBullet {
            size,
            velocity,
            explosion_fps,
            explosion_volume,
            explosion_sound,
            explosion_atlas,
            explosion_frames,
            explosion_lifetime,
            ..
        } = &bullet;

        // Move bullet
        let position = transforms.get_mut(entity).unwrap();
        position.translation += bullet.direction * velocity.extend(0.0);
        collision_world.set_actor_position(entity, position.translation.truncate());

        // Check actor collisions
        let mut hit_player = false;
        collision_world
            .actor_collisions(entity)
            .into_iter()
            .filter(|&x| player_indexes.contains(x))
            .for_each(|player| {
                hit_player = true;
                player_events.kill(player);
            });

        // check solid tile collisions
        let hit_solid =
            collision_world.collide_solids(position.translation.truncate(), size.x, size.y)
                != TileCollision::EMPTY;

        // Bullet hit something
        if hit_player || hit_solid {
            audio_events.play(explosion_sound.clone(), *explosion_volume);

            let mut explosion_transform = *transforms.get(entity).unwrap();
            explosion_transform.translation.z += 1.0;

            let explosion_fps = *explosion_fps;
            let explosion_frames = *explosion_frames;
            let explosion_lifetime = *explosion_lifetime;
            let explosion_atlas = explosion_atlas.clone();

            commands.add(
                move |mut entities: ResMut<Entities>,
                      mut transforms: CompMut<Transform>,
                      mut lifetimes: CompMut<Lifetime>,
                      mut sprites: CompMut<AtlasSprite>,
                      mut animated_sprites: CompMut<AnimatedSprite>| {
                    // Despawn the bullet
                    entities.kill(entity);

                    // spawn bullet explosion animation
                    {
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
                                start: 0,
                                end: explosion_frames,
                                fps: explosion_fps,
                                repeat: false,
                                ..default()
                            },
                        );
                        lifetimes.insert(ent, Lifetime::new(explosion_lifetime));
                    }
                },
            );
        }
    }
}
