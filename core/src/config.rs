use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::input::mapping::InputMapping;
use crate::Result;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub fullscreen: bool,
    #[serde(default)]
    pub high_dpi: bool,
    #[serde(default)]
    pub resolution: Resolution,
    #[serde(default)]
    pub input_mapping: InputMapping,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        let mut res = if path.exists() {
            let file_contents = fs::read_to_string(path)?;
            serde_json::from_str(&file_contents)?
        } else {
            Config::default()
        };

        res.input_mapping.verify()?;

        Ok(res)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: i32,
    pub height: i32,
}

impl Default for Resolution {
    fn default() -> Self {
        Resolution {
            width: 955,
            height: 600,
        }
    }
}
