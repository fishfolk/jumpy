use ff_core::ecs::{Entity, World};

use ff_core::prelude::*;

use crate::player::{
    Player, PlayerAttributes, PlayerController, PlayerEventKind, PlayerEventQueue, JUMP_SOUND_ID,
    LAND_SOUND_ID, RESPAWN_DELAY,
};
use crate::{Item, Map, PhysicsBody, PlayerEvent};

const SLIDE_STOP_THRESHOLD: f32 = 2.0;
const JUMP_FRAME_COUNT: u16 = 8;
const PLATFORM_JUMP_FORCE_MULTIPLIER: f32 = 0.2;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlayerState {
    None,
    Jumping,
    Floating,
    Crouching,
    Sliding,
    Incapacitated,
    Dead,
}

impl Default for PlayerState {
    fn default() -> Self {
        PlayerState::None
    }
}

pub fn update_player_states(world: &mut World, delta_time: f32) -> Result<()> {
    let (map_entity, _) = world
        .query_mut::<&Map>()
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("Unable to find map entity!"));

    let mut query = world.query::<(
        &mut Transform,
        &mut Player,
        &PlayerController,
        &PlayerAttributes,
        &mut PhysicsBody,
    )>();

    for (_, (transform, player, controller, attributes, body)) in query.iter() {
        // Timers
        player.attack_timer -= delta_time;

        if player.attack_timer <= 0.0 {
            player.attack_timer = 0.0;
        }

        player.is_attacking = player.attack_timer > 0.0;

        player.pickup_grace_timer += delta_time;

        if player.state == PlayerState::Crouching && !controller.should_crouch {
            player.state = PlayerState::None;
        }

        if player.state == PlayerState::Dead {
            player.respawn_timer += delta_time;

            for effect in &mut player.passive_effects {
                effect.should_end = true;
            }

            if player.respawn_timer >= RESPAWN_DELAY {
                player.state = PlayerState::None;
                player.respawn_timer = 0.0;

                let mut map = world.query_one::<&Map>(map_entity).unwrap();
                transform.position = map.get().unwrap().get_random_spawn_point();
            }
        } else if player.state == PlayerState::Incapacitated {
            player.incapacitation_timer += delta_time;

            if player.incapacitation_timer >= attributes.incapacitation_duration {
                player.state = PlayerState::None;
                player.incapacitation_timer = 0.0;
            }
        }

        if player.state == PlayerState::Sliding && body.velocity.x.abs() <= SLIDE_STOP_THRESHOLD {
            body.velocity.x = 0.0;
            player.state = PlayerState::None;
        }

        // Integration
        if player.is_attacking
            || matches!(
                player.state,
                PlayerState::Dead | PlayerState::Incapacitated | PlayerState::Sliding
            )
        {
            body.has_friction = true;

            player.jump_frame_counter = 0;
            body.has_mass = true;
        } else {
            body.has_friction = false;

            if controller.move_direction.x < 0.0 {
                player.is_facing_left = true;
            } else if controller.move_direction.x > 0.0 {
                player.is_facing_left = false;
            }

            if controller.should_slide {
                let velocity = attributes.slide_speed;

                if player.is_facing_left {
                    body.velocity.x = -velocity;
                } else {
                    body.velocity.x = velocity;
                }

                player.state = PlayerState::Sliding;
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
                        body.velocity.x = 0.0;
                        player.state = PlayerState::Crouching;
                    } else {
                        let mut physics = physics_world();
                        physics.descend(body.actor);
                    }
                }

                if body.is_on_ground && controller.should_jump {
                    let jump_force = if controller.should_crouch && body.is_on_platform {
                        attributes.jump_force * PLATFORM_JUMP_FORCE_MULTIPLIER
                    } else {
                        attributes.jump_force
                    };

                    body.velocity.y = -jump_force;

                    player.state = PlayerState::Jumping;

                    play_sound(JUMP_SOUND_ID, false);
                } else if player.state == PlayerState::Jumping {
                    player.jump_frame_counter += 1;

                    if controller.should_float && player.jump_frame_counter <= JUMP_FRAME_COUNT {
                        body.has_mass = false;
                    } else {
                        if matches!(player.state, PlayerState::Jumping | PlayerState::Floating) {
                            player.state = PlayerState::None;
                        }

                        player.jump_frame_counter = 0;
                        body.has_mass = true;
                    }
                }

                if !body.is_on_ground && body.velocity.y > 0.0 {
                    if controller.should_float {
                        body.velocity.y *= attributes.float_gravity_factor;
                        player.state = PlayerState::Floating;
                    }
                } else if player.state == PlayerState::Floating {
                    player.state = PlayerState::None;
                }
            }

            if body.is_on_ground && !body.was_on_ground {
                if matches!(player.state, PlayerState::Jumping | PlayerState::Floating) {
                    player.state = PlayerState::None;
                }

                player.jump_frame_counter = 0;
                body.has_mass = true;
            }
        }
    }

    Ok(())
}

pub fn update_player_passive_effects(world: &mut World, delta_time: f32) -> Result<()> {
    let mut function_calls = Vec::new();

    for (entity, (player, attributes, events)) in world
        .query::<(&mut Player, &mut PlayerAttributes, &mut PlayerEventQueue)>()
        .iter()
    {
        attributes.clear_mods();

        player
            .passive_effects
            .retain(|effect| !effect.should_remove);

        for effect in &mut player.passive_effects {
            if effect.should_begin {
                if let Some(f) = effect.on_begin_fn {
                    function_calls.push((f, entity, effect.item, None));
                }
            } else {
                effect.duration_timer += delta_time;

                if effect.should_end || effect.is_depleted() {
                    if let Some(f) = effect.on_end_fn {
                        function_calls.push((f, entity, effect.item, None));
                    }

                    effect.should_remove = true;
                } else {
                    attributes.apply_mods(effect);
                }
            }
        }
    }

    for (f, player_entity, item_entity, event) in function_calls {
        f(world, player_entity, item_entity, event);
    }

    Ok(())
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
