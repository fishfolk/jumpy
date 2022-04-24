use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::math::Vec2;
use crate::Result;

pub use crate::backend_impl::texture::*;
use crate::file::read_from_file;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureFormat {
    Png,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorFormat {
    Rgb,
    Rgba,
}

impl ToString for TextureFormat {
    fn to_string(&self) -> String {
        match self {
            TextureFormat::Png => "png".to_string(),
        }
    }
}

impl Default for TextureFormat {
    fn default() -> Self {
        TextureFormat::Png
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureFilterMode {
    Linear,
    Nearest,
}

impl Default for TextureFilterMode {
    fn default() -> Self {
        TextureFilterMode::Nearest
    }
}
