pub use crate::backend_impl::viewport::*;

use crate::math::{Vec2, vec2};

#[derive(Debug, Clone)]
pub struct Viewport {
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn aspect_ratio(&self) -> f32 {
        self.width / self.height
    }

    pub fn as_vec2(&self) -> Vec2 {
        vec2(self.width, self.height)
    }
}