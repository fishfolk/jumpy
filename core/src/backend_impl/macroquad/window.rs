use crate::math::Size;
pub use macroquad::miniquad::conf::Icon as WindowIcon;
use macroquad::window::{screen_height, screen_width};

pub fn window_size() -> Size<f32> {
    Size {
        width: screen_width(),
        height: screen_height(),
    }
}
