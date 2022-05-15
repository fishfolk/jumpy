use glow::HasContext;

pub mod buffer;
pub mod renderer;
pub mod shader;
pub mod vertex;
pub mod vertex_array;

pub use buffer::Buffer;
pub use shader::{Shader, ShaderProgram};
pub use vertex::{Index, Vertex};
pub use vertex_array::VertexArray;

use crate::color::Color;
use crate::render::DrawTextureParams;
use crate::result::Result;
use crate::texture::Texture2D;

use crate::gui::draw_gui;
use crate::text::draw_queued_text;
use crate::video::VideoConfig;
use crate::window::{context_wrapper, window};
use renderer::*;

pub fn clear_screen<C: Into<Option<Color>>>(clear_color: C) {
    renderer().clear_screen(clear_color);
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

pub fn begin_frame() {
    renderer().reset_stats();

    clear_screen(None);
}

pub fn end_frame() -> Result<()> {
    renderer().draw_batch();

    /*
    let viewport_size = viewport_size();

    let mut should_show_fps = self.should_show_fps;

    #[cfg(debug_assertions)]
    {
        draw_text(
            "polygons:\ndraws:",
            viewport_size.width - 175.0,
            70.0,
            TextParams {
                bounds: Some(Size::new(75.0, 100.0)),
                ..Default::default()
            },
        );

        draw_text(
            &format!("{}\n{}", self.polygons, self.draws),
            viewport_size.width - 75.0,
            70.0,
            TextParams {
                bounds: Some(Size::new(75.0, 100.0)),
                ..Default::default()
            },
        );

        should_show_fps = true;
    }

    if should_show_fps {
        draw_text(
            "FPS:",
            viewport_size.width - 175.0,
            50.0,
            TextParams {
                bounds: Some(Size::new(75.0, 100.0)),
                ..Default::default()
            },
        );

        draw_text(
            &format!("{}", self.fps()),
            viewport_size.width - 75.0,
            50.0,
            TextParams {
                bounds: Some(Size::new(75.0, 100.0)),
                ..Default::default()
            },
        );
    }
    */

    draw_queued_text()?;

    draw_gui();

    context_wrapper().swap_buffers()?;

    window().request_redraw();

    Ok(())
}

pub(crate) fn apply_video_config(config: &VideoConfig) {
    renderer().apply_config(config);
}
