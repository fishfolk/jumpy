use glow::{Context, HasContext, NativeBuffer, NativeTexture, NativeVertexArray};

use crate::color::{colors, Color};
use crate::gl::gl_context;
use crate::math::{vec2, Rect, Size, Vec2};
use crate::rendering::DrawTextureParams;
use crate::text::draw_queued_text;
use crate::texture::Texture2D;
use crate::window::{get_context_wrapper, get_window};
use crate::Result;

pub struct Vertex {
    position: Vec2,
    color: Color,
    texture_coords: Vec2,
}

const BATCH_SIZE: usize = 128;

const MAX_TEXTURE_WIDTH: u32 = 2048;
const MAX_TEXTURE_HEIGHT: u32 = 2048;

const VERTEX_SHADER_SRC: &str = "
layout(location = 0) in vec3 vertex_position;
layout(location = 1) in vec3 vertex_colour;
layout(location = 2) in vec2 texture_coords;

out vec3 colour;

void main() {
  colour = vertex_colour;
  gl_Position = vec4(vertex_position, 1.0);
}
";

pub struct Renderer {
    current_texture: Option<NativeTexture>,
    batched: Vec<Vertex>,
    batched_cnt: usize,
    indices: Vec<u32>,
    vertex_buffer: NativeBuffer,
    index_buffer: NativeBuffer,
    vertex_array: NativeVertexArray,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let gl = gl_context();

        let (vertex_buffer, index_buffer, vertex_array) = unsafe {
            gl.enable(glow::FRAMEBUFFER_SRGB);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);

            let vertex_buffer = gl.create_buffer()?;
            let index_buffer = gl.create_buffer()?;
            let vertex_array = gl.create_vertex_array()?;

            (vertex_buffer, index_buffer, vertex_array)
        };

        let mut indices = Vec::with_capacity(BATCH_SIZE * 6);
        for i in 0..BATCH_SIZE {
            let offset = i as u32 * 3;

            indices.push(0 + offset);
            indices.push(1 + offset);
            indices.push(2 + offset);
            indices.push(2 + offset);
            indices.push(1 + offset);
            indices.push(3 + offset);
        }

        Ok(Renderer {
            current_texture: None,
            batched: Vec::with_capacity(BATCH_SIZE * 4),
            batched_cnt: 0,
            indices,
            vertex_buffer,
            index_buffer,
            vertex_array,
        })
    }

    pub fn clear_screen<C: Into<Option<Color>>>(&self, color: C) {
        let gl = gl_context();
        unsafe {
            if let Some(color) = color.into() {
                gl.clear_color(color.red, color.green, color.blue, color.alpha);
            } else {
                gl.clear(glow::COLOR_BUFFER_BIT);
            }
        };
    }

    fn draw_batched(&mut self) {
        let gl = gl_context();

        unsafe {
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vertex_buffer));

            let vertices_slice: &[u8] = core::slice::from_raw_parts(
                self.batched.as_ptr() as *const u8,
                self.batched.len() * core::mem::size_of::<Vertex>(),
            );

            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertices_slice, glow::STATIC_DRAW);

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.index_buffer));

            let indices_slice: &[u8] = core::slice::from_raw_parts(
                self.indices.as_ptr() as *const u8,
                self.indices.len() * core::mem::size_of::<u32>(),
            );

            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, indices_slice, glow::STATIC_DRAW);

            gl.draw_elements(
                glow::TRIANGLES,
                self.batched_cnt as i32,
                glow::UNSIGNED_INT,
                0,
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }

        self.batched.clear();
        self.batched_cnt = 0;
    }

    pub fn draw_texture(&mut self, x: f32, y: f32, texture: Texture2D, params: DrawTextureParams) {
        if let Some(current_texture) = self.current_texture {
            if current_texture != texture.gl_texture() {
                self.draw_batched();
                self.current_texture = Some(texture.gl_texture());
            }
        }

        if self.batched_cnt > BATCH_SIZE {
            self.draw_batched();
        }

        let texture_rect = params.source.map(|urect| urect.into()).unwrap_or_else(|| {
            let size = texture.size();
            Rect::new(0.0, 0.0, size.width as f32, size.height as f32)
        });

        let size = params.dest_size.unwrap_or_else(|| texture.size());

        let color = params.tint.unwrap_or_else(|| colors::WHITE).into();

        self.batched.push(Vertex {
            position: vec2(x, y),
            color,
            texture_coords: vec2(texture_rect.x, texture_rect.y),
        });

        self.batched.push(Vertex {
            position: vec2(x + size.width, y),
            color,
            texture_coords: vec2(texture_rect.x + texture_rect.width, texture_rect.y),
        });

        self.batched.push(Vertex {
            position: vec2(x, y + size.height),
            color,
            texture_coords: vec2(texture_rect.x, texture_rect.y + texture_rect.height),
        });

        self.batched.push(Vertex {
            position: vec2(x + size.width, y + size.height),
            color,
            texture_coords: vec2(
                texture_rect.x + texture_rect.width,
                texture_rect.y + texture_rect.height,
            ),
        });

        self.batched_cnt += 1;
    }

    pub fn end_frame(&mut self) -> Result<()> {
        self.draw_batched();

        draw_queued_text()?;

        get_context_wrapper().swap_buffers()?;

        get_window().request_redraw();

        Ok(())
    }
}

static mut RENDERER: Option<Renderer> = None;

pub fn renderer() -> &'static mut Renderer {
    unsafe {
        RENDERER.get_or_insert_with(|| {
            Renderer::new()
                .unwrap_or_else(|err| panic!("ERROR: Unable to create default renderer: {}", err))
        })
    }
}
