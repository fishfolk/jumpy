use macroquad::audio::play_sound_once;
use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use hecs::{Entity, World};

use crate::{
    AnimatedSprite, Animation, PhysicsBody, QueuedAnimationAction, Resources, Result, Transform,
};

const TEXTURE_ID: &str = "sproinger";

const IDLE_ANIMATION_ID: &str = "idle";
const EXPAND_ANIMATION_ID: &str = "expand";
const CONTRACT_ANIMATION_ID: &str = "contract";

const SOUND_EFFECT_ID: &str = "jump";

const COOLDOWN: f32 = 0.75;

const TRIGGER_WIDTH: f32 = 32.0;
const TRIGGER_HEIGHT: f32 = 8.0;

const FORCE: f32 = 35.0;

#[derive(Default)]
pub struct Sproinger {
    pub cooldown_timer: f32,
}

impl Sproinger {
    pub fn new() -> Self {
        Sproinger {
            cooldown_timer: COOLDOWN,
        }
    }
}

pub fn spawn_sproinger(world: &mut World, position: Vec2) -> Result<Entity> {
    let texture_res = {
        let resources = storage::get::<Resources>();
        resources.textures.get(TEXTURE_ID).cloned().unwrap()
    };

    let frame_size = texture_res.frame_size();

    let animations = &[
        Animation {
            id: IDLE_ANIMATION_ID.to_string(),
            row: 0,
            frames: 1,
            fps: 1,
            is_looping: false,
        },
        Animation {
            id: EXPAND_ANIMATION_ID.to_string(),
            row: 1,
            frames: 2,
            fps: 8,
            is_looping: false,
        },
        Animation {
            id: CONTRACT_ANIMATION_ID.to_string(),
            row: 2,
            frames: 2,
            fps: 4,
            is_looping: false,
        },
    ];

    let entity = world.spawn((
        Sproinger::new(),
        Transform::from(position),
        AnimatedSprite::new(
            texture_res.texture,
            frame_size,
            animations,
            Default::default(),
        ),
    ));

    Ok(entity)
}

pub fn update_sproingers(world: &mut World) {
    let dt = get_frame_time();

    let bodies = world
        .query::<(&Transform, &PhysicsBody)>()
        .iter()
        .filter_map(|(e, (transform, body))| {
            if !body.is_deactivated {
                Some((e, body.as_rect(transform.position)))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mut to_be_sproinged = Vec::new();

    'sproingers: for (_, (sproinger, transform, sprite)) in
        world.query_mut::<(&mut Sproinger, &Transform, &mut AnimatedSprite)>()
    {
        sproinger.cooldown_timer += dt;

        let sound = {
            let resources = storage::get::<Resources>();
            resources.sounds[SOUND_EFFECT_ID]
        };

        if sproinger.cooldown_timer >= COOLDOWN {
            sprite.set_animation(IDLE_ANIMATION_ID, true);

            let position = transform.position - (sprite.frame_size / 2.0);

            let trigger_rect = Rect::new(position.x, position.y, TRIGGER_WIDTH, TRIGGER_HEIGHT);

            for (e, rect) in &bodies {
                if trigger_rect.overlaps(rect) {
                    to_be_sproinged.push(*e);

                    sproinger.cooldown_timer = 0.0;

                    sprite.set_animation(EXPAND_ANIMATION_ID, true);
                    sprite.queue_action(QueuedAnimationAction::Play(
                        CONTRACT_ANIMATION_ID.to_string(),
                    ));

                    play_sound_once(sound);

                    continue 'sproingers;
                }
            }
        }
    }

    for entity in to_be_sproinged {
        if let Ok(mut body) = world.get_mut::<PhysicsBody>(entity) {
            body.velocity.y = -FORCE;
        }
    }
}
