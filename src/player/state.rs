use macroquad::audio::play_sound_once;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::{Entity, World};

use crate::player::{
    PlayerAttributes, PlayerController, PlayerEventQueue, JUMP_SOUND_ID, LAND_SOUND_ID,
    RESPAWN_DELAY,
};
use crate::{
    CollisionWorld, GameCamera, Item, Map, PassiveEffectInstance, PhysicsBody, PlayerEvent,
    Resources, Transform,
};

const JUMP_FRAME_COUNT: u16 = 8;

pub struct PlayerState {
    pub camera_box: Rect,
    pub passive_effects: Vec<PassiveEffectInstance>,
    pub is_facing_left: bool,
    pub is_upside_down: bool,
    pub is_jumping: bool,
    pub is_floating: bool,
    pub is_sliding: bool,
    pub is_crouching: bool,
    pub is_attacking: bool,
    pub is_incapacitated: bool,
    pub is_dead: bool,
    pub jump_frame_counter: u16,
    pub pickup_grace_timer: f32,
    pub incapacitation_timer: f32,
    pub attack_timer: f32,
    pub respawn_timer: f32,
}

impl From<Vec2> for PlayerState {
    fn from(position: Vec2) -> Self {
        let camera_box = Rect::new(position.x - 30.0, position.y - 150.0, 100.0, 210.0);

        PlayerState {
            camera_box,
            passive_effects: Vec::new(),
            is_facing_left: false,
            is_upside_down: false,
            is_jumping: false,
            is_floating: false,
            is_sliding: false,
            is_crouching: false,
            is_attacking: false,
            is_incapacitated: false,
            is_dead: false,
            jump_frame_counter: 0,
            pickup_grace_timer: 0.0,
            attack_timer: 0.0,
            incapacitation_timer: 0.0,
            respawn_timer: 0.0,
        }
    }
}

pub fn update_player_camera_box(world: &mut World) {
    for (_, (transform, state)) in world.query_mut::<(&Transform, &mut PlayerState)>() {
        let rect = Rect::new(transform.position.x, transform.position.y, 32.0, 60.0);

        if rect.x < state.camera_box.x {
            state.camera_box.x = rect.x;
        }

        if rect.x + rect.w > state.camera_box.x + state.camera_box.w {
            state.camera_box.x = rect.x + rect.w - state.camera_box.w;
        }

        if rect.y < state.camera_box.y {
            state.camera_box.y = rect.y;
        }

        if rect.y + rect.h > state.camera_box.y + state.camera_box.h {
            state.camera_box.y = rect.y + rect.h - state.camera_box.h;
        }

        let mut camera = storage::get_mut::<GameCamera>();
        camera.add_player_rect(state.camera_box);
    }
}

const SLIDE_STOP_THRESHOLD: f32 = 2.0;
const PLATFORM_JUMP_FORCE_MULTIPLIER: f32 = 0.2;

