use crate::prelude::*;

/// A crab roaming on the ocean floor
#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("crab"))]
#[repr(C)]
pub struct CrabMeta {
    pub body_size: Vec2,
    pub walk_frames: SVec<u32>,
    pub spawn_frames: SVec<u32>,
    pub fps: f32,
    pub comfortable_spawn_distance: f32,
    pub comfortable_scared_distance: f32,
    /// How long a crab has to be away from it's spawn point before it digs into the ground and
    /// digs back out in his spawn point.
    pub uncomfortable_respawn_time: Duration,
    pub same_level_threshold: f32,
    pub walk_speed: f32,
    pub run_speed: f32,
    // TODO: migrate this to a duration like `uncomfortable_respawn_time`.
    pub timer_delay_max: u8,
    pub atlas: Handle<Atlas>,
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_crabs);
}

pub fn game_plugin(game: &mut Game) {
    CrabMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

#[derive(Default, Clone, HasSchema, Debug, Copy)]
pub enum CrabState {
    #[default]
    Paused,
    Walking {
        left: bool,
    },
    Fleeing {
        scared_of: Entity,
    },
    Spawning,
    Despawning,
}

impl CrabState {
    fn is_moving(&self) -> bool {
        matches!(self, CrabState::Walking { .. } | CrabState::Fleeing { .. })
    }
}

#[derive(HasSchema, Clone, Default)]
pub struct CrabCritter {
    state: CrabState,
    start_pos: Vec2,
    state_count: u8,
    state_count_max: u8,
    respawn_timer: Timer,
}

fn hydrate(
    game_meta: Root<GameMeta>,
    mut entities: ResMutInit<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut despawns: CompMut<DehydrateOutOfBounds>,
    mut element_handles: CompMut<ElementHandle>,
    mut crabs: CompMut<CrabCritter>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut spawner_manager: SpawnerManager,
    assets: Res<AssetServer>,
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

        if let Ok(CrabMeta {
            atlas,
            body_size,
            spawn_frames,
            uncomfortable_respawn_time,
            ..
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            crabs.insert(
                entity,
                CrabCritter {
                    state_count: 0,
                    state_count_max: 60,
                    state: CrabState::Spawning,
                    start_pos: Vec2::new(transform.translation.x, transform.translation.y),
                    respawn_timer: Timer::from_seconds(
                        uncomfortable_respawn_time.as_secs_f32(),
                        TimerMode::Once,
                    ),
                },
            );
            atlas_sprites.insert(entity, AtlasSprite::new(*atlas));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle);
            hydrated.insert(entity, MapElementHydrated);
            despawns.insert(entity, DehydrateOutOfBounds(spawner_ent));

            bodies.insert(
                entity,
                KinematicBody {
                    gravity: game_meta.core.physics.gravity,
                    has_mass: true,
                    has_friction: true,
                    shape: ColliderShape::Rectangle { size: *body_size },
                    ..default()
                },
            );

            animated_sprites.insert(
                entity,
                AnimatedSprite {
                    repeat: false,
                    frames: spawn_frames.iter().cloned().collect(),
                    ..default()
                },
            );

            spawner_manager.create_spawner(spawner_ent, vec![entity])
        }
    }
}

