use std::collections::HashMap;

use macroquad::{
    camera::*,
    experimental::{
        collections::storage,
        scene::{self, RefMut},
    },
    prelude::*,
    telemetry,
};

use macroquad_particles::{Emitter, EmittersCache};

use crate::{error::Result, Resources};

pub struct ParticleEmitters {
    pub hit: EmittersCache,
    pub smoke: Emitter,
    pub explosions: EmittersCache,
    pub explosion_fire: Emitter,
    pub explosion_particles: EmittersCache,
    pub life_ui_explosions: EmittersCache,
    pub other_emitters: HashMap<String, EmittersCache>,
}

impl ParticleEmitters {
    const HIT_EFFECT_ID: &'static str = "hit";
    const SMOKE_EFFECT_ID: &'static str = "smoke";
    const EXPLOSION_EFFECT_ID: &'static str = "explosion";
    const EXPLOSION_FIRE_EFFECT_ID: &'static str = "explosion_fire";
    const EXPLOSION_PARTICLES_EFFECT_ID: &'static str = "explosion_particles";
    const LIFE_UI_EXPLOSION_EFFECT_ID: &'static str = "life_ui_explosion";

    pub async fn new() -> Result<Self> {
        let resources = storage::get::<Resources>();

        let hit = {
            let cfg = resources
                .particle_effects
                .get(Self::HIT_EFFECT_ID)
                .cloned()
                .unwrap();

            EmittersCache::new(cfg)
        };

        let smoke = {
            let cfg = resources
                .particle_effects
                .get(Self::SMOKE_EFFECT_ID)
                .cloned()
                .unwrap();

            Emitter::new(cfg)
        };

        let explosions = {
            let cfg = resources
                .particle_effects
                .get(Self::EXPLOSION_EFFECT_ID)
                .cloned()
                .unwrap();

            EmittersCache::new(cfg)
        };

        let explosion_fire = {
            let cfg = resources
                .particle_effects
                .get(Self::EXPLOSION_FIRE_EFFECT_ID)
                .cloned()
                .unwrap();

            Emitter::new(cfg)
        };

        let explosion_particles = {
            let cfg = resources
                .particle_effects
                .get(Self::EXPLOSION_PARTICLES_EFFECT_ID)
                .cloned()
                .unwrap();

            EmittersCache::new(cfg)
        };

        let life_ui_explosions = {
            let cfg = resources
                .particle_effects
                .get(Self::LIFE_UI_EXPLOSION_EFFECT_ID)
                .cloned()
                .unwrap();

            EmittersCache::new(cfg)
        };

        Ok(ParticleEmitters {
            hit,
            smoke,
            explosions,
            explosion_fire,
            explosion_particles,
            life_ui_explosions,
            other_emitters: HashMap::new(),
        })
    }
}

impl scene::Node for ParticleEmitters {
    fn draw(mut node: RefMut<Self>) {
        let _z = telemetry::ZoneGuard::new("draw particles");

        node.smoke.draw(Vec2::new(0.0, 0.0));
        node.hit.draw();
        node.explosions.draw();
        node.explosion_fire.draw(Vec2::new(0.0, 0.0));
        node.explosion_particles.draw();

        for emitter in node.other_emitters.values_mut() {
            emitter.draw();
        }

        push_camera_state();
        set_default_camera();
        node.life_ui_explosions.draw();
        pop_camera_state();
    }
}
