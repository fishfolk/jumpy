use std::collections::HashMap;

use hecs::Entity;

use crate::camera::set_main_camera;

use crate::math::{vec3, Mat4, Size, Vec2};
use crate::render::RenderTarget;
use crate::viewport::Viewport;
use crate::window::window_size;

pub struct CameraImpl {
    pub position: Vec2,
    pub zoom: f32,
    pub bounds: Size<f32>,
    pub rotation: f32,
    pub render_target: RenderTarget,
}

impl CameraImpl {
    pub fn new<P, B, Z, R>(position: P, bounds: B, zoom: Z, render_target: R) -> CameraImpl
    where
        P: Into<Option<Vec2>>,
        B: Into<Option<Size<f32>>>,
        Z: Into<Option<f32>>,
        R: Into<Option<RenderTarget>>,
    {
        let position = position.into().unwrap_or(Vec2::ZERO);
        let zoom = zoom.into().unwrap_or(1.0);
        let bounds = bounds.into().unwrap_or_else(|| window_size());
        let render_target = render_target.into().unwrap_or_default();

        CameraImpl {
            position,
            zoom,
            bounds,
            rotation: 0.0,
            render_target,
        }
    }

    pub fn projection(&self) -> Mat4 {
        let origin = Mat4::from_translation(vec3(-self.position.x, -self.position.y, 0.0));
        let rotation = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), self.rotation.to_radians());
        let scale = Mat4::from_scale(vec3(self.zoom, self.zoom, 1.0));
        let offset = Mat4::from_translation(vec3(-0.0, -0.0, 0.0));

        offset * ((scale * rotation) * origin)
    }
}

impl Default for CameraImpl {
    fn default() -> Self {
        CameraImpl {
            position: Vec2::ZERO,
            zoom: 1.0,
            bounds: window_size(),
            rotation: 0.0,
            render_target: RenderTarget::default(),
        }
    }
}
