pub use crate::backend_impl::viewport::*;

use crate::math::{vec2, Size, UVec2, Vec2};

#[derive(Debug, Clone)]
pub struct Viewport {
    pub position: Vec2,
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }

    pub fn size(&self) -> Size<f32> {
        Size {
            width: self.width,
            height: self.height,
        }
    }
}

impl From<Size<f32>> for Viewport {
    fn from(size: Size<f32>) -> Self {
        Viewport {
            position: Vec2::ZERO,
            width: size.width,
            height: size.height,
        }
    }
}

impl From<(Vec2, Size<f32>)> for Viewport {
    fn from((position, size): (Vec2, Size<f32>)) -> Self {
        Viewport {
            position,
            width: size.width,
            height: size.height,
        }
    }
}

impl From<Vec2> for Viewport {
    fn from(vec: Vec2) -> Self {
        Viewport {
            position: Vec2::ZERO,
            width: vec.x,
            height: vec.y,
        }
    }
}

impl From<(Vec2, Vec2)> for Viewport {
    fn from((position, size): (Vec2, Vec2)) -> Self {
        Viewport {
            position,
            width: size.x,
            height: size.y,
        }
    }
}

pub fn viewport_size() -> Size<f32> {
    viewport().size()
}
