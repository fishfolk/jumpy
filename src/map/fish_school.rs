use core::Transform;

use fishsticks::error::Result;
use hecs::{Entity, World};
use macroquad::{
    prelude::{collections::storage, vec2, Color, Rect, Vec2},
    rand,
};

use crate::{
    player::Player,
    utils::{ease::Ease, timer::Timer},
    AnimatedSpriteParams, Animation, Drawable, PhysicsBody, Resources, RigidBody,
};

/// The texture of the fish school icon ( used in the editor to represent a school of fish )
pub const FISH_SCHOOL_ICON_TEXTURE_ID: &str = "fish_school_icon";

/// List of fish textures
const FISH_TEXTURE_IDS: &[&str] = &[
    "blue_tang",
    "royal_gramma",
    "arabian_angelfish",
    "blue_green_chromis",
    "banded_butterfly_fish",
    // "small_fish1",
    // "small_fish2",
    // "small_fish3",
];
/// The default and most-likely to ocurr number of fish in a school
const FISH_COUNT_BASE: u32 = 3;
/// The ammount greater or less than the base number of fish that may spawn
const FISH_COUNT_RANGE: u32 = 2;
/// The distance from the spawn point on each axis that the individual fish in the school will be
/// initially spawned within
const FISH_SPAWN_RANGE: f32 = 64.0;
/// The distance that the fish wish to stay within the center of their school
const TARGET_SCHOOL_SIZE: f32 = 100.0;

/// Minimum draw order
const DRAW_ORDER_MIN: u32 = 0;
/// Maximum draw order
const DRAW_ORDER_MAX: u32 = 100;

/// The color to debug draw school bounds
const GROUPED_SCHOOL_BOUNDS_DEBUG_DRAW_COLOR: Color = Color {
    r: 0.0,
    g: 1.0,
    b: 0.0,
    a: 1.0,
};
/// The color to debug draw school bounds when the school is not grouped
const UNGROUPED_SCHOOL_BOUNDS_DEBUG_DRAW_COLOR: Color = Color {
    r: 1.0,
    g: 0.0,
    b: 1.0,
    a: 1.0,
};

pub struct FishSchool {
    pub spawn_pos: Vec2,
    pub fish_entities: Vec<Entity>,
}

pub struct Fish {
    state: FishState,
    state_timer: Timer,
}

#[derive(Debug)]
pub enum FishState {
    /// Moving to a location
    Moving { from: Vec2, to: Vec2 },
}

pub fn spawn_fish_school(world: &mut World, spawn_position: Vec2) -> Result<Entity> {
    let resources = storage::get::<Resources>();
    let fish_school_icon_sprite = resources.textures.get(FISH_SCHOOL_ICON_TEXTURE_ID).unwrap();
    let fish_school_icon_sprite_size = fish_school_icon_sprite.meta.frame_size.unwrap();

    let rand_bool = || rand::gen_range(0u8, 2) == 0;

    let mut fish_count = FISH_COUNT_BASE as i32;
    if rand_bool() {
        let sign = if rand_bool() { -1 } else { 1 };
        for _ in 0..FISH_COUNT_RANGE {
            if rand_bool() {
                fish_count = fish_count.saturating_add(sign);
            }
        }
    }

    let mut fish_school = FishSchool {
        spawn_pos: spawn_position + fish_school_icon_sprite_size / 2.0,
        fish_entities: Vec::with_capacity(fish_count as usize),
    };
    let fish_school_entity = world.reserve_entity();

    let fish_spawn_min = spawn_position;
    let fish_spawn_max = spawn_position + Vec2::splat(FISH_SPAWN_RANGE);

    for _ in 0..fish_count {
        let spawn_point = vec2(
            rand::gen_range(fish_spawn_min.x, fish_spawn_max.x),
            rand::gen_range(fish_spawn_min.y, fish_spawn_max.y),
        );

        let texture_index = rand::gen_range(0, FISH_TEXTURE_IDS.len());
        let texture_id = FISH_TEXTURE_IDS[texture_index];

        let fish_entity = world.spawn((
            Fish {
                state: FishState::Moving {
                    from: spawn_point,
                    to: spawn_point,
                },
                state_timer: Timer::new(rand::gen_range(0.2, 1.0)),
            },
            Transform::from(spawn_point),
            Drawable::new_animated_sprite(
                rand::gen_range(DRAW_ORDER_MIN, DRAW_ORDER_MAX + 1),
                texture_id,
                &[Animation {
                    id: "default".to_string(),
                    row: 0,
                    frames: 4,
                    fps: 3,
                    tweens: Default::default(),
                    is_looping: true,
                }],
                AnimatedSpriteParams {
                    is_flipped_x: rand_bool(),
                    ..Default::default()
                },
            ),
        ));

        fish_school.fish_entities.push(fish_entity);
    }

    world.insert_one(fish_school_entity, fish_school).unwrap();

    Ok(fish_school_entity)
}

