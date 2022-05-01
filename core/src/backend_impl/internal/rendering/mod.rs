use glow::HasContext;

pub mod buffer;
pub mod render_target;
pub mod renderer;
pub mod shader;
pub mod vertex;
pub mod vertex_array;

pub use buffer::Buffer;
pub use render_target::RenderTarget;
pub use shader::{Shader, ShaderProgram};
pub use vertex::{Index, Vertex};
pub use vertex_array::VertexArray;

use crate::color::Color;
use crate::rendering::DrawTextureParams;
use crate::texture::Texture2D;
use crate::Result;

use crate::video::VideoConfig;
use renderer::*;

pub fn clear_screen<C: Into<Option<Color>>>(clear_color: C) {
    if let Some(clear_color) = clear_color.into() {
        set_clear_color(clear_color);
    }

    renderer().clear_screen();
}

pub fn set_clear_color(clear_color: Color) {
    renderer().set_clear_color(clear_color)
}

pub fn draw_texture(x: f32, y: f32, texture: Texture2D, params: DrawTextureParams) {
    renderer().draw_texture(x, y, texture, params)
}

pub fn draw_rectangle(x: f32, y: f32, width: f32, height: f32, color: Color) {}

pub fn draw_rectangle_outline(x: f32, y: f32, width: f32, height: f32, weight: f32, color: Color) {}

pub fn draw_circle(x: f32, y: f32, radius: f32, color: Color) {}

pub fn draw_circle_outline(x: f32, y: f32, radius: f32, weight: f32, color: Color) {}

pub fn draw_line(x: f32, y: f32, end_x: f32, end_y: f32, weight: f32, color: Color) {}

pub fn use_program(program: ShaderProgram) {
    renderer().use_program(program)
}

pub fn fps() -> u32 {
    renderer().fps()
}

pub fn end_frame() -> Result<()> {
    renderer().end_frame()?;
    Ok(())
}

pub(crate) fn apply_video_config(config: &VideoConfig) {
    renderer().apply_config(config);
}
