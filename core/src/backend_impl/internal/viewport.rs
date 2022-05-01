use crate::camera::main_camera;
use crate::math::Size;
use crate::viewport::Viewport;

pub fn viewport() -> Viewport {
    main_camera().viewport()
}
