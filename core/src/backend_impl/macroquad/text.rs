use std::path::Path;

use macroquad::text::measure_text;
pub use macroquad::text::Font;

use crate::math::{vec2, Size, Vec2};
use crate::text::{HorizontalAlignment, TextParams, VerticalAlignment};
use crate::viewport::viewport_size;
use crate::Result;

impl From<TextParams> for macroquad::text::TextParams {
    fn from(params: TextParams) -> Self {
        macroquad::text::TextParams {
            font: params.font.unwrap_or_default(),
            font_size: params.font_size,
            font_scale: params.font_scale,
            font_scale_aspect: 1.0,
            color: params.color.into(),
        }
    }
}

pub fn load_font_bytes(bytes: &[u8]) -> Result<Font> {
    let font = macroquad::text::load_ttf_font_from_bytes(bytes)?;
    Ok(font)
}

const BASE_LINE_MARGIN: f32 = 2.0;

pub fn draw_text(text: &str, x: f32, y: f32, params: TextParams) {
    let bounds = params.bounds.unwrap_or_else(|| {
        let viewport_size = viewport_size();
        Size::new(viewport_size.width - x, viewport_size.height - y)
    });

    let font_size = params.font_size as f32 * params.font_scale;

    let mut words = text.split(' ').collect::<Vec<_>>();

    let mut y_offset = 0.0;
    let mut current_line = Vec::new();

    while !words.is_empty() {
        current_line.push(words.pop().unwrap());

        let mut line = current_line.join(" ");

        let mut measure = measure_text(&line, params.font, params.font_size, params.font_scale);

        let mut should_draw = false;

        if measure.width > bounds.width {
            words.push(current_line.pop().unwrap());

            line = current_line.join(" ");
            measure = measure_text(&line, params.font, params.font_size, params.font_scale);

            should_draw = true;
        } else if words.is_empty() {
            should_draw = true;
        }

        if should_draw {
            let x = match params.horizontal_align {
                HorizontalAlignment::Left => x,
                HorizontalAlignment::Center => x + ((bounds.width - measure.width) / 2.0),
                HorizontalAlignment::Right => x + bounds.width - measure.width,
            };

            let y = match params.vertical_align {
                VerticalAlignment::Normal => y,
                VerticalAlignment::Center => y - (measure.height / 2.0),
            };

            macroquad::text::draw_text_ex(&line, x, y + y_offset, params.clone().into());

            y_offset += measure.height + (BASE_LINE_MARGIN * font_size);

            current_line.clear();
        }
    }
}
