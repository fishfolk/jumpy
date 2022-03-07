use serde::{Serialize, Deserialize};

use crate::math::Vec2;

pub struct EmittersCache {

}

impl EmittersCache {
    pub fn new(config: EmitterConfig) -> Self {
        EmittersCache {}
    }

    pub fn spawn(&mut self, position: Vec2) {
        unimplemented!("Particles not yet implemented")
    }

    pub fn draw(&mut self) {
        unimplemented!("Particles not yet implemented")
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct EmitterConfig {

}