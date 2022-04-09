use cfg_if::cfg_if;

use serde::{Serialize, Deserialize};

use crate::Result;
use crate::video::Resolution;

pub use crate::backend_impl::window::*;

const DEFAULT_WINDOW_WIDTH: u32 = 955;
const DEFAULT_WINDOW_HEIGHT: u32 = 600;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    #[serde(default, flatten)]
    pub mode: WindowMode,
    #[serde(default, rename = "high-dpi", skip_serializing_if = "crate::parsing::is_false")]
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
    /*
    Fullscreen {
        #[serde(default, skip_serializing_if = "Option::is_empty")]
        resolution: Option<Resolution>,
        #[serde(default, skip_serializing_if = "Option::is_empty")]
        bit_depth: Option<u16>,
        #[serde(default, skip_serializing_if = "Option::is_empty")]
        refresh_rate: Option<u16>,
    },
    */
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

pub fn default_window_icon() -> Option<Icon> {
    None
}