use crate::prelude::*;

#[derive(HasSchema, Default, Debug, Clone)]
#[type_data(metadata_asset("fish_school"))]
#[repr(C)]
pub struct FishSchoolMeta {
    pub kinds: SVec<Handle<Atlas>>,
    /// The default and most-likely to ocurr number of fish in a school
    pub base_count: u32,
    /// The ammount greater or less than the base number of fish that may spawn
    pub count_variation: u32,
    /// The distance from the spawn point on each axis that the individual fish in the school will be
    /// initially spawned within
    pub spawn_range: f32,
    /// The distance that the fish wish to stay within the center of their school
    pub school_size: f32,
    // The distance a collider must be for the fish to run away
    pub flee_range: f32,
}

pub fn game_plugin(game: &mut Game) {
    FishSchoolMeta::register_schema();
    game.init_shared_resource::<AssetServer>();
}

pub fn session_plugin(session: &mut Session) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_fish_schools);
}

#[derive(Default, Clone, HasSchema, Debug, Deref, DerefMut)]
pub struct FishSchool {
    fish: Vec<Entity>,
}

#[derive(Clone, HasSchema, Debug)]
#[schema(no_default)]
pub struct Fish {
    state: FishState,
    state_timer: Timer,
}

#[derive(Debug, Clone)]
pub enum FishState {
    /// Moving to a location
    Moving { from: Vec2, to: Vec2 },
}

pub fn hydrate(
    rng: Res<GlobalRng>,
    mut entities: ResMutInit<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    assets: Res<AssetServer>,
    mut fish_schools: CompMut<FishSchool>,
    mut fishes: CompMut<Fish>,
    mut transforms: CompMut<Transform>,
    mut actors: CompMut<Actor>,
    mut colliders: CompMut<Collider>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut animated_sprites: CompMut<AnimatedSprite>,
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
        if let Ok(FishSchoolMeta {
            kinds,
            base_count,
            count_variation,
            spawn_range,
            flee_range,
            ..
        }) = assets.get(element_meta.data).try_cast_ref()
        {
            // We can use the spawner entity itself for our
            // school because there is no respawning.
            let fish_school_ent = spawner_ent;

            hydrated.insert(fish_school_ent, MapElementHydrated);
            transforms.insert(fish_school_ent, transform);
            element_handles.insert(fish_school_ent, element_handle);

            let mut fish_ents = Vec::new();

            let rand_bool = || rng.u8(0u8..2) == 0;

            let mut fish_count = *base_count;
            if rand_bool() {
                for _ in 0..*count_variation {
                    if rand_bool() {
                        fish_count = if rand_bool() {
                            fish_count.saturating_sub(1)
                        } else {
                            fish_count.saturating_add(1)
                        };
                    }
                }
            }

            let spawn_point = transform.translation.xy();

            for _ in 0..fish_count {
                let spawn_point = spawn_point
                    + vec2(
                        rng.f32_normalized() * spawn_range,
                        rng.f32_normalized() * spawn_range,
                    );

                let duration = {
                    let min = 0.2;
                    let max = 1.0;
                    rng.f32() * (max - min) + min
                };

                let atlas = &kinds[rng.usize(0..kinds.len())];

                let fish_ent = entities.create();
                fishes.insert(
                    fish_ent,
                    Fish {
                        state: FishState::Moving {
                            from: spawn_point,
                            to: spawn_point,
                        },
                        state_timer: Timer::new(
                            Duration::from_secs_f32(duration),
                            TimerMode::Repeating,
                        ),
                    },
                );
                transforms.insert(fish_ent, transform);
                actors.insert(fish_ent, Actor);
                colliders.insert(
                    fish_ent,
                    Collider {
                        shape: ColliderShape::Circle {
                            diameter: flee_range * 2.0,
                        },
                        ..default()
                    },
                );
                atlas_sprites.insert(fish_ent, AtlasSprite::new(*atlas));
                animated_sprites.insert(
                    fish_ent,
                    AnimatedSprite {
                        frames: [0, 1, 2, 3].into_iter().collect(),
                        fps: 6.0,
                        repeat: true,
                        ..default()
                    },
                );

                fish_ents.push(fish_ent);
            }
            fish_schools.insert(
                fish_school_ent,
                FishSchool {
                    fish: fish_ents.clone(),
                },
            );
            spawner_manager.create_spawner(fish_school_ent, fish_ents);
        }
    }
}

