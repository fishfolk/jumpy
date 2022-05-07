use crate::camera::main_camera;

use crate::math::{vec2, Size, UVec2, Vec2};

#[derive(Debug, Clone)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Viewport {
            x,
            y,
            width,
            height,
        }
    }

    pub fn position(&self) -> Vec2 {
        vec2(self.x, self.y)
    }

    pub fn size(&self) -> Size<f32> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

impl From<Size<f32>> for Viewport {
    fn from(size: Size<f32>) -> Self {
        Viewport {
            x: 0.0,
            y: 0.0,
            width: size.width,
            height: size.height,
        }
    }
}

impl From<(Vec2, Size<f32>)> for Viewport {
    fn from((position, size): (Vec2, Size<f32>)) -> Self {
        Viewport {
            x: position.x,
            y: position.y,
            width: size.width,
            height: size.height,
        }
    }
}

impl From<(Vec2, Vec2)> for Viewport {
    fn from((position, size): (Vec2, Vec2)) -> Self {
        Viewport {
            x: position.x,
            y: position.y,
            width: size.x,
            height: size.y,
        }
    }
}

pub fn viewport() -> Viewport {
    main_camera().viewport()
}

pub fn viewport_size() -> Size<f32> {
    main_camera().viewport_size()
}
