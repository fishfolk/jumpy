use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::file::read_from_file;
use crate::math::Size;
use crate::result::Result;

pub use crate::backend_impl::text::*;
use crate::color::{colors, Color};
use crate::parsing::deserialize_bytes_by_extension;

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

const FONTS_FILE: &str = "fonts";

static mut FONTS: Option<HashMap<String, Font>> = None;

pub fn try_get_font(id: &str) -> Option<Font> {
    unsafe { FONTS.get_or_insert_with(HashMap::new).get(id).cloned() }
}

pub fn get_font(id: &str) -> Font {
    try_get_font(id).unwrap()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontMetadata {
    pub id: String,
    pub path: String,
}

pub async fn load_fonts<P: AsRef<Path>>(
    path: P,
    ext: &str,
    is_required: bool,
    should_overwrite: bool,
) -> Result<()> {
    let fonts = unsafe { FONTS.get_or_insert_with(HashMap::new) };

    if should_overwrite {
        fonts.clear();
    }

    let fonts_file_path = path.as_ref().join(FONTS_FILE).with_extension(ext);

    match read_from_file(&fonts_file_path).await {
        Err(err) => {
            if is_required {
                return Err(err.into());
            }
        }
        Ok(bytes) => {
            let metadata: Vec<FontMetadata> = deserialize_bytes_by_extension(ext, &bytes)?;

            for meta in metadata {
                let file_path = path.as_ref().join(&meta.path);

                let font = load_font(&file_path).await?;

                let key = meta.id.clone();

                fonts.insert(key, font);
            }
        }
    }

    Ok(())
}
