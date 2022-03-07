use std::path::Path;

use macroquad::texture::load_texture;

use crate::Result;
use crate::texture::{TextureFilterMode, TextureFormat};
use crate::math::{Size, Vec2, vec2};

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

pub fn load_texture_bytes<F: Into<Option<TextureFilterMode>>>(bytes: &[u8], format: TextureFormat, filter_mode: F) -> Result<Texture2D> {
    assert_eq!(format, TextureFormat::Png, "Only png textures are supported when using the macroquad backend (is '{}')!", format.to_string());

    let texture = macroquad::texture::Texture2D::from_file_with_format(bytes, None);

    let filter_mode = filter_mode.into().unwrap_or_default();
    texture.set_filter(filter_mode.into());

    Ok(texture.into())
}