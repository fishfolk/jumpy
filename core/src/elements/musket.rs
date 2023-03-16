use std::time::Duration;

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
    pub cooldown: Timer,
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
    mut item_throws: CompMut<ItemThrow>,
    mut item_grabs: CompMut<ItemGrab>,
    mut respawn_points: CompMut<DehydrateOutOfBounds>,
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
            fin_anim,
            grab_offset,
            max_ammo,
            body_size,
            can_rotate,
            bounciness,
            throw_velocity,
            angular_velocity,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            item_throws.insert(
                entity,
                ItemThrow::strength(*throw_velocity).with_spin(*angular_velocity),
            );
            item_grabs.insert(
                entity,
                ItemGrab {
                    fin_anim: *fin_anim,
                    sync_animation: false,
                    grab_offset: *grab_offset,
                },
            );
            muskets.insert(
                entity,
                Musket {
                    ammo: *max_ammo,
                    cooldown: Timer::new(Duration::from_millis(0), TimerMode::Once),
                },
            );
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
            respawn_points.insert(entity, DehydrateOutOfBounds(spawner_ent));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle.clone());
            hydrated.insert(entity, MapElementHydrated);
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
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
    transforms: CompMut<Transform>,
    mut sprites: CompMut<AtlasSprite>,
    mut audio_events: ResMut<AudioEvents>,

    player_inventories: PlayerInventories,
    mut items_used: CompMut<ItemUsed>,
    items_dropped: CompMut<ItemDropped>,
    time: Res<Time>,
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
            cooldown,
            bullet_meta,
            shoot_sound,
            empty_shoot_sound,
            shoot_sound_volume,
            empty_shoot_sound_volume,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        musket.cooldown.tick(time.delta());

        // If the item is being held
        if let Some(inventory) = player_inventories
            .iter()
            .find_map(|x| x.filter(|x| x.inventory == entity))
        {
            let player = inventory.player;

            // If the item is being used
            let item_used = items_used.get(entity).is_some();
            if item_used {
                items_used.remove(entity);
            }
            if item_used && musket.cooldown.finished() {
                // Empty
                if musket.ammo.eq(&0) {
                    audio_events.play(empty_shoot_sound.clone(), *empty_shoot_sound_volume);
                    continue;
                }

                // Reset fire cooldown and subtract ammo
                musket.cooldown = Timer::new(*cooldown, TimerMode::Once);
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
                                    owner: player,
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
        if items_dropped.get(entity).is_some() {
            // reload gun
            musket.ammo = *max_ammo;
        }
    }
}
