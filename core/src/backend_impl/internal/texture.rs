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
    pub filter_mode: TextureFilterMode,
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
        let image = image.into_rgba8();
        let size = Size::new(image.width(), image.height()).as_f32();

        let gl = gl_context();

        let gl_texture = unsafe {
            let texture = gl.create_texture()?;

            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA8 as i32,
                size.width as i32,
                size.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                Some(&image.into_raw()),
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

    pub fn bind(&self, texture_unit: TextureUnit) {
        let gl = gl_context();
        unsafe {
            gl.active_texture(texture_unit.into());
            gl.bind_texture(glow::TEXTURE_2D, Some(self.gl_texture));

            let mode = match self.filter_mode {
                TextureFilterMode::Nearest => glow::NEAREST as i32,
                TextureFilterMode::Linear => glow::LINEAR as i32,
            };

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, mode);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, mode);
        }
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
