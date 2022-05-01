pub use crate::backend_impl::camera::*;
use crate::math::{Mat4, Size, Vec2};
use crate::prelude::{window_size, RenderTarget};
use glam::vec3;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

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
    pub fn new<P, B>(position: P, bounds: B) -> Self
    where
        P: Into<Option<Vec2>>,
        B: Into<Option<Size<f32>>>,
    {
        let id = unsafe { camera_index() };

        cameras().insert(id, CameraImpl::new(position, bounds));

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

    pub fn projection(&self) -> Mat4 {
        let origin = Mat4::from_translation(vec3(-self.position.x, -self.position.y, 0.0));
        let rotation = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), self.rotation.to_radians());
        let scale = Mat4::from_scale(vec3(self.zoom.x, self.zoom.y, 1.0));
        let offset = Mat4::from_translation(vec3(-0.0, -0.0, 0.0));

        offset * ((scale * rotation) * origin)
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

static mut CAMERAS: Option<HashMap<usize, CameraImpl>> = None;

pub fn cameras() -> &'static mut HashMap<usize, CameraImpl> {
    unsafe { CAMERAS.get_or_insert_with(HashMap::new) }
}

static mut CAMERA: Option<usize> = None;

pub fn main_camera() -> Camera {
    let id = unsafe {
        CAMERA.get_or_insert_with(|| {
            let mut camera = Camera::new(None, None);
            camera.render_target = Some(RenderTarget::Context);
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
