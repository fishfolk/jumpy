use crate::gl::gl_context;
use glow::{HasContext, NativeTexture};
use image::codecs::png::PngDecoder;
use image::{DynamicImage, ImageBuffer, ImageDecoder};
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::math::{vec2, Size, Vec2};
use crate::texture::{ColorFormat, TextureFilterMode, TextureFormat};
use crate::Result;

#[derive(Clone, Copy, Debug)]
pub struct Texture2D {
    id: NativeTexture,
    filter_mode: TextureFilterMode,
    size: Size<f32>,
}

impl PartialEq for Texture2D {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Texture2D {}

impl Texture2D {
    pub fn from_dynamic_image<F: Into<Option<TextureFilterMode>>>(
        image: DynamicImage,
        filter_mode: F,
    ) -> Result<Self> {
        let size = Size::new(image.width(), image.height()).as_f32();
        let format = glow::RGBA;

        let gl = gl_context();

        let id = unsafe {
            let id = gl.create_texture()?;

            gl.bind_texture(glow::TEXTURE_2D, Some(id));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                format as i32,
                size.width as i32,
                size.height as i32,
                0,
                format,
                glow::UNSIGNED_BYTE,
                Some(image.as_bytes()),
            );

            gl.bind_texture(glow::TEXTURE_2D, None);

            id
        };

        let filter_mode = filter_mode.into().unwrap_or_default();

        Ok(Texture2D {
            id,
            filter_mode,
            size,
        })
    }

    pub fn size(&self) -> Size<f32> {
        self.size
    }

    pub fn gl_texture(&self) -> NativeTexture {
        self.id
    }

    pub fn set_filter_mode(&mut self, filter_mode: TextureFilterMode) {
        self.filter_mode = filter_mode;
    }
}

pub fn load_texture_bytes<F: Into<Option<TextureFilterMode>>>(
    bytes: &[u8],
    filter_mode: F,
) -> Result<Texture2D> {
    let image = image::load_from_memory(bytes)?;

    Texture2D::from_dynamic_image(image, filter_mode)
}

pub async fn load_texture_file<P: AsRef<Path>, F: Into<Option<TextureFilterMode>>>(
    path: P,
    filter_mode: F,
) -> Result<Texture2D> {
    let image = image::open(&path)?;

    Texture2D::from_dynamic_image(image, filter_mode)
}
