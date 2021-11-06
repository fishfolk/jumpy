use macroquad::prelude::*;

use crate::ParticleEmitters;

use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ParticleControllerParams {
    /// Each `ParticleController` of one weapon should have a unique id to work properly.
    pub id: String,
    pub particle_id: String,
    /// Delay before `ParticleController` starts to emit particles.
    #[serde(default)]
    pub start_delay: f32,
    /// If true, `ParticleController` will be reset when player use weapon.
    #[serde(default)]
    pub is_can_be_interrupted: bool,
    /// Amount of particles that will be emitted.
    pub amount: u32,
    /// The interval between each particle emit.
    pub interval: f32,
    /// If true, after finishing `ParticleController` will be reset and restarted automatically.
    #[serde(default)]
    pub is_looped: bool,
}

pub struct ParticleController {
    pub params: ParticleControllerParams,

    timer: f32,
    particles_emitted: u32,
    is_emitting_started: bool,
    is_waiting_for_reset: bool,
}

impl ParticleController {
    pub fn new(params: ParticleControllerParams) -> Self {
        Self {params, timer: 0.0, particles_emitted: 0, is_emitting_started: false, is_waiting_for_reset: false}
    }

    fn reset(&mut self) {
        self.is_emitting_started = false;
        self.timer = 0.0;
        self.particles_emitted = 0;
        self.is_waiting_for_reset = false;
    }

    pub fn update(&mut self, position: Vec2, interrupt: bool) {
        if self.is_waiting_for_reset {
            if interrupt {
                self.reset();
            } else {
                return;
            }
        }

        if self.params.is_can_be_interrupted && interrupt {
            self.reset();
        }

        self.timer += get_frame_time();

        if self.is_emitting_started {
            if self.timer >= self.params.interval {
                self.timer = 0.0;
                self.particles_emitted += 1;

                {
                    let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
                    particles.spawn(&self.params.particle_id, position);
                }

                if self.particles_emitted == self.params.amount {
                    if !self.params.is_looped {
                        self.is_waiting_for_reset = true;
                    } else {
                        self.reset();
                    }
                }
            }
        } else {
            //
            if self.timer >= self.params.start_delay {
                self.is_emitting_started = true;
                self.timer = self.params.interval;
            }
        }
    }
}
