use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, TypeUlid, Default)]
#[ulid = "01GQWRRV9HV52X9JAYYF1AFFS7"]
pub struct Musket {
    pub ammo: usize,
    pub cooldown_frame: usize,
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
            max_ammo,
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
            muskets.insert(
                entity,
                Musket {
                    ammo: *max_ammo,
                    cooldown_frame: 0,
                },
            );
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
    mut player_layers: CompMut<PlayerLayers>,

    player_inventories: PlayerInventories,
    mut items_used: CompMut<ItemUsed>,
    mut attachments: CompMut<PlayerBodyAttachment>,
    mut items_dropped: CompMut<ItemDropped>,
) {
    for (entity, (musket, element_handle)) in entities.iter_with((&mut muskets, &element_handles)) {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::Musket {
            max_ammo,
            shoot_fps,
            shoot_atlas,
            shoot_frames,
            shoot_lifetime,


            fin_anim,
            grab_offset,
            throw_velocity,
            angular_velocity,

            bullet_meta,
            shoot_sound,
            cooldown_frames,
            empty_shoot_sound,
            shoot_sound_volume,
            empty_shoot_sound_volume,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        musket.cooldown_frame += 1;

        // If the item is being held
        if let Some(inventory) = player_inventories
            .iter()
            .find_map(|x| x.filter(|x| x.inventory == entity))
        {
            let player = inventory.player;
            let body = bodies.get_mut(entity).unwrap();
            player_layers.get_mut(player).unwrap().fin_anim = *fin_anim;

            // Deactivate collisions while being held
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
            let can_fire = musket.cooldown_frame >= *cooldown_frames;
            if item_used {
                items_used.remove(entity);
            }
            if item_used && can_fire {
                // Empty
                if musket.ammo.eq(&0) {
                    audio_events.play(empty_shoot_sound.clone(), *empty_shoot_sound_volume);
                    continue;
                }

                // Reset fire cooldown and subtract ammo
                musket.cooldown_frame = 0;
                musket.ammo = musket.ammo.saturating_sub(1).clamp(0, musket.ammo);
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

                let bullet_meta = bullet_meta.clone();

                commands.add(
                    move |mut entities: ResMut<Entities>,
                          mut lifetimes: CompMut<Lifetime>,
                          mut sprites: CompMut<AtlasSprite>,
                          mut transforms: CompMut<Transform>,
                          mut bullets: CompMut<Bullet>,
                          mut bullet_handles: CompMut<BulletHandle>,
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
                                    frames: (0..shoot_frames).collect(),
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
                                Bullet {
                                    direction: if player_flip_x { -1.0 } else { 1.0 },
                                },
                            );
                            transforms.insert(ent, shoot_animation_transform);
                            bullet_handles.insert(ent, BulletHandle(bullet_meta.clone()));
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

            // reload gun
            musket.ammo = *max_ammo;

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

            if player_velocity != Vec2::ZERO {
                body.velocity = *throw_velocity * horizontal_flip_factor + player_velocity;
                body.angular_velocity =
                    *angular_velocity * if player_sprite.flip_x { -1.0 } else { 1.0 };
            }

            body.is_spawning = true;

            let transform = transforms.get_mut(entity).unwrap();
            transform.translation =
                player_translation + (*grab_offset * horizontal_flip_factor).extend(0.0);
        }
    }
}
