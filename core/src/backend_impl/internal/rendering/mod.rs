use glow::HasContext;

mod renderer;
mod shader;

pub use shader::*;

use crate::color::Color;
use crate::rendering::DrawTextureParams;
use crate::texture::Texture2D;
use crate::Result;

use renderer::*;

pub fn clear_screen<C: Into<Option<Color>>>(color: C) {
    renderer().clear_screen(color);
}

pub fn end_frame() -> Result<()> {
    renderer().end_frame()?;
    Ok(())
}

pub fn draw_texture(x: f32, y: f32, texture: Texture2D, params: DrawTextureParams) {
    renderer().draw_texture(x, y, texture, params)
}

pub fn draw_rectangle(x: f32, y: f32, width: f32, height: f32, color: Color) {}

pub fn draw_rectangle_outline(x: f32, y: f32, width: f32, height: f32, weight: f32, color: Color) {}

pub fn draw_circle(x: f32, y: f32, radius: f32, color: Color) {}

pub fn draw_circle_outline(x: f32, y: f32, radius: f32, weight: f32, color: Color) {}

pub fn draw_line(x: f32, y: f32, end_x: f32, end_y: f32, weight: f32, color: Color) {}
