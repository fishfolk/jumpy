use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::parsing::load_toml_file;
use crate::input::keyboard::InputMapping;
use crate::error::Result;
use crate::video::RenderingConfig;
use crate::window::WindowConfig;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub rendering: RenderingConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub input: InputMapping,
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_config_file_sync<P: AsRef<Path>>(path: P) -> Result<Config> {
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

pub async fn load_config_file<P: AsRef<Path>>(path: P) -> Result<Config> {
    let path = path.as_ref();

    let mut config = if path.exists() {
        load_toml_file(path).await?
    } else {
        Config::default()
    };

    config.input.verify()?;

    Ok(config)
}

