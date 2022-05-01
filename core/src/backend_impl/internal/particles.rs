use serde::{Deserialize, Serialize};

use glam::{vec2, vec3, vec4, Mat4, Vec2, Vec3, Vec4};

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
