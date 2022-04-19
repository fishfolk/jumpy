use glow::HasContext;

use crate::color::Color;
use crate::gl::gl_context;
use crate::math::{Circle, Rect, Vec2};
use crate::rendering::DrawTextureParams;
use crate::texture::Texture2D;
use crate::window::context_wrapper;
use crate::Result;

pub fn clear_screen<C: Into<Option<Color>>>(color: C) {
    let gl = gl_context();
    unsafe {
        if let Some(color) = color.into() {
            gl.clear_color(color.r, color.g, color.b, color.a);
        } else {
            gl.clear(glow::COLOR_BUFFER_BIT);
        }
    };
}

pub fn end_frame() -> Result<()> {
    context_wrapper().swap_buffers()?;
    Ok(())
}

pub fn draw_texture(x: f32, y: f32, texture: Texture2D, params: DrawTextureParams) {
    unimplemented!("Draw calls are not implemented")
}

pub fn draw_rectangle(x: f32, y: f32, width: f32, height: f32, color: Color) {
    unimplemented!("Draw calls are not implemented")
}

pub fn draw_rectangle_outline(x: f32, y: f32, width: f32, height: f32, weight: f32, color: Color) {
    unimplemented!("Draw calls are not implemented")
}

pub fn draw_circle(x: f32, y: f32, radius: f32, color: Color) {
    unimplemented!("Draw calls are not implemented")
}

pub fn draw_circle_outline(x: f32, y: f32, radius: f32, weight: f32, color: Color) {
    unimplemented!("Draw calls are not implemented")
}

pub fn draw_line(x: f32, y: f32, end_x: f32, end_y: f32, weight: f32, color: Color) {
    unimplemented!("Draw calls are not implemented")
}
