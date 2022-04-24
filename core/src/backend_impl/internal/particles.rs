use serde::{Deserialize, Serialize};

use crate::math::Vec2;

pub struct EmittersCache {}

impl EmittersCache {
    pub fn new(config: EmitterConfig) -> Self {
        EmittersCache {}
    }

    pub fn spawn(&mut self, position: Vec2) {}

    pub fn draw(&mut self) {}
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct EmitterConfig {}
