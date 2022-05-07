use glow_glyph::ab_glyph::FontArc;
use glow_glyph::{FontId, GlyphBrush, GlyphBrushBuilder, Section, Text};
use std::collections::HashMap;
use std::path::Path;

use crate::color::{colors, Color};
use crate::gl::gl_context;
use crate::math::Size;
use crate::result::Result;
use crate::text::TextParams;
use crate::viewport::viewport_size;

#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub struct Font(usize);

static mut FONTS: Option<Vec<FontArc>> = None;

fn fonts() -> &'static mut Vec<FontArc> {
    unsafe {
        FONTS.get_or_insert_with(|| {
            let default =
                FontArc::try_from_slice(include_bytes!("../../../assets/AnonymousPro-Regular.ttf"))
                    .unwrap_or_else(|err| panic!("ERROR: Unable to load default font: {}", err));
            vec![default]
        })
    }
}

pub fn load_font_bytes(bytes: &[u8]) -> Result<Font> {
    let font = FontArc::try_from_vec(bytes.to_vec())?;
    fonts().push(font);
    Ok(Font(fonts().len() - 1))
}

pub fn default_font() -> Font {
    let _ = fonts();
    Font(0)
}

static mut BRUSH: Option<GlyphBrush> = None;

fn brush() -> &'static mut GlyphBrush {
    unsafe {
        BRUSH.get_or_insert_with(|| {
            let fonts = fonts().clone();
            GlyphBrushBuilder::using_fonts(fonts).build(gl_context())
        })
    }
}

pub fn draw_text(text: &str, x: f32, y: f32, params: TextParams) {
    let font = params.font.unwrap_or_else(|| default_font());

    let bounds = params.bounds.unwrap_or_else(|| {
        let viewport_size = viewport_size();
        Size::new(viewport_size.width - x, viewport_size.height - y)
    });

    let font_size = (params.font_size as f32 * params.font_scale).round();

    brush().queue(Section {
        screen_position: (x, y),
        bounds: (bounds.width, bounds.height),
        text: vec![Text::default()
            .with_text(text)
            .with_font_id(FontId(font.0))
            .with_color(params.color.to_array())
            .with_scale(font_size)],
        ..Section::default()
    })
}

pub fn draw_queued_text() -> Result<()> {
    let viewport_size = viewport_size();
    brush().draw_queued(
        gl_context(),
        viewport_size.width as u32,
        viewport_size.height as u32,
    )?;
    Ok(())
}
