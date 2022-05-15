use serde::{Deserialize, Serialize};

use crate::video::Resolution;

pub use crate::backend_impl::window::*;

pub const DEFAULT_WINDOW_TITLE: &str = "Game Window";

const DEFAULT_WINDOW_WIDTH: u32 = 955;
const DEFAULT_WINDOW_HEIGHT: u32 = 600;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default, flatten)]
    pub mode: WindowMode,
    #[serde(
        default,
        rename = "high-dpi",
        skip_serializing_if = "crate::parsing::is_false"
    )]
    pub is_high_dpi: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            mode: WindowMode::Borderless,
            is_high_dpi: false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum WindowMode {
    Windowed {
        #[serde(flatten, default = "WindowMode::default_window_size")]
        size: Resolution,
    },
    Borderless,
    Fullscreen {
        resolution: Resolution,
        bit_depth: u16,
        refresh_rate: u16,
    },
}

impl WindowMode {
    pub fn default_window_size() -> Resolution {
        Resolution::new(DEFAULT_WINDOW_WIDTH, DEFAULT_WINDOW_HEIGHT)
    }
}

impl Default for WindowMode {
    fn default() -> Self {
        WindowMode::Borderless
    }
}
