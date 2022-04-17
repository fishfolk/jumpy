use std::collections::HashMap;

use glam::Vec2;
use hecs::Entity;

use crate::math::Size;
use crate::viewport::Viewport;
use crate::window::window_size;

pub struct CameraImpl {
    pub position: Vec2,
    pub bounds: Size<u32>,
}

impl CameraImpl {
    pub fn new<P: Into<Option<Vec2>>>(position: P, bounds: Size<u32>) -> Self {
        let position = position.into().unwrap_or(Vec2::ZERO);

        CameraImpl { position, bounds }
    }

    pub fn viewport(&self) -> Viewport {
        (self.position, self.bounds).into()
    }
}

impl Default for CameraImpl {
    fn default() -> Self {
        CameraImpl {
            position: Vec2::ZERO,
            bounds: window_size(),
        }
    }
}

pub fn camera_position() -> Vec2 {
    crate::camera::active_camera().position
}
