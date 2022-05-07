use std::path::Path;

pub use macroquad::prelude::ImageFormat as TextureFormat;

use macroquad::texture::load_texture;

use crate::file::read_from_file;
use crate::math::{vec2, Size, Vec2};
use crate::result::Result;
use crate::texture::{TextureFilterMode, TextureKind};

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
pub struct Texture2DImpl {
    texture_impl: macroquad::texture::Texture2D,
    pub kind: TextureKind,
    pub filter_mode: TextureFilterMode,
    frame_size: Option<Size<f32>>,
}

impl Texture2DImpl {
    pub(crate) fn from_bytes<T, K, F, S>(
        bytes: &[u8],
        format: T,
        kind: K,
        filter_mode: F,
        frame_size: S,
    ) -> Result<Self>
    where
        T: Into<Option<TextureFormat>>,
        K: Into<Option<TextureKind>>,
        F: Into<Option<TextureFilterMode>>,
        S: Into<Option<Size<f32>>>,
    {
        let format = format.into().map(|f| f.into());
        let kind = kind.into().unwrap_or_default();
        let filter_mode = filter_mode.into().unwrap_or_default();
        let frame_size = frame_size.into();

        let texture_impl = macroquad::texture::Texture2D::from_file_with_format(bytes, format);
        texture_impl.set_filter(filter_mode.into());

        Ok(Texture2DImpl {
            texture_impl,
            kind,
            filter_mode,
            frame_size,
        })
    }

    pub fn mq_texture(&self) -> macroquad::texture::Texture2D {
        self.texture_impl
    }

    pub fn size(&self) -> Size<f32> {
        self.texture_impl.size()
    }

    pub fn frame_size(&self) -> Size<f32> {
        self.frame_size.unwrap_or(self.size())
    }
}
