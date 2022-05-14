use glow::{HasContext, NativeTexture};
use image::codecs::png::PngDecoder;
use image::{DynamicImage, ImageBuffer, ImageDecoder};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;

pub use crate::image::ImageFormat as TextureFormat;

use crate::gl::gl_context;
use crate::image::{Image, ImageImpl};
use crate::math::{vec2, Size, Vec2};
use crate::render::renderer::Renderer;
use crate::result::Result;
use crate::texture::{ColorFormat, TextureFilterMode, TextureKind};

pub struct Texture2DImpl {
    gl_texture: NativeTexture,
    pub kind: TextureKind,
    pub filter_mode: TextureFilterMode,
    size: Size<f32>,
    frame_size: Option<Size<f32>>,
}

impl PartialEq for Texture2DImpl {
    fn eq(&self, other: &Self) -> bool {
        self.gl_texture == other.gl_texture
    }
}

impl PartialEq<NativeTexture> for Texture2DImpl {
    fn eq(&self, other: &NativeTexture) -> bool {
        self.gl_texture == *other
    }
}

impl Eq for Texture2DImpl {}

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
        let size = image.size();
        let frame_size = frame_size.into();

        let gl = gl_context();

        let gl_texture = unsafe {
            let texture = gl.create_texture()?;

            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::SRGB_ALPHA as i32,
                size.width as i32,
                size.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(image.as_raw()),
            );

            gl.bind_texture(glow::TEXTURE_2D, None);

            texture
        };

        let filter_mode = filter_mode.into().unwrap_or_default();

        Ok(Texture2DImpl {
            gl_texture,
            kind,
            filter_mode,
            size,
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
        let image = Image::from_bytes(bytes, format)?;
        Self::from_image(image, kind, filter_mode, frame_size)
    }

    pub fn gl_texture(&self) -> NativeTexture {
        self.gl_texture
    }

    pub fn bind(&self, texture_unit: TextureUnit) {
        let gl = gl_context();
        unsafe {
            gl.active_texture(texture_unit.into());
            gl.bind_texture(glow::TEXTURE_2D, Some(self.gl_texture));

            let mode = match self.filter_mode {
                TextureFilterMode::Nearest => glow::NEAREST as i32,
                TextureFilterMode::Linear => glow::LINEAR as i32,
            };

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, mode);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, mode);
        }
    }

    pub fn size(&self) -> Size<f32> {
        self.size
    }

    pub fn frame_size(&self) -> Size<f32> {
        self.frame_size.unwrap_or(self.size)
    }
}

impl Drop for Texture2DImpl {
    fn drop(&mut self) {
        let gl = gl_context();
        unsafe {
            gl.delete_texture(self.gl_texture);
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TextureUnit {
    Texture0,
    Texture1,
    Texture2,
    Texture3,
    Texture4,
    Texture5,
    Texture6,
    Texture7,
    Texture8,
    Texture9,
    Texture10,
    Texture11,
    Texture12,
    Texture13,
    Texture14,
    Texture15,
    Texture16,
    Texture17,
    Texture18,
    Texture19,
    Texture20,
    Texture21,
    Texture22,
    Texture23,
    Texture24,
    Texture25,
    Texture26,
    Texture27,
    Texture28,
    Texture29,
    Texture30,
    Texture31,
}

impl From<TextureUnit> for u32 {
    fn from(texture_unit: TextureUnit) -> Self {
        match texture_unit {
            TextureUnit::Texture0 => glow::TEXTURE0,
            TextureUnit::Texture1 => glow::TEXTURE1,
            TextureUnit::Texture2 => glow::TEXTURE2,
            TextureUnit::Texture3 => glow::TEXTURE3,
            TextureUnit::Texture4 => glow::TEXTURE4,
            TextureUnit::Texture5 => glow::TEXTURE5,
            TextureUnit::Texture6 => glow::TEXTURE6,
            TextureUnit::Texture7 => glow::TEXTURE7,
            TextureUnit::Texture8 => glow::TEXTURE8,
            TextureUnit::Texture9 => glow::TEXTURE9,
            TextureUnit::Texture10 => glow::TEXTURE10,
            TextureUnit::Texture11 => glow::TEXTURE11,
            TextureUnit::Texture12 => glow::TEXTURE12,
            TextureUnit::Texture13 => glow::TEXTURE13,
            TextureUnit::Texture14 => glow::TEXTURE14,
            TextureUnit::Texture15 => glow::TEXTURE15,
            TextureUnit::Texture16 => glow::TEXTURE16,
            TextureUnit::Texture17 => glow::TEXTURE17,
            TextureUnit::Texture18 => glow::TEXTURE18,
            TextureUnit::Texture19 => glow::TEXTURE19,
            TextureUnit::Texture20 => glow::TEXTURE20,
            TextureUnit::Texture21 => glow::TEXTURE21,
            TextureUnit::Texture22 => glow::TEXTURE22,
            TextureUnit::Texture23 => glow::TEXTURE23,
            TextureUnit::Texture24 => glow::TEXTURE24,
            TextureUnit::Texture25 => glow::TEXTURE25,
            TextureUnit::Texture26 => glow::TEXTURE26,
            TextureUnit::Texture27 => glow::TEXTURE27,
            TextureUnit::Texture28 => glow::TEXTURE28,
            TextureUnit::Texture29 => glow::TEXTURE29,
            TextureUnit::Texture30 => glow::TEXTURE30,
            TextureUnit::Texture31 => glow::TEXTURE31,
        }
    }
}

impl From<TextureUnit> for i32 {
    fn from(texture_unit: TextureUnit) -> Self {
        let uint: u32 = texture_unit.into();
        uint as i32
    }
}

impl From<u32> for TextureUnit {
    fn from(texture_unit: u32) -> Self {
        match texture_unit {
            glow::TEXTURE0 => Self::Texture0,
            glow::TEXTURE1 => Self::Texture1,
            glow::TEXTURE2 => Self::Texture2,
            glow::TEXTURE3 => Self::Texture3,
            glow::TEXTURE4 => Self::Texture4,
            glow::TEXTURE5 => Self::Texture5,
            glow::TEXTURE6 => Self::Texture6,
            glow::TEXTURE7 => Self::Texture7,
            glow::TEXTURE8 => Self::Texture8,
            glow::TEXTURE9 => Self::Texture9,
            glow::TEXTURE10 => Self::Texture10,
            glow::TEXTURE11 => Self::Texture11,
            glow::TEXTURE12 => Self::Texture12,
            glow::TEXTURE13 => Self::Texture13,
            glow::TEXTURE14 => Self::Texture14,
            glow::TEXTURE15 => Self::Texture15,
            glow::TEXTURE16 => Self::Texture16,
            glow::TEXTURE17 => Self::Texture17,
            glow::TEXTURE18 => Self::Texture18,
            glow::TEXTURE19 => Self::Texture19,
            glow::TEXTURE20 => Self::Texture20,
            glow::TEXTURE21 => Self::Texture21,
            glow::TEXTURE22 => Self::Texture22,
            glow::TEXTURE23 => Self::Texture23,
            glow::TEXTURE24 => Self::Texture24,
            glow::TEXTURE25 => Self::Texture25,
            glow::TEXTURE26 => Self::Texture26,
            glow::TEXTURE27 => Self::Texture27,
            glow::TEXTURE28 => Self::Texture28,
            glow::TEXTURE29 => Self::Texture29,
            glow::TEXTURE30 => Self::Texture30,
            glow::TEXTURE31 => Self::Texture31,
            _ => panic!("ERROR: Invalid texture unit '{}'!", texture_unit),
        }
    }
}
