use std::path::Path;

use crate::Result;
use crate::math::{Vec2, vec2, Size};
use crate::texture::{TextureFilterMode, TextureFormat};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Texture2D {
    filter_mode: TextureFilterMode,
    format: TextureFormat,
    size: Size<f32>,
}

impl Texture2D {
    pub fn size(&self) -> Size<f32> {
        self.size
    }

    pub fn format(&self) -> TextureFormat {
        self.format
    }

    pub fn set_filter_mode(&mut self, filter_mode: TextureFilterMode) {
        self.filter_mode = filter_mode;
    }
}

pub fn load_texture_bytes<F: Into<Option<TextureFilterMode>>>(bytes: &[u8], format: TextureFormat, filter_mode: F) -> Result<Texture2D> {
    unimplemented!("Texture loading is not implemented")
}