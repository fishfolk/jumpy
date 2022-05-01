use std::collections::HashMap;

use crate::camera::set_main_camera;
use glam::Vec2;
use hecs::Entity;

use crate::math::Size;
use crate::rendering::RenderTarget;
use crate::viewport::Viewport;
use crate::window::window_size;

pub struct CameraImpl {
    pub render_target: Option<RenderTarget>,
    pub position: Vec2,
    pub rotation: f32,
    pub zoom: Vec2,
    pub bounds: Size<f32>,
}

impl CameraImpl {
    pub fn new<P, B>(position: P, bounds: B) -> Self
    where
        P: Into<Option<Vec2>>,
        B: Into<Option<Size<f32>>>,
    {
        let position = position.into().unwrap_or(Vec2::ZERO);
        let bounds = bounds.into().unwrap_or_else(|| window_size());

        CameraImpl {
            render_target: None,
            position,
            zoom: Vec2::ONE,
            rotation: 0.0,
            bounds,
        }
    }

    pub fn viewport(&self) -> Viewport {
        (self.position, self.bounds).into()
    }
}

impl Default for CameraImpl {
    fn default() -> Self {
        CameraImpl {
            render_target: None,
            position: Vec2::ZERO,
            zoom: Vec2::ONE,
            rotation: 0.0,
            bounds: window_size(),
        }
    }
}

pub fn camera_position() -> Vec2 {
    crate::camera::main_camera().position
}
