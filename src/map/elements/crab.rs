use super::*;
use crate::random::GlobalRng;
use bevy::prelude::*;

pub struct CrabDecorationPlugin;

impl Plugin for CrabDecorationPlugin {
    fn build(&self, app: &mut App) {
        app.add_rollback_system(RollbackStage::PreUpdate, hydrate_crab)
            .add_rollback_system(RollbackStage::Update, update_crabs)
            .extend_rollback_plugin(|plugin| plugin.register_rollback_type::<CrabCritter>());
    }
}

fn hydrate_crab(
    mut commands: Commands,
    non_hydrated_map_elements: Query<
        (Entity, &Handle<MapElementMeta>, &Transform),
        Without<MapElementHydrated>,
    >,
    element_assets: Res<Assets<MapElementMeta>>,
) {
    // Hydrate the newly spawned crabs
    for (entity, map_element_meta, initial_pos) in &non_hydrated_map_elements {
        let map_element = element_assets.get(map_element_meta).unwrap();
        if let BuiltinElementKind::Crab {
            start_frame,
            end_frame,
            fps,
            atlas_handle,
            comfortable_spawn_distance,
            comfortable_scared_distance,
            same_level_threshold,
            walk_speed,
            run_speed,
            timer_delay_max,
            ..
        } = &map_element.builtin
        {
            let config = CrabConfig {
                comfortable_spawn_distance: *comfortable_spawn_distance,
                comfortable_scared_distance: *comfortable_scared_distance,
                same_level_threshold: *same_level_threshold,
                walk_speed: *walk_speed,
                run_speed: *run_speed,
                timer_delay_max: *timer_delay_max,
            };

            commands
                .entity(entity)
                .insert(MapElementHydrated)
                .insert(Name::new("Environment: Crab"))
                .insert(AnimatedSprite {
                    start: *start_frame,
                    end: *end_frame,
                    atlas: atlas_handle.inner.clone(),
                    repeat: true,
                    fps: *fps,
                    ..default()
                })
                .insert(CrabCritter {
                    state: CrabState::Paused,
                    state_count: 0,
                    state_count_max: 60,
                    config,
                    start_pos: Vec2::new(initial_pos.translation.x, initial_pos.translation.y),
                })
                .insert(KinematicBody {
                    size: Vec2::new(17.0, 12.0),
                    offset: Vec2::new(0.0, 0.0),
                    gravity: 1.0,
                    has_mass: true,
                    has_friction: true,
                    ..default()
                });
        }
    }
}

#[derive(Reflect, Copy, Clone, PartialEq, Serialize, Deserialize)]
#[reflect_value(PartialEq, Serialize, Deserialize)]
pub enum CrabState {
    Paused,
    Walking { left: bool },
    Fleeing { scared_of: Vec2 },
}

impl CrabState {
    fn is_moving(&self) -> bool {
        matches!(self, CrabState::Walking { .. } | CrabState::Fleeing { .. })
    }
}

impl Default for CrabState {
    fn default() -> Self {
        Self::Paused
    }
}

#[derive(Reflect, Default, Clone)]
pub struct CrabConfig {
    comfortable_spawn_distance: f32,
    comfortable_scared_distance: f32,
    same_level_threshold: f32,
    walk_speed: f32,
    run_speed: f32,
    timer_delay_max: u8,
}

#[derive(Component, Reflect, Clone)]
pub struct CrabCritter {
    state: CrabState,
    start_pos: Vec2,
    config: CrabConfig,
    state_count: u8,
    state_count_max: u8,
}

impl Default for CrabCritter {
    fn default() -> Self {
        Self {
            state: CrabState::default(),
            start_pos: Vec2::default(),
            config: CrabConfig::default(),
            state_count: u8::default(),
            state_count_max: u8::default(),
        }
    }
}

fn update_crabs(
    mut crab_query: Query<(
        &mut CrabCritter,
        &mut AnimatedSprite,
        &Transform,
        &mut KinematicBody,
    )>,
    scary_things_query: Query<&Transform, (With<PlayerIdx>, Without<CrabCritter>)>,
    rng: Res<GlobalRng>,
) {
    for (mut crab, mut sprite, transform, mut body) in crab_query.iter_mut() {
        let pos = Vec2::new(transform.translation.x, transform.translation.y);

        crab.state_count += 1;
        let mut rand_bool = |true_bias: u8| -> bool { rng.u8(0..(1_u8 + true_bias)) > 0 };
        let mut rand_timer_delay = |max: u8| rng.u8(0..max);

        let next_scary_thing = |crab: &CrabCritter| {
            for scary_thing_transform in scary_things_query.iter() {
                let scary_thing_pos = Vec2::new(
                    scary_thing_transform.translation.x,
                    scary_thing_transform.translation.y,
                );

                if pos.distance(scary_thing_pos) < crab.config.comfortable_scared_distance
                    && (pos.y - scary_thing_pos.y).abs() < crab.config.same_level_threshold
                {
                    return Some(scary_thing_pos);
                }
            }
            None
        };

        let mut pick_next_move = |crab: &CrabCritter| {
            let distance_from_home = pos.x - crab.start_pos.x;
            let pause_bias = if crab.state.is_moving() { 2 } else { 0 };

            if rand_bool(pause_bias) {
                (
                    CrabState::Paused,
                    rand_timer_delay(crab.config.timer_delay_max),
                )
            } else {
                let left = if distance_from_home > crab.config.comfortable_spawn_distance
                    && rand_bool(2)
                {
                    distance_from_home > 0.0
                } else {
                    rand_bool(0)
                };
                (
                    CrabState::Walking { left },
                    rand_timer_delay(crab.config.timer_delay_max),
                )
            }
        };

        if crab.state_count >= crab.state_count_max {
            crab.state_count = 0;
            if let Some(scared_of_pos) = next_scary_thing(&crab) {
                crab.state = CrabState::Fleeing {
                    scared_of: scared_of_pos,
                };
                crab.state_count_max = rand_timer_delay(crab.config.timer_delay_max);
            } else {
                match &crab.state {
                    CrabState::Paused | CrabState::Walking { .. } => {
                        let (state, timer) = pick_next_move(&crab);
                        crab.state = state;
                        crab.state_count_max = timer;
                    }
                    CrabState::Fleeing { scared_of } => {
                        if pos.distance(*scared_of) > crab.config.comfortable_scared_distance {
                            if let Some(scared_of) = next_scary_thing(&crab) {
                                crab.state = CrabState::Fleeing { scared_of };
                                crab.state_count_max =
                                    rand_timer_delay(crab.config.timer_delay_max / 3);
                            } else {
                                let (state, timer) = pick_next_move(&crab);
                                crab.state = state;
                                crab.state_count_max = timer;
                            }
                        }
                    }
                }
            }
        }

        match &crab.state {
            CrabState::Paused => {
                body.velocity.x = 0.0;
            }
            CrabState::Walking { left } => {
                sprite.flip_x = *left;

                let direction = if *left { -1.0 } else { 1.0 };
                body.velocity.x = crab.config.walk_speed * direction;
            }
            CrabState::Fleeing { scared_of } => {
                let direction = (pos.x - scared_of.x).signum();
                body.velocity.x = direction * crab.config.run_speed;
            }
        }
    }
}
