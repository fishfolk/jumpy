use std::collections::HashMap;

use macroquad::{
    experimental::{
        collections::storage,
        scene::{self, RefMut},
    },
    prelude::*,
    telemetry,
};

use macroquad_particles::EmittersCache;

use crate::Resources;

pub struct ParticleEmitters {
    pub emitters: HashMap<String, EmittersCache>,
}

impl ParticleEmitters {
    pub fn new() -> Self {
        let resources = storage::get::<Resources>();

        let mut emitters = HashMap::new();

        for (id, cfg) in resources.particle_effects.clone() {
            let emitter = EmittersCache::new(cfg);
            emitters.insert(id, emitter);
        }

        ParticleEmitters { emitters }
    }

    pub fn spawn(&mut self, id: &str, position: Vec2) {
        let emitter = self.emitters.get_mut(id).unwrap();
        emitter.spawn(position);
    }
}

impl scene::Node for ParticleEmitters {
    fn draw(mut node: RefMut<Self>) {
        let _z = telemetry::ZoneGuard::new("draw particles");

        for emitter in node.emitters.values_mut() {
            emitter.draw();
        }
    }
}