pub fn update_fish_schools(world: &mut World) {
    let mut schools = Vec::new();

    for (_, school) in world.query::<&FishSchool>().iter() {
        let school: &FishSchool = school;

        let school_info = if let Some(info) = get_school_info(world, school) {
            info
        } else {
            continue;
        };

        schools.push(school_info);
    }

    for school in &schools {
        let school: &SchoolInfo = school;

        let mut collider_rects = world
            .query::<(&Transform, &PhysicsBody)>()
            .with::<Player>()
            .iter()
            .map(|(_, (transform, body))| body.as_rect(transform.position))
            .collect::<Vec<_>>();

        collider_rects.extend(
            world
                .query::<(&Transform, &RigidBody)>()
                .iter()
                .map(|(_, (transform, body))| body.as_rect(transform.position)),
        );

        for fish_entity in &school.fish_entities {
            let (mut fish, drawable, transform) = world
                .query_one_mut::<(&mut Fish, &mut Drawable, &mut Transform)>(*fish_entity)
                .unwrap();
            let sprite = drawable.get_animated_sprite_mut().unwrap();
            let pos: &mut Vec2 = &mut transform.position;
            let padding = 20.0;
            let rect = Rect::new(
                pos.x - padding / 2.0,
                pos.y - padding / 2.0,
                sprite.texture.width() + padding,
                sprite.texture.height() + padding,
            );

            let mut collides_with = None;
            for body in &collider_rects {
                if rect.overlaps(body) {
                    collides_with = Some(body);
                    break;
                }
            }

            let rand_bool = || rand::gen_range(0u8, 2) > 0;
            let rand_delay = |min, max| Timer::new(rand::gen_range(min, max));

            let pick_next_move = || {
                if !school.is_grouped {
                    let target_point = pos.lerp(school.center, rand::gen_range(0.1, 0.4));

                    (
                        FishState::Moving {
                            from: *pos,
                            to: target_point,
                        },
                        rand_delay(0.2, 0.7),
                    )
                } else if rand_bool() {
                    let target_point = vec2(
                        pos.x + rand::gen_range(-20.0, 20.0),
                        pos.y + rand::gen_range(-20.0, 20.0),
                    );
                    (
                        FishState::Moving {
                            from: *pos,
                            to: target_point,
                        },
                        rand_delay(0.5, 1.5),
                    )
                } else {
                    let target_point = pos.lerp(school.spawn_pos, rand::gen_range(0.10, 0.25));
                    (
                        FishState::Moving {
                            from: *pos,
                            to: target_point,
                        },
                        rand_delay(0.5, 1.5),
                    )
                }
            };

            fish.state_timer.tick_frame_time();

            if let Some(collision_rect) = collides_with {
                let collision_center = collision_rect.point() + collision_rect.size() / 2.0;
                let diff = *pos - collision_center;
                fish.state = FishState::Moving {
                    from: *pos,
                    to: *pos + diff.normalize() * rand::gen_range(30.0, 60.0),
                };
                fish.state_timer = rand_delay(0.2, 0.6);

                // We tick the timer an extra time here to make sure that the fish gets moving
                // immediately without waiting for an extra frame, because if we keep colliding we
                // may just keep re-setting the timer and the fish get's stuck until it stops
                // colliding.
                fish.state_timer.tick_frame_time();
            } else if fish.state_timer.has_finished() {
                let (state, timer) = pick_next_move();
                fish.state = state;
                fish.state_timer = timer;
            }

            match &fish.state {
                FishState::Moving { from, to } => {
                    let lerp_progress = Ease {
                        ease_in: false,
                        ease_out: true,
                        function: crate::utils::ease::EaseFunction::Sinusoidial,
                        progress: fish.state_timer.progress(),
                    }
                    .output();

                    sprite.is_flipped_x = from.x > to.x;
                    sprite.current_frame =
                        (sprite.animations[0].frames as f32 * lerp_progress).floor() as u32;

                    *pos = from.lerp(*to, lerp_progress);
                }
            }
        }
    }
}

