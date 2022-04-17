use crate::camera::active_camera;
use crate::math::Size;
use crate::viewport::Viewport;

pub fn viewport() -> Viewport {
    active_camera().viewport()
}
