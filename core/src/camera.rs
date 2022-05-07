use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

pub use crate::backend_impl::camera::*;

use crate::math::{vec2, vec3, Mat4, Size, Vec2};
use crate::render::RenderTarget;
use crate::viewport::Viewport;
use crate::window::window_size;

static mut NEXT_CAMERA_INDEX: usize = 0;

fn camera_index() -> usize {
    unsafe {
        let index = NEXT_CAMERA_INDEX;
        NEXT_CAMERA_INDEX += 1;
        index
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Camera(usize);

impl Camera {
    pub fn new<P, Z, R>(position: P, zoom: Z, render_target: R) -> Self
    where
        P: Into<Option<Vec2>>,
        Z: Into<Option<f32>>,
        R: Into<Option<RenderTarget>>,
    {
        let id = unsafe { camera_index() };
        let position = position.into().unwrap_or(Vec2::ZERO);
        let zoom = zoom.into().unwrap_or(1.0);

        cameras().insert(id, CameraImpl::new(position, zoom, render_target));

        let camera = Camera(id);

        if !is_main_camera_set() {
            set_main_camera(camera);
        }

        camera
    }

    pub fn destroy(self) {
        if is_main_camera_set() && main_camera().0 == self.0 {
            unsafe { CAMERA = None };
        }

        cameras().remove(&self.0);
    }
}

impl Deref for Camera {
    type Target = CameraImpl;

    fn deref(&self) -> &Self::Target {
        cameras().get(&self.0).unwrap()
    }
}

impl DerefMut for Camera {
    fn deref_mut(&mut self) -> &mut Self::Target {
        cameras().get_mut(&self.0).unwrap()
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(None, None, RenderTarget::Context)
    }
}

static mut CAMERAS: Option<HashMap<usize, CameraImpl>> = None;

pub fn cameras() -> &'static mut HashMap<usize, CameraImpl> {
    unsafe { CAMERAS.get_or_insert_with(HashMap::new) }
}

static mut CAMERA: Option<usize> = None;

pub fn main_camera() -> Camera {
    let id = unsafe {
        CAMERA.get_or_insert_with(|| {
            let mut camera = Camera::new(None, None, RenderTarget::Context);
            camera.0
        })
    };
    Camera(*id)
}

pub fn is_main_camera_set() -> bool {
    unsafe { CAMERA.is_some() }
}

pub fn set_main_camera<C: Into<Option<Camera>>>(camera: C) {
    unsafe {
        if let Some(camera) = camera.into() {
            CAMERA = Some(camera.0);
        } else {
            CAMERA = None;
        }
    }
}

pub fn camera_position() -> Vec2 {
    main_camera().position
}
