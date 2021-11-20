use macroquad::prelude::*;

use crate::components::{AnimationParams, AnimationPlayer};
use serde::{Deserialize, Serialize};

use crate::json::{self, helpers::*};
use crate::math::IsZero;
use crate::ParticleEmitters;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParticleControllerParams {
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
    pub animations: Option<AnimationParams>,
    /// If this is set to `true` the `ParticleController` will start to emit automatically
    #[serde(default, skip_serializing_if = "bool::is_false")]
    pub should_autostart: bool,
}

impl Default for ParticleControllerParams {
    fn default() -> Self {
        ParticleControllerParams {
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

impl From<ParticleControllerParams> for ParticleController {
    fn from(params: ParticleControllerParams) -> Self {
        ParticleController::new(params)
    }
}

#[derive(Clone)]
pub struct ParticleController {
    particle_effect_id: String,
    offset: Vec2,
    delay: f32,
    emissions: Option<u32>,
    interval: f32,
    animations: Option<AnimationPlayer>,
    delay_timer: f32,
    interval_timer: f32,
    emission_cnt: u32,
    is_active: bool,
    position: Option<Vec2>,
}

impl ParticleController {
    const EFFECT_ANIMATION_ID: &'static str = "animated_effect";

    pub fn new(params: ParticleControllerParams) -> Self {
        let mut animations: Option<AnimationPlayer> = params.animations.map(|params| params.into());

        if let Some(animations) = &mut animations {
            animations.set_animation(Self::EFFECT_ANIMATION_ID);
            animations.is_deactivated = !params.should_autostart;

            if params.should_autostart {
                animations.play();
            } else {
                animations.stop();
            }
        }

        ParticleController {
            particle_effect_id: params.particle_effect_id,
            offset: params.offset,
            delay: params.delay,
            interval: params.interval,
            emissions: params.emissions,
            animations,
            delay_timer: 0.0,
            interval_timer: params.interval,
            emission_cnt: 0,
            is_active: params.should_autostart,
            position: None,
        }
    }

    fn get_offset(&self, flip_x: bool, flip_y: bool) -> Vec2 {
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
        if let Some(animations) = &mut self.animations {
            animations.is_deactivated = false;
            animations.play();
        }

        self.delay_timer = 0.0;
        self.interval_timer = self.interval;
        self.emission_cnt = 0;
        self.is_active = true;
    }

    pub fn update(&mut self, dt: f32) {
        if self.is_active {
            self.delay_timer += dt;

            if self.delay_timer >= self.delay {
                self.interval_timer += dt;
            }

            if let Some(position) = self.position {
                if let Some(animations) = &mut self.animations {
                    animations.update();
                }

                if self.delay_timer >= self.delay && self.interval_timer >= self.interval {
                    self.interval_timer = 0.0;

                    {
                        let mut particles = scene::find_node_by_type::<ParticleEmitters>().unwrap();
                        particles.spawn(&self.particle_effect_id, position);
                    }

                    if let Some(emissions) = self.emissions {
                        self.emission_cnt += 1;

                        if emissions > 0 && self.emission_cnt >= emissions {
                            self.is_active = false;

                            if let Some(animations) = &mut self.animations {
                                animations.is_deactivated = true;
                                animations.stop();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn draw(&mut self, position: Vec2, flip_x: bool, flip_y: bool) {
        let position = position + self.get_offset(flip_x, flip_y);

        if let Some(animations) = &mut self.animations {
            animations.draw(position, 0.0, flip_x, flip_y);
        }

        self.position = Some(position);
    }
}