pub fn update_player_states(world: &mut World) {
    let query = world.query_mut::<(
        &mut Transform,
        &PlayerController,
        &PlayerAttributes,
        &mut PlayerState,
        &mut PhysicsBody,
    )>();
    for (_, (transform, controller, attributes, state, body)) in query {
        // Timers
        let dt = get_frame_time();

        state.attack_timer -= dt;
        if state.attack_timer <= 0.0 {
            state.attack_timer = 0.0;
        }

        state.is_attacking = state.attack_timer > 0.0;

        state.pickup_grace_timer += dt;

        if state.is_dead {
            state.respawn_timer += dt;

            if state.respawn_timer >= RESPAWN_DELAY {
                state.is_dead = false;
                state.respawn_timer = 0.0;

                let map = storage::get::<Map>();
                transform.position = map.get_random_spawn_point();
            }
        } else if state.is_incapacitated {
            state.incapacitation_timer += dt;

            if state.incapacitation_timer >= attributes.incapacitation_duration {
                state.is_incapacitated = false;
                state.incapacitation_timer = 0.0;
            }
        }

        if state.is_sliding && body.velocity.x.abs() <= SLIDE_STOP_THRESHOLD {
            body.velocity.x = 0.0;
            state.is_sliding = false;
        }

        // Integration
        if state.is_dead || state.is_attacking || state.is_incapacitated || state.is_sliding {
            body.has_friction = true;

            if state.is_attacking {
                state.is_jumping = false;
                state.jump_frame_counter = 0;
                body.has_mass = true;
            }
        } else {
            body.has_friction = false;

            if controller.move_direction.x < 0.0 {
                state.is_facing_left = true;
            } else if controller.move_direction.x > 0.0 {
                state.is_facing_left = false;
            }

            state.is_crouching = false;

            if controller.should_slide {
                let velocity = attributes.move_speed * attributes.slide_speed_factor;

                if state.is_facing_left {
                    body.velocity.x = -velocity;
                } else {
                    body.velocity.x = velocity;
                }

                state.is_sliding = true;
            } else {
                if controller.move_direction.x < 0.0 {
                    body.velocity.x = -attributes.move_speed;
                } else if controller.move_direction.x > 0.0 {
                    body.velocity.x = attributes.move_speed;
                } else {
                    body.velocity.x = 0.0;
                }

                if controller.should_crouch {
                    if body.is_on_ground {
                        state.is_crouching = true;
                        body.velocity.x = 0.0;
                    } else {
                        let mut collision_world = storage::get_mut::<CollisionWorld>();
                        collision_world.descent(body.actor);
                    }
                }

                if body.is_on_ground && controller.should_jump {
                    let jump_force = if controller.should_crouch && body.is_on_platform {
                        attributes.jump_force * PLATFORM_JUMP_FORCE_MULTIPLIER
                    } else {
                        attributes.jump_force
                    };

                    body.velocity.y = -jump_force;

                    state.is_jumping = true;

                    let resources = storage::get::<Resources>();
                    let sound = resources.sounds[JUMP_SOUND_ID];

                    play_sound_once(sound);
                } else if state.is_jumping {
                    state.jump_frame_counter += 1;

                    if controller.should_float && state.jump_frame_counter <= JUMP_FRAME_COUNT {
                        body.has_mass = false;
                    } else {
                        state.is_jumping = false;
                        state.jump_frame_counter = 0;
                        body.has_mass = true;
                    }
                }

                if !body.is_on_ground && body.velocity.y > 0.0 {
                    state.is_floating = controller.should_float;

                    if state.is_floating {
                        body.velocity.y *= attributes.float_gravity_factor;
                    }
                } else {
                    state.is_floating = false;
                }
            }

            if body.is_on_ground && !body.was_on_ground {
                state.is_jumping = false;
                state.jump_frame_counter = 0;
                body.has_mass = true;

                let resources = storage::get::<Resources>();
                let sound = resources.sounds[LAND_SOUND_ID];

                play_sound_once(sound);
            }
        }
    }
}

pub fn update_player_passive_effects(world: &mut World) {
    let mut function_calls = Vec::new();

    for (entity, (state, events)) in world
        .query::<(&mut PlayerState, &mut PlayerEventQueue)>()
        .iter()
    {
        let dt = get_frame_time();

        for effect in &mut state.passive_effects {
            effect.duration_timer += dt;
        }

        state.passive_effects.retain(|effect| !effect.is_depleted());

        events.queue.push(PlayerEvent::Update { dt });

        for event in events.queue.iter() {
            let kind = event.into();

            for effect in &mut state.passive_effects {
                if effect.activated_on.contains(&kind) {
                    effect.use_cnt += 1;

                    if let Some(item_entity) = effect.item {
                        let mut item = world.get_mut::<Item>(item_entity).unwrap();

                        item.use_cnt += 1;
                    }

                    if let Some(f) = &effect.function {
                        function_calls.push((*f, entity, effect.item, event.clone()));
                    }
                }
            }
        }
    }

    for (f, player_entity, item_entity, event) in function_calls.drain(0..) {
        f(world, player_entity, item_entity, event);
    }
}

pub fn on_player_damage(world: &mut World, damage_from_entity: Entity, damage_to_entity: Entity) {
    let mut is_from_left = false;

    if let Ok(owner_transform) = world.get::<Transform>(damage_from_entity) {
        if let Ok(target_transform) = world.get::<Transform>(damage_to_entity) {
            is_from_left = owner_transform.position.x < target_transform.position.x;
        }
    }

    {
        let mut events = world
            .get_mut::<PlayerEventQueue>(damage_from_entity)
            .unwrap();

        events.queue.push(PlayerEvent::GiveDamage {
            damage_to: Some(damage_to_entity),
        });
    }

    {
        let mut events = world.get_mut::<PlayerEventQueue>(damage_to_entity).unwrap();

        events.queue.push(PlayerEvent::ReceiveDamage {
            is_from_left,
            damage_from: Some(damage_from_entity),
        });
    }
}
