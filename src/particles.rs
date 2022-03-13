use std::collections::HashMap;

use core::particles::EmittersCache;

use hecs::World;

use serde::{Deserialize, Serialize};

use core::prelude::*;

use crate::AnimatedSpriteMetadata;

pub fn update_one_particle_emitter(
    delta_time: f32,
    mut position: Vec2,
    rotation: f32,
    emitter: &mut ParticleEmitter,
) {
    if emitter.is_active {
        emitter.delay_timer += delta_time;

        if emitter.delay_timer >= emitter.delay {
            emitter.interval_timer += delta_time;
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

            let mut particles = storage::get_mut::<Particles>();
            let cache = particles
                .cache_map
                .get_mut(&emitter.particle_effect_id)
                .unwrap();

            cache.spawn(position);

            if let Some(emissions) = emitter.emissions {
                emitter.emission_cnt += 1;

                if emissions > 0 && emitter.emission_cnt >= emissions {
                    emitter.is_active = false;
                }
            }
        }
    }
}

pub fn update_particle_emitters(world: &mut World, delta_time: f32) {
    for (_, (transform, emitter)) in world.query_mut::<(&Transform, &mut ParticleEmitter)>() {
        update_one_particle_emitter(delta_time, transform.position, transform.rotation, emitter);
    }

    for (_, (transform, emitters)) in world.query_mut::<(&Transform, &mut Vec<ParticleEmitter>)>() {
        for emitter in emitters.iter_mut() {
            update_one_particle_emitter(delta_time, transform.position, transform.rotation, emitter);
        }
    }
}

pub fn draw_particles(_world: &mut World) {
    let mut particles = storage::get_mut::<Particles>();

    for cache in particles.cache_map.values_mut() {
        cache.draw();
    }
}

#[derive(Default)]
pub struct Particles {
    pub cache_map: HashMap<String, EmittersCache>,
}

impl Particles {
    pub fn new() -> Self {
        let mut cache_map = HashMap::new();

        for (id, config) in iter_particle_effects() {
            cache_map.insert(id.clone(), EmittersCache::new(config.clone()));
        }

        Particles { cache_map }
    }
}
