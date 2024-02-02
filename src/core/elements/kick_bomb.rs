use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("kick_bomb"))]
#[repr(C)]
pub struct KickBombMeta {
    pub body_diameter: f32,
    pub fin_anim: Ustr,
    pub grab_offset: Vec2,
    pub damage_region_size: Vec2,
    pub damage_region_lifetime: f32,
    pub kick_velocity: Vec2,
    pub throw_velocity: f32,
    pub explosion_lifetime: f32,
    pub explosion_frames: u32,
    pub explosion_fps: f32,
    pub explosion_sound: Handle<AudioSource>,
    pub explosion_volume: f64,
    pub fuse_sound: Handle<AudioSource>,
    pub fuse_sound_volume: f64,
    /// The time in seconds before a grenade explodes
    pub fuse_time: Duration,
    pub can_rotate: bool,
    /// The grenade atlas
    pub atlas: Handle<Atlas>,
    pub explosion_atlas: Handle<Atlas>,
    pub bounciness: f32,
    pub angular_velocity: f32,
    pub arm_delay: Duration,
}

pub fn game_plugin(_game: &mut Game) {
    KickBombMeta::register_schema();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_lit_kick_bombs)
        .add_system_to_stage(CoreStage::PostUpdate, update_idle_kick_bombs);
}

#[derive(Clone, HasSchema, Default, Debug, Copy)]
pub struct IdleKickBomb;

#[derive(Clone, HasSchema, Default, Debug)]
pub struct LitKickBomb {
    arm_delay: Timer,
    fuse_time: Timer,
    kicking: bool,
    kicks: u32,
}

fn hydrate(
    game_meta: Root<GameMeta>,
    mut items: CompMut<Item>,
    mut item_throws: CompMut<ItemThrow>,
    mut item_grabs: CompMut<ItemGrab>,
    mut entities: ResMutInit<Entities>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut idle_bombs: CompMut<IdleKickBomb>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    assets: Res<AssetServer>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    mut animated_sprites: CompMut<AnimatedSprite>,
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

        if let Ok(KickBombMeta {
            atlas,
            fin_anim,
            grab_offset,
            body_diameter,
            can_rotate,
            bounciness,
            throw_velocity,
            angular_velocity,
            ..
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            hydrated.insert(entity, MapElementHydrated);
            element_handles.insert(entity, element_handle);
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
            idle_bombs.insert(entity, IdleKickBomb);
            atlas_sprites.insert(entity, AtlasSprite::new(*atlas));
            respawn_points.insert(entity, DehydrateOutOfBounds(spawner_ent));
            transforms.insert(entity, transform);
            animated_sprites.insert(entity, default());
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Circle {
                        diameter: *body_diameter,
                    },
                    gravity: game_meta.core.physics.gravity,
                    has_mass: true,
                    has_friction: true,
                    can_rotate: *can_rotate,
                    bounciness: *bounciness,
                    ..default()
                },
            );
            spawner_manager.create_spawner(spawner_ent, vec![entity])
        }
    }
}

fn update_idle_kick_bombs(
    entities: Res<Entities>,
    mut commands: Commands,
    mut items_used: CompMut<ItemUsed>,
    mut audio_center: ResMut<AudioCenter>,
    element_handles: Comp<ElementHandle>,
    mut idle_bombs: CompMut<IdleKickBomb>,
    assets: Res<AssetServer>,
    mut animated_sprites: CompMut<AnimatedSprite>,
) {
    for (entity, (_kick_bomb, element_handle)) in
        entities.iter_with((&mut idle_bombs, &element_handles))
    {
        let element_meta = assets.get(element_handle.0);

        let asset = assets.get(element_meta.data);
        let Ok(KickBombMeta {
            fuse_sound,
            fuse_sound_volume,
            arm_delay,
            fuse_time,
            ..
        }) = asset.try_cast_ref()
        else {
            unreachable!();
        };

        let arm_delay = *arm_delay;
        let fuse_time = *fuse_time;

        if items_used.remove(entity).is_some() {
            audio_center.play_sound(*fuse_sound, *fuse_sound_volume);
            let animated_sprite = animated_sprites.get_mut(entity).unwrap();
            animated_sprite.frames = [3, 4, 5].into_iter().collect();
            animated_sprite.repeat = true;
            animated_sprite.fps = 8.0;
            commands.add(
                move |mut idle: CompMut<IdleKickBomb>, mut lit: CompMut<LitKickBomb>| {
                    idle.remove(entity);
                    lit.insert(
                        entity,
                        LitKickBomb {
                            arm_delay: Timer::new(arm_delay, TimerMode::Once),
                            fuse_time: Timer::new(fuse_time, TimerMode::Once),
                            kicking: false,
                            kicks: 0,
                        },
                    );
                },
            );
        }
    }
}

