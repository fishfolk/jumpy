use macroquad::miniquad::conf::Icon;
use macroquad::window::Conf;

use crate::config::Config;
use crate::math::Size;
use crate::video::resolutions::HD720;
use crate::video::{Resolution, VideoConfig};
use crate::window::{WindowConfig, WindowMode};

const DEFAULT_BORDERLESS_RESOLUTION: Resolution = HD720;

static mut CONFIG: Option<Config> = None;

pub fn config() -> &'static Config {
    unsafe { CONFIG.get_or_insert_with(Config::default) }
}

pub fn config_mut() -> &'static mut Config {
    unsafe { CONFIG.get_or_insert_with(Config::default) }
}

pub fn set_config(config: Config) {
    unsafe { CONFIG = Some(config) };
}

impl Config {
    pub fn to_macroquad<I: Into<Option<Icon>>>(
        &self,
        title: &str,
        icon: I,
        is_resizable_window: bool,
    ) -> Conf {
        let (is_fullscreen, width, height) = match self.window.mode {
            WindowMode::Windowed {
                size: Size { width, height },
            } => (false, width, height),
            WindowMode::Borderless => (
                true,
                DEFAULT_BORDERLESS_RESOLUTION.width,
                DEFAULT_BORDERLESS_RESOLUTION.height,
            ),
            WindowMode::Fullscreen { .. } => (
                true,
                DEFAULT_BORDERLESS_RESOLUTION.width,
                DEFAULT_BORDERLESS_RESOLUTION.height,
            ),
        };

        Conf {
            window_title: title.to_string(),
            fullscreen: is_fullscreen,
            high_dpi: self.window.is_high_dpi,
            window_width: width as i32,
            window_height: height as i32,
            window_resizable: is_resizable_window,
            sample_count: self.video.msaa_samples.unwrap_or(1) as i32,
            icon: icon.into(),
        }
    }
}
