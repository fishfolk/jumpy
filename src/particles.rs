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

#[derive(Default)]
pub struct ParticleEmitters {
    pub emitters: HashMap<String, EmittersCache>,
}

impl ParticleEmitters {
    pub fn new() -> Self {
        ParticleEmitters {
            emitters: HashMap::new(),
        }
    }

    pub fn spawn(&mut self, id: &str, position: Vec2) {
        if !self.emitters.contains_key(id) {
            let resources = storage::get::<Resources>();
            let cfg = resources.particle_effects.get(id).cloned().unwrap();
            self.emitters
                .insert(id.to_string(), EmittersCache::new(cfg));
        }

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