fn update_crabs(
    rng: Res<GlobalRng>,
    time: Res<Time>,
    entities: Res<Entities>,
    player_indexes: Comp<PlayerIdx>,
    mut crabs: CompMut<CrabCritter>,
    mut sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut commands: Commands,
    spawners: Comp<DehydrateOutOfBounds>,
    element_handles: Comp<ElementHandle>,
    assets: Res<AssetServer>,
) {
    for (entity, (crab, element_handle, spawner)) in
        entities.iter_with((&mut crabs, &element_handles, &spawners))
    {
        let element_meta = assets.get(element_handle.0);

        let asset = assets.get(element_meta.data);
        let CrabMeta {
            fps,
            run_speed,
            walk_speed,
            walk_frames,
            spawn_frames,
            comfortable_spawn_distance,
            comfortable_scared_distance,
            same_level_threshold,
            timer_delay_max,
            ..
        } = asset.cast_ref();

        let body = bodies.get_mut(entity).unwrap();
        let sprite = sprites.get_mut(entity).unwrap();
        let animated_sprite = animated_sprites.get_mut(entity).unwrap();
        let transform = transforms.get_mut(entity).unwrap();
        let pos = Vec2::new(transform.translation.x, transform.translation.y);

        if (crab.start_pos - pos).length() > *comfortable_spawn_distance {
            crab.respawn_timer.tick(time.delta());
        } else {
            crab.respawn_timer.reset();
        }

        if let CrabState::Despawning = crab.state {
            if finished_playing(animated_sprite).unwrap() {
                crab.state = CrabState::Spawning;
                hydrated.remove(**spawner);
                commands.add(move |mut entities: ResMutInit<Entities>| entities.kill(entity));
            }
            continue;
        } else if body.is_on_ground && crab.respawn_timer.finished() {
            crab.state = CrabState::Despawning;
            body.is_deactivated = true;
            *animated_sprite = AnimatedSprite {
                frames: spawn_frames.iter().rev().cloned().collect(),
                repeat: false,
                fps: *fps,
                ..default()
            };
            continue;
        }

        if let CrabState::Spawning = crab.state {
            if body.is_on_ground {
                body.velocity = Vec2::ZERO;
                animated_sprite.fps = *fps;
                if finished_playing(animated_sprite).unwrap() {
                    crab.state = CrabState::Paused;
                    *animated_sprite = AnimatedSprite {
                        frames: walk_frames.iter().cloned().collect(),
                        repeat: true,
                        fps: *fps,
                        ..default()
                    };
                }
            }
            continue;
        }

        crab.state_count += 1;
        let rand_bool = |true_bias: u8| -> bool { rng.u8(0..(1_u8 + true_bias)) > 0 };
        let rand_timer_delay = |max: u8| rng.u8(0..max);

        let next_scary_thing = || {
            for (scary_entity, (_player_idx, scary_thing_transform)) in
                entities.iter_with((&player_indexes, &transforms))
            {
                let scary_thing_pos = Vec2::new(
                    scary_thing_transform.translation.x,
                    scary_thing_transform.translation.y,
                );

                if pos.distance(scary_thing_pos) < *comfortable_scared_distance
                    && (pos.y - scary_thing_pos.y).abs() < *same_level_threshold
                {
                    return Some(scary_entity);
                }
            }
            None
        };

        let pick_next_move = |crab: &CrabCritter| {
            let pause_bias = if crab.state.is_moving() { 2 } else { 0 };

            if rand_bool(pause_bias) {
                (CrabState::Paused, rand_timer_delay(*timer_delay_max))
            } else {
                let distance_from_home = pos.x - crab.start_pos.x;
                let left = if distance_from_home.abs() > *comfortable_spawn_distance && rand_bool(2)
                {
                    distance_from_home > 0.0
                } else {
                    rand_bool(0)
                };
                (
                    CrabState::Walking { left },
                    rand_timer_delay(*timer_delay_max),
                )
            }
        };

        let get_scared_of_pos = |scared_of: Entity| {
            transforms
                .get(scared_of)
                .map(|x| x.translation.xy())
                .unwrap_or_default()
        };

        if crab.state_count >= crab.state_count_max {
            crab.state_count = 0;

            if let Some(scared_of_pos) = next_scary_thing() {
                crab.state = CrabState::Fleeing {
                    scared_of: scared_of_pos,
                };
                crab.state_count_max = rand_timer_delay(*timer_delay_max);
            } else {
                match &crab.state {
                    CrabState::Paused | CrabState::Walking { .. } => {
                        let (state, timer) = pick_next_move(crab);
                        crab.state = state;
                        crab.state_count_max = timer;
                    }
                    CrabState::Fleeing { scared_of } => {
                        let scared_of_pos = get_scared_of_pos(*scared_of);

                        if pos.distance(scared_of_pos) > *comfortable_scared_distance {
                            if let Some(scared_of) = next_scary_thing() {
                                crab.state = CrabState::Fleeing { scared_of };
                                crab.state_count_max = rand_timer_delay(*timer_delay_max / 3);
                            } else {
                                let (state, timer) = pick_next_move(crab);
                                crab.state = state;
                                crab.state_count_max = timer;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        match &crab.state {
            CrabState::Paused => {
                animated_sprite.fps = 0.0;
                body.velocity.x = 0.0;
            }
            CrabState::Walking { left } => {
                animated_sprite.fps = *fps;
                sprite.flip_x = *left;

                let direction = if *left { -1.0 } else { 1.0 };
                body.velocity.x = *walk_speed * direction;
            }
            CrabState::Fleeing { scared_of } => {
                animated_sprite.fps = *fps;
                let scared_of_pos = get_scared_of_pos(*scared_of);
                let direction = (pos.x - scared_of_pos.x).signum();
                body.velocity.x = direction * *run_speed;
            }
            _ => {}
        }
    }
}

fn finished_playing(
    AnimatedSprite {
        index,
        frames,
        fps,
        timer,
        repeat,
    }: &AnimatedSprite,
) -> Option<bool> {
    (!repeat)
        .then(|| *index == frames.len() as u32 - 1 && *timer > 1.0 / fps.max(f32::MIN_POSITIVE))
}
