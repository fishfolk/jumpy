use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

use crate::math::{vec3, Mat4, Size, Vec2};
use crate::render::RenderTarget;
use crate::window::window_size;

static mut NEXT_CAMERA_INDEX: usize = 0;

fn camera_index() -> usize {
    unsafe {
        let index = NEXT_CAMERA_INDEX;
        NEXT_CAMERA_INDEX += 1;
        index
    }
}

pub struct CameraImpl {
    pub target: Vec2,
    pub zoom: Vec2,
    pub bounds: Size<f32>,
    pub rotation: f32,
    pub offset: Vec2,
    pub render_target: Option<RenderTarget>,
}

impl CameraImpl {
    pub fn new<P, B, Z, R>(position: P, bounds: B, zoom: Z, render_target: R) -> CameraImpl
    where
        P: Into<Option<Vec2>>,
        B: Into<Option<Size<f32>>>,
        Z: Into<Option<Vec2>>,
        R: Into<Option<RenderTarget>>,
    {
        let position = position.into().unwrap_or(Vec2::ZERO);
        let zoom = zoom.into().unwrap_or(Vec2::ONE);
        let bounds = bounds.into().unwrap_or_else(window_size);
        let render_target = render_target.into();

        CameraImpl {
            target: position,
            zoom,
            bounds,
            rotation: 0.0,
            offset: Vec2::ZERO,
            render_target,
        }
    }

    pub fn projection(&self) -> Mat4 {
        let origin = Mat4::from_translation(vec3(-self.target.x, -self.target.y, 0.0));
        let rotation = Mat4::from_axis_angle(vec3(0.0, 0.0, 1.0), self.rotation.to_radians());
        let scale = Mat4::from_scale(vec3(self.zoom.x, self.zoom.y, 1.0));
        let offset = Mat4::from_translation(vec3(-self.offset.x, -self.offset.y, 0.0));

        offset * ((scale * rotation) * origin)
    }
}

impl Default for CameraImpl {
    fn default() -> Self {
        CameraImpl {
            target: Vec2::ZERO,
            zoom: Vec2::ONE,
            bounds: window_size(),
            rotation: 0.0,
            offset: Vec2::ZERO,
            render_target: None,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Camera(usize);

impl Camera {
    pub fn new<P, B, Z, R>(position: P, bounds: B, zoom: Z, render_target: R) -> Self
    where
        P: Into<Option<Vec2>>,
        B: Into<Option<Size<f32>>>,
        Z: Into<Option<Vec2>>,
        R: Into<Option<RenderTarget>>,
    {
        let index = camera_index();

        cameras().insert(
            index,
            CameraImpl::new(position, bounds, zoom, render_target),
        );

        let camera = Camera(index);

        if !is_main_camera_set() {
            set_main_camera(camera);
        }

        camera
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.bounds.width / self.bounds.height
    }

    pub fn world_bounds(&self) -> Size<f32> {
        self.bounds * (1.0 / self.zoom)
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
        Camera::new(None, None, Vec2::ONE, None)
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
            let camera = Camera::default();
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
    main_camera().target
}
