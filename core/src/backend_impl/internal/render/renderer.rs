use crate::camera::{cameras, main_camera};
use glam::{vec3, Mat4};
use glow::{Context, HasContext, NativeProgram, NativeTexture, NativeVertexArray};
use std::any::Any;
use std::time::Duration;

use crate::color::{colors, Color};
use crate::config::Config;
use crate::game::draw_delta_time;
use crate::gl::gl_context;
use crate::math::{vec2, Rect, Size, Vec2};
use crate::prelude::shader::ShaderKind;
use crate::prelude::vertex::VertexLayout;
use crate::prelude::viewport_size;
use crate::render::shader::{Uniform, UniformType};
use crate::render::vertex::{Index, VertexImpl};
use crate::render::{
    Buffer, DrawTextureParams, RenderTarget, Shader, ShaderProgram, Vertex, VertexArray,
};
use crate::result::Result;
use crate::text::{draw_queued_text, draw_text, HorizontalAlignment, TextParams};
use crate::texture::{Texture2D, TextureFilterMode, TextureUnit};
use crate::video::VideoConfig;
use crate::viewport::viewport;
use crate::window::{get_context_wrapper, get_window};

const BATCH_SIZE: usize = 128;

const FPS_FRAME_HISTORY_CNT: usize = 32;

const QUAD_VERTEX_CNT: usize = 4;
const QUAD_INDEX_CNT: usize = 6;

const VERTEX_SHADER_SRC: &str = "
#version 420

layout(location = 0) in vec2 vertex_position;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in vec2 texture_coords;

uniform mat4 mvp;

out vec4 color;
out vec2 uv;

void main() {
  color = vertex_color;
  uv = texture_coords;

  gl_Position = mvp * vec4(vertex_position, 0.0, 1.0);
}
";

const FRAGMENT_SHADER_SRC: &str = "
#version 420
layout(binding = 0) uniform sampler2D texture_sampler;

in vec4 color;
in vec2 uv;

out vec4 frag_color;

void main() {
  frag_color = texture(texture_sampler, uv) * color;
}
";

pub struct Renderer {
    clear_color: Option<Color>,
    current_texture: Option<Texture2D>,
    current_program: Option<ShaderProgram>,
    should_show_fps: bool,
    draws: u32,
    polygons: u32,
    frame_history: Vec<Duration>,
    batched: Vec<Vertex>,
    batched_cnt: usize,
    indices: Vec<u32>,
    vertex_buffer: Buffer<Vertex>,
    index_buffer: Buffer<Index>,
    vertex_array: VertexArray,
}

impl Renderer {
    pub fn new(config: &VideoConfig) -> Result<Self> {
        let vertex_array = VertexArray::new::<Vertex>()?;

        let vertex_buffer = Buffer::new_vertex(BATCH_SIZE * QUAD_VERTEX_CNT)?;
        let index_buffer = Buffer::new_element(BATCH_SIZE * QUAD_INDEX_CNT)?;

        let mut indices = Vec::with_capacity(BATCH_SIZE * QUAD_INDEX_CNT);
        for i in 0..BATCH_SIZE {
            let offset = i * QUAD_VERTEX_CNT;

            indices.push(0 + offset as u32);
            indices.push(1 + offset as u32);
            indices.push(2 + offset as u32);
            indices.push(2 + offset as u32);
            indices.push(1 + offset as u32);
            indices.push(3 + offset as u32);
        }

        vertex_array.enable_layout();

        let program = ShaderProgram::new(
            &[
                Shader::new(ShaderKind::Vertex, VERTEX_SHADER_SRC.as_bytes())?,
                Shader::new(ShaderKind::Fragment, FRAGMENT_SHADER_SRC.as_bytes())?,
            ],
            &[("mvp", UniformType::Mat4)],
        )?;

        let gl = gl_context();
        unsafe {
            gl.enable(glow::DEPTH_TEST);
            gl.depth_func(glow::LESS);
            gl.enable(glow::BLEND);
            gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        }

        Ok(Renderer {
            clear_color: None,
            should_show_fps: config.should_show_fps,
            draws: 0,
            polygons: 0,
            frame_history: Vec::with_capacity(FPS_FRAME_HISTORY_CNT),
            current_texture: None,
            current_program: Some(program),
            batched: Vec::with_capacity(BATCH_SIZE * QUAD_VERTEX_CNT),
            batched_cnt: 0,
            indices,
            vertex_buffer,
            index_buffer,
            vertex_array,
        })
    }