pub fn debug_draw_fish_schools(world: &mut World) {
    for (_, school) in world.query::<&FishSchool>().iter() {
        let school: &FishSchool = school;
        let school = if let Some(info) = get_school_info(world, school) {
            info
        } else {
            continue;
        };

        macroquad::shapes::draw_rectangle_lines(
            school.bounds_rect.x,
            school.bounds_rect.y,
            school.bounds_rect.w,
            school.bounds_rect.h,
            1.0,
            if school.is_grouped {
                GROUPED_SCHOOL_BOUNDS_DEBUG_DRAW_COLOR
            } else {
                UNGROUPED_SCHOOL_BOUNDS_DEBUG_DRAW_COLOR
            },
        );

        macroquad::shapes::draw_circle_lines(
            school.center.x,
            school.center.y,
            2.0,
            1.0,
            if school.is_grouped {
                GROUPED_SCHOOL_BOUNDS_DEBUG_DRAW_COLOR
            } else {
                UNGROUPED_SCHOOL_BOUNDS_DEBUG_DRAW_COLOR
            },
        );
    }
}

#[derive(Clone)]
struct SchoolInfo {
    spawn_pos: Vec2,
    center: Vec2,
    bounds_rect: Rect,
    is_grouped: bool,
    fish_entities: Vec<Entity>,
}

fn get_school_info(world: &World, school: &FishSchool) -> Option<SchoolInfo> {
    let resources = storage::get::<Resources>();
    let fish_sprite = resources.textures.get(FISH_TEXTURE_IDS[0]).unwrap();
    let fish_size = fish_sprite.meta.frame_size.unwrap();

    let fish_transforms = school
        .fish_entities
        .iter()
        .map(|&x| world.get::<Transform>(x).unwrap())
        .collect::<Vec<_>>();

    if fish_transforms.is_empty() {
        return None;
    }

    let mut school_bounds_min = fish_transforms[0].position;
    let mut school_bounds_max = fish_transforms[0].position;

    for transform in &fish_transforms {
        let pos: &Vec2 = &transform.position;

        school_bounds_min.x = school_bounds_min.x.min(pos.x);
        school_bounds_min.y = school_bounds_min.y.min(pos.y);
        school_bounds_max.x = school_bounds_max.x.max(pos.x + fish_size.x);
        school_bounds_max.y = school_bounds_max.y.max(pos.y + fish_size.y);
    }
    let bounds_rect = Rect::new(
        school_bounds_min.x,
        school_bounds_min.y,
        school_bounds_max.x - school_bounds_min.x,
        school_bounds_max.y - school_bounds_min.y,
    );

    let size = school_bounds_max - school_bounds_min;
    let center = school_bounds_min + size / 2.0;
    let is_grouped = size.x.max(size.y) < TARGET_SCHOOL_SIZE;

    Some(SchoolInfo {
        spawn_pos: school.spawn_pos,
        center,
        bounds_rect,
        is_grouped,
        fish_entities: school.fish_entities.clone(),
    })
}
