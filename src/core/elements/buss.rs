use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("buss"))]
#[repr(C)]
pub struct BussMeta {
    pub grab_offset: Vec2,
    pub fin_anim: Ustr,

    pub body_size: Vec2,
    pub bounciness: f32,
    pub can_rotate: bool,
    pub throw_velocity: f32,
    pub angular_velocity: f32,
    pub atlas: Handle<Atlas>,

    pub max_ammo: u32,
    pub cooldown: Duration,
    pub bullet_count: u32,
    pub bullet_spread: f32,
    pub bullet_meta: Handle<BulletMeta>,
    pub kickback: f32,

    pub shoot_fps: f32,
    pub shoot_lifetime: f32,
    pub shoot_frames: u32,
    pub shoot_sound_volume: f64,
    pub empty_shoot_sound_volume: f64,
    pub shoot_atlas: Handle<Atlas>,
    pub shoot_sound: Handle<AudioSource>,
    pub empty_shoot_sound: Handle<AudioSource>,
}

pub fn game_plugin(game: &mut Game) {
    BussMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, HasSchema, Default)]
pub struct Buss {
    pub ammo: u32,
    pub cooldown: Timer,
}

fn hydrate(
    game_meta: Root<GameMeta>,
    mut entities: ResMutInit<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    assets: Res<AssetServer>,
    mut busses: CompMut<Buss>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut items: CompMut<Item>,
    mut item_throws: CompMut<ItemThrow>,
    mut item_grabs: CompMut<ItemGrab>,
    mut respawn_points: CompMut<DehydrateOutOfBounds>,
    mut spawner_manager: SpawnerManager,
) {
    let mut not_hydrated_bitset = hydrated.bitset().clone();
    not_hydrated_bitset.bit_not();
    not_hydrated_bitset.bit_and(element_handles.bitset());

    let spawner_entities = entities
        .iter_with_bitset(&not_hydrated_bitset)
        .collect::<Vec<_>>();

    for spawner_ent in spawner_entities {
        let transform = *transforms.get(spawner_ent).unwrap();
        let element_handle = *element_handles.get(spawner_ent).unwrap();
        let element_meta = assets.get(element_handle.0);

        if let Ok(BussMeta {
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
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            item_throws.insert(
                entity,
                ItemThrow::strength(*throw_velocity)
                    .with_spin(*angular_velocity)
                    .with_system(buss_drop(entity, *max_ammo)),
            );
            item_grabs.insert(
                entity,
                ItemGrab {
                    fin_anim: *fin_anim,
                    sync_animation: false,
                    grab_offset: *grab_offset,
                },
            );
            busses.insert(
                entity,
                Buss {
                    ammo: *max_ammo,
                    cooldown: Timer::new(Duration::from_millis(0), TimerMode::Once),
                },
            );
            atlas_sprites.insert(entity, AtlasSprite::new(*atlas));
            respawn_points.insert(entity, DehydrateOutOfBounds(spawner_ent));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle);
            hydrated.insert(entity, MapElementHydrated);
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: true,
                    has_friction: true,
                    can_rotate: *can_rotate,
                    bounciness: *bounciness,
                    gravity: game_meta.core.physics.gravity,
                    ..default()
                },
            );
            spawner_manager.create_spawner(spawner_ent, vec![entity])
        }
    }
}

fn update(
    entities: Res<Entities>,
    mut commands: Commands,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,

    mut busses: CompMut<Buss>,
    transforms: CompMut<Transform>,
    mut sprites: CompMut<AtlasSprite>,
    mut audio_center: ResMut<AudioCenter>,

    player_inventories: PlayerInventories,
    mut items_used: CompMut<ItemUsed>,
    items_dropped: CompMut<ItemDropped>,
    time: Res<Time>,

    mut bodies: CompMut<KinematicBody>,
) {
    for (entity, (buss, element_handle)) in entities.iter_with((&mut busses, &element_handles)) {
        let element_meta = assets.get(element_handle.0);

        let asset = assets.get(element_meta.data);
        let Ok(BussMeta {
            max_ammo,
            shoot_fps,
            shoot_atlas,
            shoot_frames,
            shoot_lifetime,
            cooldown,
            bullet_meta,
            bullet_count,
            bullet_spread,
            shoot_sound,
            empty_shoot_sound,
            shoot_sound_volume,
            empty_shoot_sound_volume,
            kickback,
            ..
        }) = asset.try_cast_ref()
        else {
            unreachable!();
        };

        buss.cooldown.tick(time.delta());

        // If the item is being held
        if let Some(Inv { player, .. }) = player_inventories.find_item(entity) {
            // If the item is being used
            let item_used = items_used.remove(entity).is_some();
            if item_used && buss.cooldown.finished() {
                // Reset fire cooldown
                buss.cooldown = Timer::new(*cooldown, TimerMode::Once);
                // Empty
                if buss.ammo.eq(&0) {
                    audio_center.play_sound(*empty_shoot_sound, *empty_shoot_sound_volume);
                    continue;
                }

                // Subtract ammo
                buss.ammo = buss.ammo.saturating_sub(1).clamp(0, buss.ammo);
                audio_center.play_sound(*shoot_sound, *shoot_sound_volume);

                let player_sprite = sprites.get_mut(player).unwrap();
                let player_flip_x = player_sprite.flip_x;
                let player_body = bodies.get_mut(player).unwrap();

                //Set kickback
                player_body.velocity.x = if player_flip_x { 1.0 } else { -1.0 } * kickback;

                let mut shoot_animation_transform = *transforms.get(entity).unwrap();
                shoot_animation_transform.translation.z += 1.0;
                shoot_animation_transform.translation.y += 6.0;
                shoot_animation_transform.translation.x +=
                    if player_sprite.flip_x { -28.0 } else { 28.0 };

                let shoot_fps = *shoot_fps;
                let shoot_frames = *shoot_frames;
                let shoot_lifetime = *shoot_lifetime;
                let shoot_atlas = *shoot_atlas;

                let bullet_meta = *bullet_meta;
                let bullet_spread = *bullet_spread;
                let bullet_count = *bullet_count;

                commands.add(
                    move |rng: Res<GlobalRng>,
                          mut entities: ResMutInit<Entities>,
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
                                    atlas: shoot_atlas,
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
                            for _bullet in 0..bullet_count {
                                let ent = entities.create();
                                bullets.insert(
                                    ent,
                                    Bullet {
                                        owner: player,
                                        direction: if player_flip_x {
                                            vec2(-1.0, (rng.f32() - 0.5) * bullet_spread)
                                        } else {
                                            vec2(1.0, (rng.f32() - 0.5) * bullet_spread)
                                        },
                                    },
                                );
                                transforms.insert(ent, shoot_animation_transform);
                                bullet_handles.insert(ent, BulletHandle(bullet_meta));
                            }
                        }
                    },
                );
            }
        }

        // If the item was dropped
        if items_dropped.get(entity).is_some() {
            // reload gun
            buss.ammo = *max_ammo;
        }
    }
}

fn buss_drop(entity: Entity, max_ammo: u32) -> StaticSystem<(), ()> {
    (move |mut busses: CompMut<Buss>| {
        // Reload musket
        busses.get_mut(entity).unwrap().ammo = max_ammo;
    })
    .system()
}
