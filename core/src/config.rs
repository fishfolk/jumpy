use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::slice::{Iter, IterMut};
use std::sync::{Arc, Mutex};
use std::{env, fs};

use serde::{Deserialize, Serialize};

use crate::audio::AudioConfig;
use crate::input::InputMapping;
use crate::parsing::{deserialize_toml_bytes, load_toml_file};
use crate::result::Result;
use crate::video::VideoConfig;
use crate::window::WindowConfig;

pub use crate::backend_impl::config::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub video: VideoConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub input: InputMapping,
}

pub async fn load_config<P: AsRef<Path>>(path: P) -> Result<Config> {
    let mut cfg: Config = load_toml_file(path).await?;
    cfg.input.verify()?;
    Ok(cfg)
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub fn load_config_sync<P: AsRef<Path>>(path: P) -> Result<Config> {
    let bytes = fs::read(path)?;
    let mut cfg: Config = deserialize_toml_bytes(&bytes)?;
    cfg.input.verify()?;
    Ok(cfg)
}
