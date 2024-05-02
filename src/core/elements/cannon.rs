use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("cannon"))]
#[repr(C)]
pub struct CannonMeta {
    pub grab_offset: Vec2,
    pub fin_anim: Ustr,

    pub body_size: Vec2,
    pub bounciness: f32,
    pub can_rotate: bool,
    pub throw_velocity: f32,
    pub angular_velocity: f32,
    pub atlas: Handle<Atlas>,
    pub animation_fps: u32,

    pub max_ammo: u32,
    pub bomb_meta: Handle<KickBombMeta>,
    pub cooldown: Duration,
    pub bullet_spawn_offset: Vec2,
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
    CannonMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update);
}

#[derive(Clone, Debug, HasSchema, Default)]
pub struct Cannon {
    pub ammo: u32,
    pub cooldown: Timer,
    pub animation_cooldown: Timer,
}

fn hydrate(
    game_meta: Root<GameMeta>,
    mut entities: ResMutInit<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    assets: Res<AssetServer>,
    mut cannons: CompMut<Cannon>,
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

        if let Ok(CannonMeta {
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
                    .with_system(cannon_drop(entity, *max_ammo)),
            );
            item_grabs.insert(
                entity,
                ItemGrab {
                    fin_anim: *fin_anim,
                    sync_animation: false,
                    grab_offset: *grab_offset,
                },
            );
            cannons.insert(
                entity,
                Cannon {
                    ammo: *max_ammo,
                    cooldown: Timer::new(Duration::from_millis(0), TimerMode::Once),
                    animation_cooldown: Timer::new(Duration::from_millis(0), TimerMode::Once),
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

    mut cannons: CompMut<Cannon>,
    transforms: CompMut<Transform>,
    mut sprites: CompMut<AtlasSprite>,
    mut audio_center: ResMut<AudioCenter>,

    player_inventories: PlayerInventories,
    mut items_used: CompMut<ItemUsed>,
    items_dropped: CompMut<ItemDropped>,
    time: Res<Time>,

    mut bodies: CompMut<KinematicBody>,
) {
    for (entity, (cannon, element_handle)) in entities.iter_with((&mut cannons, &element_handles)) {
        let element_meta = assets.get(element_handle.0);

        let asset = assets.get(element_meta.data);
        let Ok(CannonMeta {
            animation_fps,
            max_ammo,
            shoot_fps,
            shoot_atlas,
            shoot_frames,
            shoot_lifetime,
            cooldown,
            bullet_spawn_offset,
            bomb_meta,
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

        cannon.cooldown.tick(time.delta());
        cannon.animation_cooldown.tick(time.delta());

        let sprite = sprites.get_mut(entity).unwrap();
        if sprite.index != 0 && cannon.animation_cooldown.finished() {
            // Reset animation cooldown & turn FPS into MS
            cannon.animation_cooldown = Timer::new(
                Duration::from_millis((1000 / animation_fps).into()),
                TimerMode::Once,
            );
            // Update sprite
            match sprite.index {
                1 => sprite.index = 2,
                2 => sprite.index = 3,
                3 => sprite.index = 0,
                _ => (),
            };
        }

        // If the item is being held
        if let Some(Inv { player, .. }) = player_inventories.find_item(entity) {
            // If the item is being used
            let item_used = items_used.remove(entity).is_some();
            if item_used && cannon.cooldown.finished() {
                // Reset fire cooldown
                cannon.cooldown = Timer::new(*cooldown, TimerMode::Once);
                // Empty
                if cannon.ammo.eq(&0) {
                    audio_center.play_sound(*empty_shoot_sound, *empty_shoot_sound_volume);
                    continue;
                }

                // Update sprite
                sprite.index = 1;
                // Reset animation cooldown & turn FPS into MS
                cannon.animation_cooldown = Timer::new(
                    Duration::from_millis((1000 / animation_fps).into()),
                    TimerMode::Once,
                );

                // Subtract ammo
                cannon.ammo = cannon.ammo.saturating_sub(1).clamp(0, cannon.ammo);
                audio_center.play_sound(*shoot_sound, *shoot_sound_volume);

                let player_sprite = sprites.get_mut(player).unwrap();
                let player_flip_x = player_sprite.flip_x;
                let player_body = bodies.get_mut(player).unwrap();

                //Set kickback
                player_body.velocity.x = if player_flip_x { 1.0 } else { -1.0 } * kickback;

                let mut shoot_animation_transform = *transforms.get(entity).unwrap();
                let bullet_spawn_offset = *bullet_spawn_offset;
                shoot_animation_transform.translation.z += 1.0;
                shoot_animation_transform.translation.y += bullet_spawn_offset.y;
                shoot_animation_transform.translation.x += if player_sprite.flip_x {
                    -bullet_spawn_offset.x
                } else {
                    bullet_spawn_offset.x
                };

                let shoot_fps = *shoot_fps;
                let shoot_frames = *shoot_frames;
                let shoot_lifetime = *shoot_lifetime;
                let shoot_atlas = *shoot_atlas;

                let bomb_meta = *bomb_meta;

                commands.add(
                    move |mut entities: ResMutInit<Entities>,
                          mut lifetimes: CompMut<Lifetime>,
                          mut sprites: CompMut<AtlasSprite>,
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
                    },
                );
                //Spawn bomb
                commands.add(KickBombCommand::spawn_kick_bomb(
                    None,
                    shoot_animation_transform,
                    bomb_meta.untyped(),
                    true,
                    Some(player_flip_x),
                ));
            }
        }

        // If the item was dropped
        if items_dropped.get(entity).is_some() {
            // reload gun
            cannon.ammo = *max_ammo;
        }
    }
}

fn cannon_drop(entity: Entity, max_ammo: u32) -> StaticSystem<(), ()> {
    (move |mut cannons: CompMut<Cannon>| {
        cannons.get_mut(entity).unwrap().ammo = max_ammo;
    })
    .system()
}
