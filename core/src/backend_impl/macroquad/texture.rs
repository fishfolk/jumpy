use std::path::Path;

use crate::file::read_from_file;
use macroquad::texture::load_texture;

use crate::math::{vec2, Size, Vec2};
use crate::texture::{TextureFilterMode, TextureFormat};
use crate::Result;

impl From<macroquad::texture::FilterMode> for TextureFilterMode {
    fn from(mode: macroquad::texture::FilterMode) -> Self {
        match mode {
            macroquad::texture::FilterMode::Nearest => TextureFilterMode::Nearest,
            macroquad::texture::FilterMode::Linear => TextureFilterMode::Linear,
        }
    }
}

impl From<TextureFilterMode> for macroquad::texture::FilterMode {
    fn from(mode: TextureFilterMode) -> Self {
        match mode {
            TextureFilterMode::Nearest => macroquad::texture::FilterMode::Nearest,
            TextureFilterMode::Linear => macroquad::texture::FilterMode::Linear,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Texture2D(macroquad::texture::Texture2D);

impl Texture2D {
    pub fn size(&self) -> Size<f32> {
        Size::new(self.0.width(), self.0.height())
    }

    pub fn format(&self) -> TextureFormat {
        TextureFormat::Png
    }

    pub fn set_filter_mode(&mut self, filter_mode: TextureFilterMode) {
        self.0.set_filter(filter_mode.into())
    }
}

impl From<macroquad::texture::Texture2D> for Texture2D {
    fn from(texture: macroquad::texture::Texture2D) -> Self {
        Texture2D(texture)
    }
}

impl From<Texture2D> for macroquad::texture::Texture2D {
    fn from(texture: Texture2D) -> Self {
        texture.0
    }
}

pub fn load_texture_bytes<F: Into<Option<TextureFilterMode>>>(
    bytes: &[u8],
    filter_mode: F,
) -> Result<Texture2D> {
    let texture = macroquad::texture::Texture2D::from_file_with_format(bytes, None);

    let filter_mode = filter_mode.into().unwrap_or_default();
    texture.set_filter(filter_mode.into());

    Ok(texture.into())
}

pub async fn load_texture_file<P: AsRef<Path>, F: Into<Option<TextureFilterMode>>>(
    path: P,
    filter_mode: F,
) -> Result<Texture2D> {
    let bytes = read_from_file(path).await?;
    load_texture_bytes(&bytes, filter_mode)
}
