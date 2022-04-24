use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::file::read_from_file;
use crate::math::{Size, Vec2};
use crate::Result;

pub use crate::backend_impl::text::*;
use crate::color::{colors, Color};

pub async fn load_font<P: AsRef<Path>>(path: P) -> Result<Font> {
    let bytes = read_from_file(path).await?;
    load_font_bytes(&bytes)
}

/// Arguments for "draw_text_ex" function such as font, font_size etc
#[derive(Debug, Clone)]
pub struct TextParams {
    pub font: Option<Font>,
    pub bounds: Option<Size<f32>>,
    pub horizontal_align: HorizontalAlignment,
    pub vertical_align: VerticalAlignment,
    pub font_size: u16,
    pub font_scale: f32,
    pub color: Color,
}

impl Default for TextParams {
    fn default() -> TextParams {
        TextParams {
            font: None,
            bounds: None,
            horizontal_align: HorizontalAlignment::default(),
            vertical_align: VerticalAlignment::default(),
            font_size: 20,
            font_scale: 1.0,
            color: colors::WHITE,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalAlignment {
    Left,
    Right,
    Center,
}

impl Default for HorizontalAlignment {
    fn default() -> Self {
        Self::Left
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VerticalAlignment {
    Normal,
    Center,
}

impl Default for VerticalAlignment {
    fn default() -> Self {
        Self::Normal
    }
}
