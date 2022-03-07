use crate::math::Size;
use crate::viewport::Viewport;

pub fn get_viewport() -> Viewport {
    Viewport {
        width: 800.0,
        height: 600.0,
    }
}