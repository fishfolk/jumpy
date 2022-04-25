use crate::gl::gl_context;
use glow::{HasContext, NativeTexture};
use image::codecs::png::PngDecoder;
use image::{DynamicImage, ImageBuffer, ImageDecoder};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::path::Path;

use crate::math::{vec2, Size, Vec2};
use crate::rendering::renderer::Renderer;
use crate::texture::{ColorFormat, TextureFilterMode, TextureFormat};
use crate::Result;

pub struct Texture2DImpl {
    gl_texture: NativeTexture,
    filter_mode: TextureFilterMode,
    size: Size<f32>,
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
    pub fn size(&self) -> Size<f32> {
        self.size
    }

    pub fn set_filter_mode(&mut self, filter_mode: TextureFilterMode) {
        self.filter_mode = filter_mode;
    }

    pub fn gl_texture(&self) -> NativeTexture {
        self.gl_texture
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

static mut NEXT_TEXTURE_INDEX: usize = 0;
static mut TEXTURES: Option<HashMap<usize, Texture2DImpl>> = None;

fn texture_map() -> &'static mut HashMap<usize, Texture2DImpl> {
    unsafe { TEXTURES.get_or_insert_with(HashMap::new) }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Texture2D(usize);

impl Texture2D {
    pub fn from_dynamic_image<F: Into<Option<TextureFilterMode>>>(
        image: DynamicImage,
        filter_mode: F,
    ) -> Result<Self> {
        let size = Size::new(image.width(), image.height()).as_f32();
        let format = glow::RGBA;

        let gl = gl_context();

        let gl_texture = unsafe {
            let texture = gl.create_texture()?;

            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

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

            texture
        };

        let filter_mode = filter_mode.into().unwrap_or_default();

        let index = unsafe {
            let index = NEXT_TEXTURE_INDEX;
            NEXT_TEXTURE_INDEX += 1;
            index
        };

        texture_map().insert(
            index,
            Texture2DImpl {
                gl_texture,
                filter_mode,
                size,
            },
        );

        Ok(Texture2D(index))
    }
}

impl Deref for Texture2D {
    type Target = Texture2DImpl;

    fn deref(&self) -> &Self::Target {
        texture_map().get(&self.0).unwrap()
    }
}

impl DerefMut for Texture2D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        texture_map().get_mut(&self.0).unwrap()
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
