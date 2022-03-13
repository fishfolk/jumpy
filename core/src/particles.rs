use serde::{Serialize, Deserialize};

use num_traits::*;

use crate::math::Vec2;

pub use crate::backend_impl::particles::*;
use crate::drawables::AnimatedSpriteMetadata;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParticleEmitterMetadata {
    /// The id of the particle effect.
    #[serde(rename = "particle_effect")]
    pub particle_effect_id: String,
    /// The offset is added to the `position` provided when calling `draw`
    #[serde(
    default,
    with = "crate::json::vec2_def"
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
    #[serde(default, skip_serializing_if = "crate::json::is_false")]
    pub should_autostart: bool,
}

impl Default for ParticleEmitterMetadata {
    fn default() -> Self {
        ParticleEmitterMetadata {
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
    pub particle_effect_id: String,
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
    pub fn new(meta: ParticleEmitterMetadata) -> Self {
        ParticleEmitter {
            particle_effect_id: meta.particle_effect_id,
            offset: meta.offset,
            delay: meta.delay,
            interval: meta.interval,
            emissions: meta.emissions,
            emission_cnt: 0,
            delay_timer: 0.0,
            interval_timer: meta.interval,
            is_active: meta.should_autostart,
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

impl From<ParticleEmitterMetadata> for ParticleEmitter {
    fn from(params: ParticleEmitterMetadata) -> Self {
        ParticleEmitter::new(params)
    }
}