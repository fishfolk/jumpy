use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub fullscreen: bool,
    pub high_dpi: bool,
    pub resolution: Resolution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: i32,
    pub height: i32,
}

impl Config {
    pub fn parse(path: PathBuf) -> Result<Self, Error> {
        let file_contents = fs::read_to_string(&path)?;
        Ok(serde_json::from_str(&file_contents)?)
    }
}
