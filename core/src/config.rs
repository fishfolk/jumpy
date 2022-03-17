use std::{env, fs};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use crate::audio::AudioConfig;

use crate::parsing::{deserialize_toml_bytes, load_toml_file};
use crate::input::InputMapping;
use crate::error::Result;
use crate::video::RenderingConfig;
use crate::window::WindowConfig;

pub use crate::backend_impl::config::*;

const DEFAULT_CONFIG_PATH: &str = "config.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub rendering: RenderingConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub audio: AudioConfig,
    #[serde(default)]
    pub input: InputMapping,
}

static mut CONFIG: Option<Config> = None;

pub fn set_config(config: Config) {
    unsafe { CONFIG = Some(config) }
}

pub async fn load_config<P: AsRef<Path>>(path: P) -> Result<&'static Config> {
    let mut config: Config = load_toml_file(path).await?;
    config.input.verify()?;
    set_config(config);
    Ok(get_config())
}

pub async fn load_config_or_default<P: AsRef<Path>>(path: P) -> Result<&'static Config> {
    let mut config: Config = load_toml_file(path).await.unwrap_or_else(|err| {
        #[cfg(debug_assertions)]
        println!("WARNING: Unable to load config; using defaults: {:?}", err);
        Config::default()
    });

    config.input.verify()?;
    set_config(config);
    Ok(get_config())
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub fn load_config_sync<P: AsRef<Path>>(path: P) -> Result<&'static Config> {
    let bytes = fs::read(path)?;
    let mut config: Config = deserialize_toml_bytes(&bytes)?;
    config.input.verify()?;
    set_config(config);
    Ok(get_config())
}

#[cfg(not(any(target_os = "android", target_arch = "wasm32")))]
pub fn load_config_or_default_sync<P: AsRef<Path>>(path: P) -> Result<&'static Config> {
    let bytes = fs::read(path)?;
    let mut config: Config = deserialize_toml_bytes(&bytes).unwrap_or_else(|err| {
        #[cfg(debug_assertions)]
        println!("WARNING: Unable to load config; using defaults: {:?}", err);
        Config::default()
    });
    config.input.verify()?;
    set_config(config);
    Ok(get_config())
}

pub fn get_config() -> &'static Config {
    unsafe { CONFIG.as_ref().unwrap() }
}

pub fn get_config_mut() -> &'static mut Config {
    unsafe { CONFIG.as_mut().unwrap() }
}

pub fn default_config_path() -> String {
    DEFAULT_CONFIG_PATH.to_string()
}