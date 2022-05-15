use crate::math::{vec2, Mat4, Size, Vec2};
use crate::window::window_size;

#[derive(Debug, Copy, Clone)]
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

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::orthographic_rh_gl(0.0, self.width, self.height, 0.0, -1.0, 1.0)
    }
}

impl Default for Viewport {
    fn default() -> Self {
        let window_size = window_size();
        Viewport::new(0.0, 0.0, window_size.width, window_size.height)
    }
}

static mut VIEWPORT: Option<Viewport> = None;

pub fn viewport() -> Viewport {
    *unsafe { VIEWPORT.get_or_insert_with(Viewport::default) }
}

pub(crate) fn viewport_mut() -> &'static mut Viewport {
    unsafe { VIEWPORT.get_or_insert_with(Viewport::default) }
}

pub fn viewport_size() -> Size<f32> {
    viewport().size()
}

pub fn resize_viewport(width: f32, height: f32) {
    let viewport = viewport_mut();
    viewport.width = width;
    viewport.height = height;
}
