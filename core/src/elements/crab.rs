use crate::{prelude::*, random::GlobalRng};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PreUpdate, hydrate)
        .add_system_to_stage(CoreStage::PostUpdate, update_crabs);
}

#[derive(Default, Clone, TypeUlid, Debug, Copy)]
#[ulid = "01GQ0HN03T4KVTB95CDZ6YG48G"]
pub enum CrabState {
    #[default]
    Paused,
    Walking {
        left: bool,
    },
    Fleeing {
        scared_of: Entity,
    },
}

impl CrabState {
    fn is_moving(&self) -> bool {
        matches!(self, CrabState::Walking { .. } | CrabState::Fleeing { .. })
    }
}

#[derive(TypeUlid, Clone, Default)]
#[ulid = "01GQ0J08W112T1JVB0QJ42HJSE"]
pub struct CrabCritter {
    state: CrabState,
    start_pos: Vec2,
    state_count: u8,
    state_count_max: u8,
}

fn hydrate(
    game_meta: Res<CoreMetaArc>,
    mut entities: ResMut<Entities>,
    mut hydrated: CompMut<MapElementHydrated>,
    mut element_handles: CompMut<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
    mut crabs: CompMut<CrabCritter>,
    mut atlas_sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    mut animated_sprites: CompMut<AnimatedSprite>,
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

        if let BuiltinElementKind::Crab {
            fps,
            atlas,
            start_frame,
            end_frame,
            ..
        } = &element_meta.builtin
        {
            hydrated.insert(spawner_ent, MapElementHydrated);

            let entity = entities.create();
            crabs.insert(
                entity,
                CrabCritter {
                    state_count: 0,
                    state_count_max: 60,
                    state: CrabState::Paused,
                    start_pos: Vec2::new(transform.translation.x, transform.translation.y),
                },
            );
            atlas_sprites.insert(entity, AtlasSprite::new(atlas.clone()));
            transforms.insert(entity, transform);
            element_handles.insert(entity, element_handle.clone());
            hydrated.insert(entity, MapElementHydrated);

            bodies.insert(
                entity,
                KinematicBody {
                    gravity: game_meta.physics.gravity,
                    has_mass: true,
                    has_friction: true,
                    size: Vec2::new(17.0, 12.0),
                    offset: Vec2::new(0.0, 0.0),
                    ..default()
                },
            );

            animated_sprites.insert(
                entity,
                AnimatedSprite {
                    fps: *fps,
                    repeat: true,
                    frames: (*start_frame..*end_frame).collect(),
                    ..default()
                },
            );
        }
    }
}

fn update_crabs(
    rng: Res<GlobalRng>,
    entities: Res<Entities>,
    player_indexes: Comp<PlayerIdx>,
    mut crabs: CompMut<CrabCritter>,
    mut sprites: CompMut<AtlasSprite>,
    mut bodies: CompMut<KinematicBody>,
    mut transforms: CompMut<Transform>,
    element_handles: Comp<ElementHandle>,
    element_assets: BevyAssets<ElementMeta>,
) {
    for (entity, (crab, element_handle)) in entities.iter_with((&mut crabs, &element_handles)) {
        let Some(element_meta) = element_assets.get(&element_handle.get_bevy_handle()) else {
            continue;
        };

        let BuiltinElementKind::Crab {
            run_speed,
            walk_speed,
            comfortable_spawn_distance,
            comfortable_scared_distance,
            same_level_threshold,
            timer_delay_max,
            ..
        } = &element_meta.builtin else {
            unreachable!();
        };

        let transform = transforms.get_mut(entity).unwrap();
        let pos = Vec2::new(transform.translation.x, transform.translation.y);

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
            let distance_from_home = pos.x - crab.start_pos.x;
            let pause_bias = if crab.state.is_moving() { 2 } else { 0 };

            if rand_bool(pause_bias) {
                (CrabState::Paused, rand_timer_delay(*timer_delay_max))
            } else {
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
            let scared_of_transform = transforms.get(scared_of).unwrap();

            Vec2::new(
                scared_of_transform.translation.x,
                scared_of_transform.translation.y,
            )
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
                }
            }
        }

        let body = bodies.get_mut(entity).unwrap();
        let sprite = sprites.get_mut(entity).unwrap();

        match &crab.state {
            CrabState::Paused => {
                body.velocity.x = 0.0;
            }
            CrabState::Walking { left } => {
                sprite.flip_x = *left;

                let direction = if *left { -1.0 } else { 1.0 };
                body.velocity.x = *walk_speed * direction;
            }
            CrabState::Fleeing { scared_of } => {
                let scared_of_pos = get_scared_of_pos(*scared_of);
                let direction = (pos.x - scared_of_pos.x).signum();
                body.velocity.x = direction * *run_speed;
            }
        }
    }
}
