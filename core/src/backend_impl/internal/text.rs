use std::path::Path;
use crate::color::{colors, Color};
use crate::Result;

/// TTF font loaded to GPU
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Font(usize);

pub fn load_ttf_font_bytes(bytes: &[u8]) -> Result<Font> {
    unimplemented!("Font loading is not implemented")
}

/// Arguments for "draw_text_ex" function such as font, font_size etc
#[derive(Debug, Clone, Copy)]
pub struct TextParams {
    pub font: Font,
    /// Base size for character height. The size in pixel used during font rasterizing.
    pub font_size: u16,
    /// The glyphs sizes actually drawn on the screen will be font_size * font_scale
    /// However with font_scale too different from 1.0 letters may be blurry
    pub font_scale: f32,
    /// Font X axis would be scaled by font_scale * font_scale_aspect
    /// and Y axis would be scaled by font_scale
    /// Default is 1.0
    pub font_scale_aspect: f32,
    pub color: Color,
}

impl Default for TextParams {
    fn default() -> TextParams {
        TextParams {
            font: Font(0),
            font_size: 20,
            font_scale: 1.0,
            font_scale_aspect: 1.0,
            color: colors::WHITE,
        }
    }
}

pub fn draw_text(text: &str, x: f32, y: f32, params: TextParams) {
    unimplemented!("Text draw is not implemented")
}

/// World space dimensions of the text, measured by "measure_text" function
#[derive(Default)]
pub struct TextDimensions {
    /// Distance from very left to very right of the rasterized text
    pub width: f32,
    /// Distance from the bottom to the top of the text.
    pub height: f32,
    /// Height offset from the baseline of the text.
    /// "draw_text(.., X, Y, ..)" will be rendered in a "Rect::new(X, Y - dimensions.offset_y, dimensions.width, dimensions.height)"
    /// For reference check "text_dimensions" example.
    pub offset_y: f32,
}

pub fn measure_text(
    text: &str,
    font: Option<Font>,
    font_size: u16,
    font_scale: f32,
) -> TextDimensions {
    unimplemented!("Text measuring is not implemented")
}