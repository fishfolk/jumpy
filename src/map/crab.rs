use core::Transform;

use fishsticks::error::Result;
use hecs::{Entity, With, World};
use macroquad::{
    prelude::{collections::storage, Vec2},
    rand,
};

use crate::{
    player::Player, utils::timer::Timer, Animation, CollisionWorld, Drawable, PhysicsBody,
    PhysicsBodyParams, Resources,
};

pub const CRAB_TEXTURE_ID: &str = "crab";

const DRAW_ORDER: u32 = 2;
/// This horizontal distance away from it's spawn point that a crab is comfortable being
const COMFORTABLE_SPAWN_DISTANCE: f32 = 25.0;
/// The distance away from "scary stuff" that the crab wants to be
const COMFORTABLE_SCARY_DISTANCE: f32 = 100.0;
/// The y difference in position to still consider being as on the same level as the crab
const SAME_LEVEL_THRESHOLD: f32 = 50.0;

const WALK_SPEED: f32 = 0.5;
const RUN_SPEED: f32 = 2.5;

pub struct Crab {
    pub spawn_position: Vec2,
    pub state: CrabState,
    pub state_timer: Timer,
}

#[derive(Debug)]
pub enum CrabState {
    /// Standing still for a time
    Paused,
    /// Walking in a direction for a time
    Walking {
        /// Are we walking left? If false, we are walking right.
        left: bool,
    },
    /// We are running from something we're scared of
    Running {
        /// What scared the little crabbie? 🦀
        scared_of: Entity,
    },
}

impl CrabState {
    fn is_moving(&self) -> bool {
        matches!(self, CrabState::Walking { .. } | CrabState::Running { .. })
    }
}

impl Default for CrabState {
    fn default() -> Self {
        CrabState::Paused
    }
}

pub fn spawn_crab(world: &mut World, spawn_position: Vec2) -> Result<Entity> {
    let resources = storage::get::<Resources>();
    let texture_res = resources.textures.get(CRAB_TEXTURE_ID).unwrap();
    let size = texture_res.meta.size;
    let actor = storage::get_mut::<CollisionWorld>().add_actor(
        spawn_position,
        size.x as i32,
        size.y as i32,
    );

    let crab_animations = &[Animation {
        id: "idle".to_string(),
        row: 0,
        frames: 2,
        fps: 2,
        tweens: Default::default(),
        is_looping: true,
    }];

    Ok(world.spawn((
        Crab {
            spawn_position,
            state: CrabState::default(),
            state_timer: Timer::new(1.0),
        },
        Transform::from(spawn_position),
        Drawable::new_animated_sprite(
            DRAW_ORDER,
            CRAB_TEXTURE_ID,
            crab_animations,
            Default::default(),
        ),
        PhysicsBody::new(
            actor,
            None,
            PhysicsBodyParams {
                size,
                can_rotate: false,
                gravity: 0.5,
                ..Default::default()
            },
        ),
    )))
}

pub fn update_crabs(world: &mut World) {
    for (_, (crab, drawable, transform, body)) in world
        .query::<(&mut Crab, &mut Drawable, &Transform, &mut PhysicsBody)>()
        .iter()
    {
        let transform: &Transform = transform;
        let drawable: &mut Drawable = drawable;
        let crab: &mut Crab = crab;
        let body: &mut PhysicsBody = body;

        let pos = transform.position;

        let rand_bool = |true_bias: u8| rand::gen_range(0u8, 2 + true_bias) > 0;
        let rand_delay = |min, max| Timer::new(rand::gen_range(min, max));

        let next_scary_thing = || {
            for (scary_entity, transform) in world.query::<With<Player, &Transform>>().iter() {
                let scary_pos = transform.position;

                if (pos - scary_pos).length() < COMFORTABLE_SCARY_DISTANCE // If scary thing is too close
                    && (pos.y - scary_pos.y).abs() < SAME_LEVEL_THRESHOLD
                // and we're on the same level
                {
                    return Some(scary_entity);
                }
            }

            None
        };

        let pick_next_move = || {
            let x_diff = pos.x - crab.spawn_position.x;

            let pause_bias = if crab.state.is_moving() { 2 } else { 0 };
            if rand_bool(pause_bias) {
                (CrabState::Paused, rand_delay(0.2, 0.5))
            } else {
                let left = if (x_diff.abs() > COMFORTABLE_SPAWN_DISTANCE) && rand_bool(2) {
                    x_diff > 0.0
                } else {
                    rand_bool(0)
                };
                (CrabState::Walking { left }, rand_delay(0.05, 0.75))
            }
        };

        crab.state_timer.tick_frame_time();

        // Perform any state transitions
        if crab.state_timer.has_finished() {
            if let Some(scared_of) = next_scary_thing() {
                crab.state = CrabState::Running { scared_of };
                crab.state_timer = rand_delay(0.3, 0.7);
            } else {
                match &crab.state {
                    CrabState::Paused | CrabState::Walking { .. } => {
                        let (state, timer) = pick_next_move();
                        crab.state = state;
                        crab.state_timer = timer;
                    }
                    CrabState::Running { scared_of } => {
                        let scary_pos = world.get::<Transform>(*scared_of).unwrap().position;

                        if (pos - scary_pos).length() > COMFORTABLE_SCARY_DISTANCE {
                            if let Some(scared_of) = next_scary_thing() {
                                crab.state = CrabState::Running { scared_of };
                                crab.state_timer = rand_delay(0.3, 0.7);
                            } else {
                                let (state, timer) = pick_next_move();
                                crab.state = state;
                                crab.state_timer = timer;
                            }
                        }
                    }
                }
            }
        }

        // Apply any component modifications for the current state
        let sprite = drawable.get_animated_sprite_mut().unwrap();
        match &crab.state {
            CrabState::Paused => {
                sprite.is_playing = true;

                body.velocity.x = 0.0;
            }
            CrabState::Walking { left } => {
                sprite.is_flipped_x = *left;
                sprite.is_playing = false;

                let direction = if *left { -1.0 } else { 1.0 };
                let speed = direction * WALK_SPEED;
                body.velocity.x = speed;
            }
            CrabState::Running { scared_of } => {
                sprite.is_playing = true;

                let scary_pos = world.get::<Transform>(*scared_of).unwrap().position;

                let direction = (pos.x - scary_pos.x).signum();
                let speed = direction * RUN_SPEED;
                body.velocity.x = speed;
            }
        }
    }
}
