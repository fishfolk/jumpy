use macroquad::window::{screen_height, screen_width};

use crate::viewport::Viewport;

pub fn get_viewport() -> Viewport {
    Viewport {
        width: screen_width(),
        height: screen_height(),
    }
}