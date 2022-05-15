use std::ops::Deref;

pub use crate::image::ImageFormat as TextureFormat;

use crate::image::Image;
use crate::math::Size;
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
    mq_texture: macroquad::texture::Texture2D,
    pub kind: TextureKind,
    pub filter_mode: TextureFilterMode,
    frame_size: Option<Size<f32>>,
}

impl Texture2DImpl {
    pub(crate) fn from_image<K, F, S>(
        image: Image,
        kind: K,
        filter_mode: F,
        frame_size: S,
    ) -> Result<Self>
    where
        K: Into<Option<TextureKind>>,
        F: Into<Option<TextureFilterMode>>,
        S: Into<Option<Size<f32>>>,
    {
        let kind = kind.into().unwrap_or_default();
        let filter_mode = filter_mode.into().unwrap_or_default();
        let frame_size = frame_size.into();

        let texture_impl = macroquad::texture::Texture2D::from_image(image.deref());
        texture_impl.set_filter(filter_mode.into());

        Ok(Texture2DImpl {
            mq_texture: texture_impl,
            kind,
            filter_mode,
            frame_size,
        })
    }

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
            mq_texture: texture_impl,
            kind,
            filter_mode,
            frame_size,
        })
    }

    pub fn size(&self) -> Size<f32> {
        Size::new(self.mq_texture.width(), self.mq_texture.height())
    }

    pub fn frame_size(&self) -> Size<f32> {
        self.frame_size.unwrap_or_else(|| self.size())
    }
}

impl From<Texture2DImpl> for macroquad::texture::Texture2D {
    fn from(texture: Texture2DImpl) -> Self {
        texture.mq_texture
    }
}

impl From<&Texture2DImpl> for macroquad::texture::Texture2D {
    fn from(texture: &Texture2DImpl) -> Self {
        texture.mq_texture
    }
}
