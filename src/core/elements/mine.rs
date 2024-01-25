use crate::prelude::*;

/// The mine item
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("mine"))]
#[repr(C)]
pub struct MineMeta {
    pub atlas: Handle<Atlas>,

    pub damage_region_size: Vec2,
    pub damage_region_lifetime: f32,
    pub explosion_atlas: Handle<Atlas>,
    pub explosion_lifetime: f32,
    pub explosion_frames: u32,
    pub explosion_fps: f32,
    pub explosion_volume: f64,
    pub explosion_sound: Handle<AudioSource>,

    /// The delay after throwing the mine, before it becomes armed and will blow up on contact.
    pub arm_delay: f32,
    pub armed_frames: u32,
    pub armed_fps: f32,
    pub arm_sound_volume: f64,
    pub arm_sound: Handle<AudioSource>,

    pub throw_velocity: f32,
    pub body_size: Vec2,
    pub grab_offset: Vec2,
    pub fin_anim: Ustr,
    pub bounciness: f32,
}

pub fn game_plugin(game: &mut Game) {
    MineMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_thrown_mines)
        .add_system_to_stage(CoreStage::PostUpdate, update_idle_mines);
}

#[derive(Clone, HasSchema, Default, Debug, Copy)]
pub struct IdleMine;

#[derive(Clone, HasSchema, Default, Debug)]
pub struct ThrownMine {
    // The mine won't explode until this timer finishes.
    arm_delay: Timer,
}

fn hydrate(
    game_meta: Root<GameMeta>,
    mut entities: ResMutInit<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    assets: Res<AssetServer>,
    mut idle_mines: CompMut<IdleMine>,
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

        if let Ok(MineMeta {
            atlas,
            fin_anim,
            grab_offset,
            body_size,
            bounciness,
            throw_velocity,
            ..
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            items.insert(entity, Item);
            idle_mines.insert(entity, IdleMine);
            atlas_sprites.insert(entity, AtlasSprite::new(*atlas));
            item_throws.insert(entity, ItemThrow::strength(*throw_velocity));
            item_grabs.insert(
                entity,
                ItemGrab {
                    fin_anim: *fin_anim,
                    sync_animation: false,
                    grab_offset: *grab_offset,
                },
            );
            respawn_points.insert(entity, DehydrateOutOfBounds(spawner_ent));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle);
            hydrated.insert(entity, MapElementHydrated);
            animated_sprites.insert(entity, default());
            bodies.insert(
                entity,
                KinematicBody {
                    shape: ColliderShape::Rectangle { size: *body_size },
                    has_mass: true,
                    has_friction: true,
                    bounciness: *bounciness,
                    gravity: game_meta.core.physics.gravity,
                    ..default()
                },
            );
            spawner_manager.create_spawner(spawner_ent, vec![entity])
        }
    }
}

fn update_idle_mines(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut idle_mines: CompMut<IdleMine>,
    mut items_used: CompMut<ItemUsed>,
    player_inventories: PlayerInventories,
    mut commands: Commands,
) {
    for (entity, (_mine, element_handle)) in entities.iter_with((&mut idle_mines, &element_handles))
    {
        let element_meta = assets.get(element_handle.0);

        let asset = assets.get(element_meta.data);
        let Ok(MineMeta { arm_delay, .. }) = asset.try_cast_ref() else {
            unreachable!();
        };
        let arm_delay = *arm_delay;

        if let Some(Inv { player, .. }) = player_inventories.find_item(entity) {
            if items_used.remove(entity).is_some() {
                commands.add(PlayerCommand::set_inventory(player, None));
                commands.add(
                    move |mut idle: CompMut<IdleMine>, mut thrown: CompMut<ThrownMine>| {
                        idle.remove(entity);
                        thrown.insert(
                            entity,
                            ThrownMine {
                                arm_delay: Timer::new(
                                    Duration::from_secs_f32(arm_delay),
                                    TimerMode::Once,
                                ),
                            },
                        );
                    },
                );
            }
        }
    }
}

fn update_thrown_mines(
    entities: Res<Entities>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
    mut audio_center: ResMut<AudioCenter>,
    mut trauma_events: ResMutInit<CameraTraumaEvents>,
    mut thrown_mines: CompMut<ThrownMine>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut hydrated: CompMut<MapElementHydrated>,
    player_indexes: Comp<PlayerIdx>,
    mut commands: Commands,
    collision_world: CollisionWorld,
    transforms: Comp<Transform>,
    time: Res<Time>,
    spawners: Comp<DehydrateOutOfBounds>,
    invincibles: CompMut<Invincibility>,
) {
    let players = entities
        .iter_with(&player_indexes)
        .map(|x| x.0)
        .collect::<Vec<_>>();
    for (entity, (thrown_mine, element_handle, sprite, spawner)) in entities.iter_with((
        &mut thrown_mines,
        &element_handles,
        &mut animated_sprites,
        &spawners,
    )) {
        let element_meta = assets.get(element_handle.0);

        let asset = assets.get(element_meta.data);
        let Ok(MineMeta {
            explosion_fps,
            explosion_frames,
            explosion_sound,
            explosion_atlas,
            arm_sound,
            armed_frames,
            armed_fps,
            damage_region_size,
            damage_region_lifetime,
            explosion_volume,
            arm_sound_volume,
            explosion_lifetime,
            ..
        }) = asset.try_cast_ref()
        else {
            unreachable!();
        };

        thrown_mine.arm_delay.tick(time.delta());

        if thrown_mine.arm_delay.just_finished() {
            audio_center.play_sound(*arm_sound, *arm_sound_volume);

            sprite.frames = (0..*armed_frames).collect();
            sprite.fps = *armed_fps;
            sprite.repeat = true;
        }

        let colliding_with_players = collision_world
            .actor_collisions_filtered(entity, |e| {
                players.contains(&e) && invincibles.get(e).is_none()
            })
            .into_iter()
            .collect::<Vec<_>>();

        if !colliding_with_players.is_empty() && thrown_mine.arm_delay.finished() {
            let mine_transform = *transforms.get(entity).unwrap();

            trauma_events.send(6.0);

            for player in &colliding_with_players {
                commands.add(PlayerCommand::kill(
                    *player,
                    Some(mine_transform.translation.xy()),
                ));
            }

            audio_center.play_sound(*explosion_sound, *explosion_volume);

            hydrated.remove(**spawner);

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
                    let mut explosion_transform = mine_transform;
                    explosion_transform.translation.z = -10.0; // On top of almost everything
                    explosion_transform.rotation = Quat::IDENTITY;

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
