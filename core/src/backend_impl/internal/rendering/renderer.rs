use glow::{Context, HasContext, NativeProgram, NativeTexture, NativeVertexArray};

use crate::color::{colors, Color};
use crate::gl::gl_context;
use crate::math::{vec2, Rect, Size, Vec2};
use crate::rendering::vertex::Index;
use crate::rendering::{Buffer, DrawTextureParams, Vertex};
use crate::text::draw_queued_text;
use crate::texture::Texture2D;
use crate::window::{get_context_wrapper, get_window};
use crate::Result;

const BATCH_SIZE: usize = 128;

const QUAD_VERTEX_CNT: usize = 4;
const QUAD_INDEX_CNT: usize = 6;

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

const FRAGMENT_SHADER_SRC: &str = "
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
    pub(crate) current_texture: Option<NativeTexture>,
    pub(crate) current_program: Option<NativeProgram>,
    batched: Vec<Vertex>,
    batched_cnt: usize,
    indices: Vec<u32>,
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<Index>,
    vertex_array: NativeVertexArray,
}

impl Renderer {
    pub fn new() -> Result<Self> {
        let gl = gl_context();

        let vertex_buffer = Buffer::new_vertex()?;
        let index_buffer = Buffer::new_element()?;

        let vertex_array = unsafe { gl.create_vertex_array() }?;

        let mut indices = Vec::with_capacity(BATCH_SIZE * QUAD_INDEX_CNT);
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
            batched: Vec::with_capacity(BATCH_SIZE * QUAD_VERTEX_CNT),
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
            }

            gl.clear(glow::COLOR_BUFFER_BIT);
        };
    }

    pub fn draw_batch(&mut self) {
        self.vertex_buffer.bind();
        self.vertex_buffer.set_data(&self.batched);

        let index_cnt = self.batched_cnt * QUAD_INDEX_CNT;

        self.index_buffer.bind();
        self.index_buffer.set_data(&self.indices[0..index_cnt]);

        let gl = gl_context();
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, self.current_texture);

            gl.draw_elements(
                glow::TRIANGLES,
                self.batched_cnt as i32,
                glow::UNSIGNED_INT,
                0,
            );

            gl.bind_texture(glow::TEXTURE_2D, None);
        }

        self.vertex_buffer.unbind();
        self.index_buffer.unbind();

        self.batched.clear();
        self.batched_cnt = 0;
    }

    pub fn draw_texture(&mut self, x: f32, y: f32, texture: Texture2D, params: DrawTextureParams) {
        if let Some(current_texture) = self.current_texture {
            if current_texture != texture.gl_texture() {
                self.draw_batch();
                self.current_texture = Some(texture.gl_texture());
            }
        }

        if self.batched_cnt >= BATCH_SIZE {
            self.draw_batch();
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
        self.draw_batch();

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