pub fn update_fish_schools(
    rng: Res<GlobalRng>,
    time: Res<Time>,
    entities: Res<Entities>,
    fish_schools: Comp<FishSchool>,
    mut fishes: CompMut<Fish>,
    element_handles: CompMut<ElementHandle>,
    assets: Res<AssetServer>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut transforms: CompMut<Transform>,
    collision_world: CollisionWorld,
    bodies: Comp<KinematicBody>,
) {
    for (school_ent, school) in entities.iter_with(&fish_schools) {
        let element_handle = element_handles.get(school_ent).unwrap();
        let element_meta = assets.get(element_handle.0);
        let asset = assets.get(element_meta.data);
        let Ok(FishSchoolMeta { school_size, .. }) = asset.try_cast_ref() else {
            continue;
        };

        let transform = transforms.get(school_ent).unwrap();
        let mut fish_transforms = school
            .fish
            .iter()
            .map(|entity| transforms.get(*entity).unwrap());
        let Some(fish_transform) = fish_transforms.next() else {
            continue;
        };

        let mut school_bounds_min = fish_transform.translation.xy();
        let mut school_bounds_max = fish_transform.translation.xy();

        for transform in fish_transforms {
            let pos = &transform.translation.xy();

            school_bounds_min.x = school_bounds_min.x.min(pos.x);
            school_bounds_min.y = school_bounds_min.y.min(pos.y);
            school_bounds_max.x = school_bounds_max.x.max(pos.x);
            school_bounds_max.y = school_bounds_max.y.max(pos.y);
        }

        let size = school_bounds_max - school_bounds_min;
        let center = school_bounds_min + size / 2.0;
        let is_grouped = size.x.max(size.y) < *school_size;
        let spawn_pos = transform.translation.xy();

        for fish_ent in school.fish.iter() {
            let flee = collision_world
                .actor_collisions(*fish_ent)
                .into_iter()
                .find(|ent| {
                    !fishes.contains(*ent)
                        && bodies
                            .get(*ent)
                            .map(|x| x.velocity != Vec2::ZERO)
                            .unwrap_or_default()
                });

            let fish = fishes.get_mut(*fish_ent).unwrap();
            let sprite = atlas_sprites.get_mut(*fish_ent).unwrap();

            let mut pos = transforms.get(*fish_ent).unwrap().translation.xy();

            let rand_bool = || rng.u8(0u8..2) == 0;
            let rand_range = |min: f32, max: f32| rng.f32() * (max - min) + min;

            let pick_next_move = || {
                if !is_grouped {
                    let target_point = pos.lerp(center, rand_range(0.1, 0.4));

                    (
                        FishState::Moving {
                            from: pos,
                            to: target_point,
                        },
                        Timer::new(
                            Duration::from_secs_f32(rand_range(0.2, 0.7)),
                            TimerMode::Repeating,
                        ),
                    )
                } else if rand_bool() {
                    let target_point = vec2(
                        pos.x + rand_range(-20.0, 20.0),
                        pos.y + rand_range(-20.0, 20.0),
                    );
                    (
                        FishState::Moving {
                            from: pos,
                            to: target_point,
                        },
                        Timer::new(
                            Duration::from_secs_f32(rand_range(0.5, 1.5)),
                            TimerMode::Repeating,
                        ),
                    )
                } else {
                    let target_point = pos.lerp(spawn_pos, rand_range(0.10, 0.25));
                    (
                        FishState::Moving {
                            from: pos,
                            to: target_point,
                        },
                        Timer::new(
                            Duration::from_secs_f32(rand_range(0.5, 1.5)),
                            TimerMode::Repeating,
                        ),
                    )
                }
            };

            fish.state_timer.tick(time.delta());

            if let Some(scary_thing) = flee {
                let diff = pos - transforms.get(scary_thing).unwrap().translation.xy();
                fish.state = FishState::Moving {
                    from: pos,
                    to: pos + diff.normalize() * rand_range(30.0, 60.0),
                };
                fish.state_timer = Timer::new(
                    Duration::from_secs_f32(rand_range(0.2, 0.6)),
                    TimerMode::Repeating,
                );
                // We tick the timer an extra time here to make sure that the fish gets moving
                // immediately without waiting for an extra frame, because if we keep colliding we
                // may just keep re-setting the timer and the fish get's stuck until it stops
                // colliding.
                fish.state_timer.tick(time.delta());
            } else if fish.state_timer.finished() {
                let (state, timer) = pick_next_move();
                fish.state = state;
                fish.state_timer = timer;
            }

            match &fish.state {
                FishState::Moving { from, to } => {
                    let lerp_progress = Ease {
                        ease_in: false,
                        ease_out: true,
                        function: EaseFunction::Sinusoidial,
                        progress: fish.state_timer.percent(),
                    }
                    .output();

                    sprite.flip_x = from.x > to.x;
                    sprite.index = (4.0 * lerp_progress).floor() as u32;

                    pos = from.lerp(*to, lerp_progress);
                }
            }
            let transform = transforms.get_mut(*fish_ent).unwrap();
            transform.translation.x = pos.x;
            transform.translation.y = pos.y;
        }
    }
}
