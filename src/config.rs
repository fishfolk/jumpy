use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use core::Error;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub fullscreen: bool,
    pub high_dpi: bool,
    pub resolution: Resolution,
}

impl Config {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();

        let res = if path.exists() {
            let file_contents = fs::read_to_string(path)?;
            serde_json::from_str(&file_contents)?
        } else {
            Config::default()
        };

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
