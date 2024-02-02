use crate::prelude::*;

/// Grenades item
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("grenade"))]
#[repr(C)]
pub struct GrenadeMeta {
    pub body_diameter: f32,
    pub fin_anim: Ustr,
    pub grab_offset: Vec2,
    pub damage_region_size: Vec2,
    pub damage_region_lifetime: f32,
    pub throw_velocity: f32,
    pub explosion_lifetime: f32,
    pub explosion_frames: u32,
    pub explosion_fps: f32,
    pub explosion_sound: Handle<AudioSource>,
    pub explosion_volume: f64,
    pub fuse_sound: Handle<AudioSource>,
    pub fuse_sound_volume: f64,
    /// The time in seconds before a grenade explodes
    pub fuse_time: f32,
    pub can_rotate: bool,
    /// The grenade atlas
    pub atlas: Handle<Atlas>,
    pub explosion_atlas: Handle<Atlas>,
    pub bounciness: f32,
    pub angular_velocity: f32,
}

pub fn game_plugin(game: &mut Game) {
    GrenadeMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_lit_grenades)
        .add_system_to_stage(CoreStage::PostUpdate, update_idle_grenades);
}

#[derive(Clone, HasSchema, Debug, Copy, Default)]
pub struct IdleGrenade;

#[derive(Clone, HasSchema, Debug, Default)]
pub struct LitGrenade {
    /// The owner of the grenade.
    pub owner: Entity,
    /// The amount of time left until the grenade explodes.
    pub fuse_time: Timer,
}

fn hydrate(
    game_meta: Root<GameMeta>,
    mut entities: ResMutInit<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    assets: Res<AssetServer>,
    mut idle_grenades: CompMut<IdleGrenade>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
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

        if let Ok(GrenadeMeta {
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
            items.insert(entity, Item);
            idle_grenades.insert(entity, IdleGrenade);
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
            atlas_sprites.insert(entity, AtlasSprite::new(*atlas));
            respawn_points.insert(entity, DehydrateOutOfBounds(spawner_ent));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle);
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
                    gravity: game_meta.core.physics.gravity,
                    ..default()
                },
            );
            spawner_manager.create_spawner(spawner_ent, vec![entity])
        }
    }
}

fn update_idle_grenades(
    mut commands: Commands,
    entities: Res<Entities>,
    items_used: Comp<ItemUsed>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut audio_center: ResMut<AudioCenter>,
    mut idle_grenades: CompMut<IdleGrenade>,
    mut animated_sprites: CompMut<AnimatedSprite>,
) {
    for (entity, (_grenade, element_handle)) in
        entities.iter_with((&mut idle_grenades, &element_handles))
    {
        let element_meta = assets.get(element_handle.0);

        let asset = assets.get(element_meta.data);
        let Ok(GrenadeMeta {
            fuse_sound,
            fuse_sound_volume,
            fuse_time,
            ..
        }) = asset.try_cast_ref()
        else {
            unreachable!();
        };
        let fuse_time = *fuse_time;

        if items_used.get(entity).is_some() {
            // Animate Grenade
            let animated_sprite = animated_sprites.get_mut(entity).unwrap();
            animated_sprite.frames = [3, 4, 5].into_iter().collect();
            animated_sprite.repeat = true;
            animated_sprite.fps = 8.0;

            // Play that hissss sound
            audio_center.play_sound(*fuse_sound, *fuse_sound_volume);

            commands.add(
                move |mut lit: CompMut<LitGrenade>,
                      mut idle: CompMut<IdleGrenade>,
                      mut items_used: CompMut<ItemUsed>| {
                    idle.remove(entity);

                    lit.insert(
                        entity,
                        LitGrenade {
                            owner: items_used.get(entity).unwrap().owner,
                            fuse_time: Timer::new(
                                Duration::from_secs_f32(fuse_time),
                                TimerMode::Once,
                            ),
                        },
                    );

                    items_used.remove(entity);
                },
            );
        }
    }
}

fn update_lit_grenades(
    time: Res<Time>,
    mut commands: Commands,
    entities: Res<Entities>,
    transforms: CompMut<Transform>,
    element_handles: Comp<ElementHandle>,
    spawners: Comp<DehydrateOutOfBounds>,
    mut audio_center: ResMut<AudioCenter>,
    mut lit_grenades: CompMut<LitGrenade>,
    player_inventories: PlayerInventories,
    assets: Res<AssetServer>,
    mut emote_regions: CompMut<EmoteRegion>,
    mut player_layers: CompMut<PlayerLayers>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut trauma_events: ResMutInit<CameraTraumaEvents>,
) {
    for (entity, (grenade, element_handle, spawner)) in
        entities.iter_with((&mut lit_grenades, &element_handles, &spawners))
    {
        let element_meta = assets.get(element_handle.0);
        let asset = assets.get(element_meta.data);
        let Ok(GrenadeMeta {
            explosion_sound,
            explosion_volume,
            damage_region_lifetime,
            damage_region_size,
            explosion_lifetime,
            explosion_atlas,
            explosion_fps,
            explosion_frames,
            fin_anim,
            ..
        }) = asset.try_cast_ref()
        else {
            unreachable!();
        };

        grenade.fuse_time.tick(time.delta());

        if !emote_regions.contains(entity) {
            emote_regions.insert(
                entity,
                EmoteRegion {
                    active: true,
                    emote: Emote::Alarm,
                    owner: Some(grenade.owner),
                    direction_sensitive: true,
                    size: *damage_region_size * 2.0,
                    buffer: Some(Timer::new(Duration::from_millis(400), TimerMode::Once)),
                },
            );
        }
        let emote_region = emote_regions.get_mut(entity).unwrap();

        // If the item is being held
        if let Some(inventory) = player_inventories.find_item(entity) {
            let player = inventory.player;
            let layers = player_layers.get_mut(player).unwrap();
            layers.fin_anim = *fin_anim;

            emote_region.active = false;

        // If the item is not being held
        } else {
            emote_region.active = true;
        }

        // If it's time to explode
        if grenade.fuse_time.finished() {
            audio_center.play_sound(*explosion_sound, *explosion_volume);

            trauma_events.send(5.0);

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
