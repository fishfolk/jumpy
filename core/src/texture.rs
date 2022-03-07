use std::path::Path;

use serde::{Serialize, Deserialize};

use crate::Result;
use crate::math::Vec2;

pub use crate::backend_impl::texture::*;
use crate::file::load_file;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextureFormat {
    Png,
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

pub async fn load_texture_file<P: AsRef<Path>, F: Into<Option<TextureFilterMode>>>(path: P, format: TextureFormat, filter_mode: F) -> Result<Texture2D> {
    let bytes = load_file(path).await?;
    load_texture_bytes(&bytes, format, filter_mode)
}