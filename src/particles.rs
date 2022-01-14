use macroquad::experimental::collections::storage;
use macroquad::prelude::*;

use ff_particles::EmittersCache;

use hecs::World;

use serde::{Deserialize, Serialize};

use crate::json;
use crate::math::IsZero;
use crate::{AnimatedSpriteMetadata, Resources, Transform};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParticleEmitterParams {
    /// The id of the particle effect.
    #[serde(rename = "particle_effect")]
    pub particle_effect_id: String,
    /// The offset is added to the `position` provided when calling `draw`
    #[serde(
        default,
        with = "json::vec2_def",
        skip_serializing_if = "Vec2::is_zero"
    )]
    pub offset: Vec2,
    /// Delay before emission will begin
    #[serde(default, skip_serializing_if = "f32::is_zero")]
    pub delay: f32,
    /// The interval between each emission.
    #[serde(default, skip_serializing_if = "f32::is_zero")]
    pub interval: f32,
    /// Amount of emissions per activation. If set to `None` it will emit indefinitely
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emissions: Option<u32>,
    /// This is a temporary hack that enables texture based effects until we add texture support
    /// to our macroquad-particles fork
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub animations: Option<AnimatedSpriteMetadata>,
    /// If this is set to `true` the `ParticleController` will start to emit automatically
    #[serde(default, skip_serializing_if = "json::is_false")]
    pub should_autostart: bool,
}

impl Default for ParticleEmitterParams {
    fn default() -> Self {
        ParticleEmitterParams {
            particle_effect_id: "".to_string(),
            offset: Vec2::ZERO,
            delay: 0.0,
            emissions: None,
            interval: 0.0,
            animations: None,
            should_autostart: false,
        }
    }
}

pub struct ParticleEmitter {
    pub cache: EmittersCache,
    pub offset: Vec2,
    pub delay: f32,
    pub emissions: Option<u32>,
    pub interval: f32,
    pub emission_cnt: u32,
    pub delay_timer: f32,
    pub interval_timer: f32,
    pub is_active: bool,
}

impl ParticleEmitter {
    pub fn new(params: ParticleEmitterParams) -> Self {
        let resources = storage::get::<Resources>();
        let config = resources
            .particle_effects
            .get(&params.particle_effect_id)
            .cloned()
            .unwrap();

        ParticleEmitter {
            cache: EmittersCache::new(config),
            offset: params.offset,
            delay: params.delay,
            interval: params.interval,
            emissions: params.emissions,
            emission_cnt: 0,
            delay_timer: 0.0,
            interval_timer: params.interval,
            is_active: params.should_autostart,
        }
    }

    pub fn get_offset(&self, flip_x: bool, flip_y: bool) -> Vec2 {
        let mut offset = self.offset;

        if flip_x {
            offset.x = -offset.x;
        }

        if flip_y {
            offset.y = -offset.y;
        }

        offset
    }

    pub fn activate(&mut self) {
        self.delay_timer = 0.0;
        self.interval_timer = self.interval;
        self.emission_cnt = 0;
        self.is_active = true;
    }
}

impl From<ParticleEmitterParams> for ParticleEmitter {
    fn from(params: ParticleEmitterParams) -> Self {
        ParticleEmitter::new(params)
    }
}

pub fn update_one_particle_emitter(
    mut position: Vec2,
    rotation: f32,
    emitter: &mut ParticleEmitter,
) {
    let dt = get_frame_time();

    if emitter.is_active {
        emitter.delay_timer += dt;

        if emitter.delay_timer >= emitter.delay {
            emitter.interval_timer += dt;
        }

        if emitter.delay_timer >= emitter.delay && emitter.interval_timer >= emitter.interval {
            emitter.interval_timer = 0.0;

            if rotation == 0.0 {
                position += emitter.offset;
            } else {
                let offset_position = position + emitter.offset;

                let sin = rotation.sin();
                let cos = rotation.cos();

                position = Vec2::new(
                    cos * (offset_position.x - position.x) - sin * (offset_position.y - position.y)
                        + position.x,
                    sin * (offset_position.x - position.x)
                        + cos * (offset_position.y - position.y)
                        + position.y,
                );
            }

            emitter.cache.spawn(position);

            if let Some(emissions) = emitter.emissions {
                emitter.emission_cnt += 1;

                if emissions > 0 && emitter.emission_cnt >= emissions {
                    emitter.is_active = false;
                }
            }
        }
    }
}

pub fn draw_one_particle_emitter(emitter: &mut ParticleEmitter) {
    emitter.cache.draw();
}

pub fn update_particle_emitters(world: &mut World) {
    for (_, (transform, emitter)) in world.query_mut::<(&Transform, &mut ParticleEmitter)>() {
        update_one_particle_emitter(transform.position, transform.rotation, emitter);
    }
}

pub fn draw_particle_emitters(world: &mut World) {
    for (_, emitter) in world.query_mut::<&mut ParticleEmitter>() {
        draw_one_particle_emitter(emitter);
    }
}

pub fn update_particle_emitter_sets(world: &mut World) {
    for (_, (transform, emitters)) in world.query_mut::<(&Transform, &mut Vec<ParticleEmitter>)>() {
        for emitter in emitters.iter_mut() {
            update_one_particle_emitter(transform.position, transform.rotation, emitter);
        }
    }
}

pub fn draw_particle_emitter_sets(world: &mut World) {
    for (_, emitters) in world.query_mut::<&mut Vec<ParticleEmitter>>() {
        for emitter in emitters.iter_mut() {
            draw_one_particle_emitter(emitter);
        }
    }
}
