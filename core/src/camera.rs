pub use crate::backend_impl::camera::*;
use crate::math::{Size, Vec2};
use crate::prelude::window_size;
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
    pub fn new<P: Into<Option<Vec2>>>(position: P, bounds: Size<f32>) -> Self {
        let id = unsafe { camera_index() };

        cameras().insert(id, CameraImpl::new(position, bounds));

        let camera = Camera(id);

        if !is_active_camera_set() {
            set_active_camera(camera);
        }

        camera
    }

    pub fn delete(self) {
        if is_active_camera_set() && active_camera().0 == self.0 {
            unsafe { CAMERA = None };
        }

        cameras().remove(&self.0);
    }
}

impl Default for Camera {
    fn default() -> Self {
        Camera::new(None, window_size())
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

fn cameras() -> &'static mut HashMap<usize, CameraImpl> {
    unsafe { CAMERAS.get_or_insert_with(HashMap::new) }
}

static mut CAMERA: Option<usize> = None;

pub fn active_camera() -> Camera {
    let id = unsafe { CAMERA.get_or_insert_with(|| Camera::default().0) };
    Camera(*id)
}

pub fn is_active_camera_set() -> bool {
    unsafe { CAMERA.is_some() }
}

pub fn set_active_camera(camera: Camera) {
    unsafe { CAMERA = Some(camera.0) }
}
