use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::parsing::load_toml_file;
use crate::input::InputMapping;
use crate::error::Result;
use crate::video::RenderingConfig;
use crate::window::WindowConfig;

pub use crate::backend_impl::config::*;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub rendering: RenderingConfig,
    #[serde(default)]
    pub window: WindowConfig,
    #[serde(default)]
    pub input: InputMapping,
}


