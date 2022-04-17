use crate::math::Size;
pub use macroquad::miniquad::conf::Icon as WindowIcon;
use macroquad::window::{screen_height, screen_width};

pub fn window_size() -> Size<u32> {
    Size {
        width: screen_width() as u32,
        height: screen_height() as u32,
    }
}
