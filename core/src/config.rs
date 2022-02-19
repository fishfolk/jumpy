use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::input::mapping::InputMapping;
use crate::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub input: InputMapping,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        let mut res = if path.exists() {
            let bytes = fs::read(path)?;
            toml::from_slice(&bytes)?
        } else {
            Config::default()
        };

        res.input.verify()?;

        Ok(res)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    #[serde(default, rename = "fullscreen")]
    pub is_fullscreen: bool,
    #[serde(default, rename = "high-dpi")]
    pub is_high_dpi: bool,
}

impl Default for WindowConfig {
    fn default() -> Self {
        WindowConfig {
            width: 955,
            height: 600,
            is_fullscreen: false,
            is_high_dpi: false,
        }
    }
}
