use std::path::Path;

pub use macroquad::text::{Font, TextParams, draw_text_ex as draw_text, measure_text};

use crate::Result;

pub fn load_ttf_font_bytes(bytes: &[u8]) -> Result<Font> {
    let font = macroquad::text::load_ttf_font_from_bytes(bytes)?;
    Ok(font)
}