fn update_lit_kick_bombs(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,

    collision_world: CollisionWorld,
    player_indexes: Comp<PlayerIdx>,
    mut audio_center: ResMut<AudioCenter>,
    mut trauma_events: ResMutInit<CameraTraumaEvents>,
    mut lit_grenades: CompMut<LitKickBomb>,
    mut sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut hydrated: CompMut<MapElementHydrated>,
    player_inventories: PlayerInventories,
    mut transforms: CompMut<Transform>,
    mut commands: Commands,
    time: Res<Time>,
    spawners: Comp<DehydrateOutOfBounds>,
    invincibles: CompMut<Invincibility>,
) {
    for (entity, (kick_bomb, element_handle, spawner)) in
        entities.iter_with((&mut lit_grenades, &element_handles, &spawners))
    {
        let element_meta = assets.get(element_handle.0);
        let asset = assets.get(element_meta.data);
        let Ok(KickBombMeta {
            explosion_sound,
            explosion_volume,
            kick_velocity,
            damage_region_lifetime,
            damage_region_size,
            explosion_lifetime,
            explosion_atlas,
            explosion_fps,
            explosion_frames,
            ..
        }) = asset.try_cast_ref()
        else {
            unreachable!();
        };

        kick_bomb.fuse_time.tick(time.delta());
        kick_bomb.arm_delay.tick(time.delta());

        let should_explode = 'should_explode: {
            if kick_bomb.fuse_time.finished() {
                break 'should_explode true;
            }

            // If the item is being held
            if player_inventories.find_item(entity).is_some() {
                kick_bomb.kicking = false;
                break 'should_explode false;
            }

            // If the item is colliding with a non-invincible player
            if let Some(player_entity) = collision_world
                .actor_collisions_filtered(entity, |e| !invincibles.contains(e))
                .into_iter()
                .find(|&x| player_indexes.contains(x))
            {
                if !std::mem::replace(&mut kick_bomb.kicking, true) {
                    kick_bomb.kicks += 1;
                }

                // Explode on the 3rd kick.
                // Dropping the bomb is detected as a kick so we explode when
                // the counter reaches 4.
                if kick_bomb.kicks > 3 {
                    break 'should_explode true;
                }

                let body = bodies.get_mut(entity).unwrap();
                let translation = transforms.get_mut(entity).unwrap().translation;

                let player_sprite = sprites.get_mut(player_entity).unwrap();
                let player_translation = transforms.get(player_entity).unwrap().translation;

                let player_standing_left = player_translation.x <= translation.x;

                if body.velocity.x == 0.0 {
                    body.velocity = *kick_velocity;
                    if player_sprite.flip_x {
                        body.velocity.x *= -1.0;
                    }
                } else if player_standing_left && !player_sprite.flip_x {
                    body.velocity.x = kick_velocity.x;
                    body.velocity.y = kick_velocity.y;
                } else if !player_standing_left && player_sprite.flip_x {
                    body.velocity.x = -kick_velocity.x;
                    body.velocity.y = kick_velocity.y;
                } else if kick_bomb.arm_delay.finished() {
                    break 'should_explode true;
                }
            } else {
                kick_bomb.kicking = false;
            }

            false
        };

        // If it's time to explode
        if should_explode {
            audio_center.play_sound(*explosion_sound, *explosion_volume);

            trauma_events.send(7.5);

            // Cause the item to respawn by un-hydrating it's spawner.
            hydrated.remove(**spawner);
            let mut explosion_transform = *transforms.get(entity).unwrap();
            explosion_transform.translation.z = -10.0; // On top of almost everything
            explosion_transform.rotation = Quat::IDENTITY;

            // Clone types for move into closure
            let damage_region_size = *damage_region_size;
            let damage_region_lifetime = *damage_region_lifetime;
            let explosion_lifetime = *explosion_lifetime;
            let explosion_atlas = *explosion_atlas;
            let explosion_fps = *explosion_fps;
            let explosion_frames = *explosion_frames;
            commands.add(
                move |mut entities: ResMutInit<Entities>,
                      mut transforms: CompMut<Transform>,
                      mut damage_regions: CompMut<DamageRegion>,
                      mut lifetimes: CompMut<Lifetime>,
                      mut sprites: CompMut<AtlasSprite>,
                      mut animated_sprites: CompMut<AnimatedSprite>| {
                    // Despawn the kick bomb
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
                            atlas: explosion_atlas,
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