    pub fn draw_batch(&mut self) {
        if !self.batched.is_empty() {
            let mut camera = main_camera();

            let program = self
                .current_program
                .as_mut()
                .unwrap_or_else(|| panic!("ERROR: No shader program set on renderer!"));

            program.activate();

            let texture = &self
                .current_texture
                .unwrap_or_else(|| panic!("ERROR: No texture set on renderer!"));

            texture.bind(TextureUnit::Texture0);

            let index_cnt = self.batched_cnt * QUAD_INDEX_CNT;

            self.index_buffer.set_data(0, &self.indices[0..index_cnt]);

            let model = Mat4::IDENTITY;

            let viewport = viewport();
            let view =
                Mat4::orthographic_rh_gl(0.0, viewport.width, viewport.height, 0.0, -1.0, 1.0);

            let projection = camera.projection();

            program.set_uniform_mat4("mvp", false, model * view * projection);

            self.vertex_buffer.set_data(0, &self.batched);

            self.vertex_array.bind();

            let gl = gl_context();
            unsafe {
                gl.viewport(
                    viewport.x as i32,
                    viewport.y as i32,
                    viewport.width as i32,
                    viewport.height as i32,
                );

                gl.draw_elements(
                    glow::TRIANGLES,
                    self.batched_cnt as i32 * QUAD_INDEX_CNT as i32,
                    glow::UNSIGNED_INT,
                    0,
                );

                // gl.draw_arrays(glow::TRIANGLES, 0, self.batched_cnt as i32);

                gl.bind_texture(glow::TEXTURE_2D, None);

                gl.bind_vertex_array(None);

                // gl.bind_buffer(glow::ARRAY_BUFFER, None);
                // gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

                gl.use_program(None);
            }

            self.draws += 1;
            self.polygons += self.batched_cnt as u32 * 2;

            self.batched.clear();
            self.batched_cnt = 0;
        }
    }

    pub fn use_program(&mut self, mut program: ShaderProgram) {
        if self.current_program.is_none() || self.current_program.unwrap() != program {
            self.draw_batch();

            program.activate();

            self.current_program = Some(program);
        }
    }

    pub fn set_clear_color(&mut self, clear_color: Color) {
        self.clear_color = Some(clear_color);

        let gl = gl_context();
        unsafe {
            gl.clear_color(
                clear_color.red,
                clear_color.green,
                clear_color.blue,
                clear_color.alpha,
            );
        }
    }

    pub fn clear_screen<C: Into<Option<Color>>>(&mut self, clear_color: C) {
        if let Some(clear_color) = clear_color.into() {
            self.set_clear_color(clear_color);
        }

        let gl = gl_context();
        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        };
    }

    pub fn draw_texture(&mut self, x: f32, y: f32, texture: Texture2D, params: DrawTextureParams) {
        if let Some(current_texture) = self.current_texture {
            if current_texture != texture {
                self.current_texture = Some(texture);
                self.draw_batch();
            }
        }

        self.current_texture = Some(texture);

        if self.batched_cnt >= BATCH_SIZE {
            self.draw_batch();
        }

        let texture_size = texture.size();

        let source_rect = params
            .source
            .unwrap_or_else(|| Rect::new(0.0, 0.0, texture_size.width, texture_size.height));

        let size = params
            .dest_size
            .unwrap_or_else(|| source_rect.size().into());

        let mut uv_rect = Rect::new(
            source_rect.x / texture_size.width,
            source_rect.y / texture_size.height,
            source_rect.width / texture_size.width,
            source_rect.height / texture_size.height,
        );

        if params.flip_x {
            uv_rect.x += uv_rect.width;
            uv_rect.width = -uv_rect.width;
        }

        if params.flip_y {
            uv_rect.y += uv_rect.height;
            uv_rect.height = -uv_rect.height;
        }

        self.batched.push(Vertex::new(
            vec2(x, y),
            params.tint,
            vec2(uv_rect.x, uv_rect.y),
        ));

        self.batched.push(Vertex::new(
            vec2(x + size.width, y),
            params.tint,
            vec2(uv_rect.x + uv_rect.width, uv_rect.y),
        ));

        self.batched.push(Vertex::new(
            vec2(x, y + size.height),
            params.tint,
            vec2(uv_rect.x, uv_rect.y + uv_rect.height),
        ));

        self.batched.push(Vertex::new(
            vec2(x + size.width, y + size.height),
            params.tint,
            vec2(uv_rect.x + uv_rect.width, uv_rect.y + uv_rect.height),
        ));

        self.batched_cnt += 1;
    }

    pub fn end_frame(&mut self) -> Result<()> {
        while self.frame_history.len() >= FPS_FRAME_HISTORY_CNT - 1 {
            self.frame_history.remove(0);
        }

        self.frame_history.push(draw_delta_time());

        self.draw_batch();

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

        draw_queued_text()?;

        get_context_wrapper().swap_buffers()?;

        get_window().request_redraw();

        self.polygons = 0;
        self.draws = 0;

        Ok(())
    }

    pub fn fps(&self) -> u32 {
        let mut total = 0.0;
        for t in &self.frame_history {
            total += t.as_secs_f32();
        }

        (1.0 / (total / self.frame_history.len() as f32)).round() as u32
    }

    pub fn apply_config(&mut self, config: &VideoConfig) {
        self.should_show_fps = config.should_show_fps;
    }
}

#[cfg(feature = "2d-renderer")]
static mut RENDERER: Option<Renderer> = None;

#[cfg(feature = "2d-renderer")]
pub fn init_renderer(config: &VideoConfig) -> Result<()> {
    let renderer = Renderer::new(config)?;
    unsafe { RENDERER = Some(renderer) }
    Ok(())
}

#[cfg(feature = "2d-renderer")]
pub fn renderer() -> &'static mut Renderer {
    unsafe {
        RENDERER.as_mut().unwrap_or_else(|| {
            panic!("ERROR: Attempted to access renderer but it has not been initialized yet!")
        })
    }
}
