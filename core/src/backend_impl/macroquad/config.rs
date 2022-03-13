use macroquad::miniquad::conf::Icon;
use macroquad::window::Conf;

use crate::Config;
use crate::math::Size;
use crate::video::{RenderingConfig, Resolution};
use crate::video::resolutions::HD720;
use crate::window::{WindowConfig, WindowMode};

const DEFAULT_BORDERLESS_RESOLUTION: Resolution = HD720;

impl Config {
    pub fn as_macroquad_window_conf<I: Into<Option<Icon>>>(&self, title: &str, icon: I, is_resizable_window: bool) -> Conf {
        let (is_fullscreen, width, height) = match self.window.mode {
            WindowMode::Windowed { size: Size { width, height } } => (false, width, height),
            WindowMode::Borderless => (true, DEFAULT_BORDERLESS_RESOLUTION.width, DEFAULT_BORDERLESS_RESOLUTION.height),
        };

        Conf {
            window_title: title.to_string(),
            fullscreen: is_fullscreen,
            high_dpi: self.window.is_high_dpi,
            window_width: width as i32,
            window_height: height as i32,
            window_resizable: is_resizable_window,
            sample_count: self.rendering.msaa_samples.unwrap_or(1) as i32,
            icon: icon.into(),
        }
    }
}