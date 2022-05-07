use std::collections::HashMap;

use crate::camera::set_main_camera;
use glam::Vec2;
use hecs::Entity;

use crate::math::Size;
use crate::render::RenderTarget;
use crate::viewport::Viewport;
use crate::window::window_size;

pub struct CameraImpl {
    pub position: Vec2,
    pub zoom: f32,
    pub rotation: f32,
    pub render_target: Option<RenderTarget>,
}

impl CameraImpl {
    pub fn new<P, C, Z, R>(position: P, zoom: Z, render_target: R) -> CameraImpl
    where
        P: Into<Option<Vec2>>,
        Z: Into<Option<f32>>,
        R: Into<Option<RenderTarget>>,
    {
        let position = position.into().unwrap_or(Vec2::ZERO);
        let zoom = zoom.into().unwrap_or(1.0);

        CameraImpl {
            position,
            zoom,
            rotation: 0.0,
            render_target: render_target.into(),
        }
    }

    pub fn projection(&self) -> Mat4 {
        let origin = Mat4::from_translation(vec3(-self.position.x, -self.position.y, 0.0));
        let rotation = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), self.rotation.to_radians());
        let scale = Mat4::from_scale(vec3(self.zoom, self.zoom, 1.0));
        let offset = Mat4::from_translation(vec3(-0.0, -0.0, 0.0));

        offset * ((scale * rotation) * origin)
    }

    pub fn viewport(&self) -> Viewport {
        let size = self.viewport_size();
        Viewport::new(self.position.x, self.position.y, size.width, size.height)
    }

    pub fn viewport_size(&self) -> Size<f32> {
        window_size() * self.zoom
    }
}

impl Default for CameraImpl {
    fn default() -> Self {
        CameraImpl {
            position: Vec2::ZERO,
            zoom: 1.0,
            rotation: 0.0,
            render_target: RenderTarget::Context,
        }
    }
}
