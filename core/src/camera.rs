pub use crate::backend_impl::camera::*;
use crate::math::{Size, Vec2};
use crate::prelude::window_size;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

static mut LAST_CAMERA_ID: usize = 0;

fn camera_id() -> usize {
    unsafe {
        LAST_CAMERA_ID += 1;
        LAST_CAMERA_ID
    }
}

static mut CAMERAS: Option<HashMap<usize, CameraImpl>> = None;

fn cameras() -> &'static mut HashMap<usize, CameraImpl> {
    unsafe { CAMERAS.get_or_insert_with(HashMap::new) }
}

static mut ACTIVE_CAMERA: Option<usize> = None;

pub fn is_active_camera_set() -> bool {
    unsafe { ACTIVE_CAMERA.is_some() }
}

pub fn active_camera() -> Camera {
    let id = unsafe {
        ACTIVE_CAMERA
            .unwrap_or_else(|| panic!("Attempted to get active camera but none has been set!"))
    };

    Camera(id)
}

pub fn set_active_camera(camera: Camera) {
    unsafe { ACTIVE_CAMERA = Some(camera.0) }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Camera(usize);

impl Camera {
    pub fn new<P: Into<Option<Vec2>>>(position: P, bounds: Size<u32>) -> Self {
        let id = unsafe { camera_id() };

        cameras().insert(id, CameraImpl::new(position, bounds));

        let camera = Camera(id);

        if !is_active_camera_set() {
            set_active_camera(camera);
        }

        camera
    }

    pub fn delete(self) {
        if is_active_camera_set() && active_camera().0 == self.0 {
            unsafe { ACTIVE_CAMERA = None };
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
