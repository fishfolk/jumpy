use crate::camera::{cameras, main_camera};
use glam::{vec3, Mat4};
use glow::{Context, HasContext, NativeProgram, NativeTexture, NativeVertexArray};
use std::any::Any;
use std::time::Duration;

use crate::color::{colors, Color};
use crate::game::draw_delta_time;
use crate::gl::gl_context;
use crate::math::{vec2, Rect, Size, Vec2};
use crate::prelude::shader::ShaderKind;
use crate::prelude::vertex::VertexLayout;
use crate::prelude::viewport_size;
use crate::rendering::shader::{Uniform, UniformType};
use crate::rendering::vertex::{Index, VertexImpl};
use crate::rendering::{
    Buffer, DrawTextureParams, RenderTarget, Shader, ShaderProgram, Vertex, VertexArray,
};
use crate::text::{draw_queued_text, draw_text, HorizontalAlignment, TextParams};
use crate::texture::{Texture2D, TextureFilterMode, TextureUnit};
use crate::video::VideoConfig;
use crate::window::{get_context_wrapper, get_window};
use crate::Result;

const BATCH_SIZE: usize = 128;

const FPS_FRAME_HISTORY_CNT: usize = 32;

const QUAD_VERTEX_CNT: usize = 4;
const QUAD_INDEX_CNT: usize = 6;

const VERTEX_SHADER_SRC: &str = "
#version 410

layout(location = 0) in vec2 vertex_position;
layout(location = 1) in vec4 vertex_color;
layout(location = 2) in vec2 texture_coords;

uniform mat4 projection;
uniform mat4 model;

out vec4 color;
out vec2 uv;

void main() {
  color = vertex_color;
  uv = texture_coords;
  gl_Position = projection * model * vec4(vertex_position, 1.0, 1.0);
}
";

const FRAGMENT_SHADER_SRC: &str = "
#version 410

in vec4 color;
in vec2 uv;

uniform sampler2D base_texture;

void main() {
  gl_FragColor = texture(base_texture, uv) * color;
}
";

#[derive(Clone)]
pub struct RenderStats {
    pub fps: u32,
    pub frame_time: Vec<Duration>,
    pub quads: u32,
    pub draws: u32,
}

pub trait Renderer {
    fn use_program(&mut self, program: ShaderProgram);
    fn set_clear_color(&mut self, clear_color: Color);
    fn clear_screen(&self);
    fn draw_texture(&mut self, x: f32, y: f32, texture: Texture2D, params: DrawTextureParams);
    fn end_frame(&mut self) -> Result<()>;
    fn apply_config(&mut self, config: &VideoConfig);
    fn frame_time(&self) -> &[Duration];

    fn fps(&self) -> u32 {
        let mut total = 0.0;
        for t in self.frame_time() {
            total += t.as_secs_f32();
        }

        (1.0 / (total / self.frame_time().len() as f32)).round() as u32
    }
}

pub struct DefaultRenderer<V: VertexImpl> {
    clear_color: Option<Color>,
    current_texture: Option<Texture2D>,
    current_program: Option<ShaderProgram>,
    should_show_fps: bool,
    draws: u32,
    quads: u32,
    frame_time: Vec<Duration>,
    batched: Vec<V>,
    batched_cnt: usize,
    indices: Vec<u32>,
    vertex_buffer: Buffer<V>,
    index_buffer: Buffer<Index>,
    vertex_array: VertexArray,
}

impl<V: VertexImpl> DefaultRenderer<V> {
    pub fn new(config: &VideoConfig) -> Result<Self> {
        let vertex_buffer = Buffer::new_vertex()?;
        let index_buffer = Buffer::new_element()?;

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

        let program = ShaderProgram::new(
            &[
                Shader::new(ShaderKind::Vertex, VERTEX_SHADER_SRC.as_bytes())?,
                Shader::new(ShaderKind::Fragment, FRAGMENT_SHADER_SRC.as_bytes())?,
            ],
            &[
                ("projection", UniformType::Mat4),
                ("model", UniformType::Mat4),
                ("base_texture", UniformType::Sampler2D),
            ],
        )?;

        let vertex_array = VertexArray::new::<V>()?;

        Ok(DefaultRenderer {
            clear_color: None,
            should_show_fps: config.should_show_fps,
            draws: 0,
            quads: 0,
            frame_time: Vec::with_capacity(FPS_FRAME_HISTORY_CNT),
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
            let camera = main_camera();

            let program = self
                .current_program
                .as_mut()
                .unwrap_or_else(|| panic!("ERROR: No shader program set on renderer!"));

            program.activate();

            let projection = camera.projection();
            program.set_uniform_mat4("projection", false, projection);

            let model = Mat4::IDENTITY;
            program.set_uniform_mat4("model", false, model);

            self.vertex_array.bind();

            self.vertex_buffer.bind();
            self.vertex_buffer.set_data(&self.batched);

            let index_cnt = self.batched_cnt * QUAD_INDEX_CNT;

            self.index_buffer.bind();
            self.index_buffer.set_data(&self.indices[0..index_cnt]);

            let texture = &self
                .current_texture
                .unwrap_or_else(|| panic!("ERROR: No texture set on renderer!"));

            texture.bind(TextureUnit::Texture0);

            program.set_uniform_sampler_2d("base_texture", TextureUnit::Texture0);

            let gl = gl_context();
            unsafe {
                /*
                gl.draw_elements(
                    glow::TRIANGLES,
                    self.batched_cnt as i32,
                    glow::UNSIGNED_INT,
                    0,
                );
                */

                gl.draw_arrays(glow::TRIANGLES, 0, self.batched_cnt as i32);

                gl.bind_texture(glow::TEXTURE_2D, None);

                for i in 0..self.vertex_array.attr_cnt() {
                    gl.disable_vertex_attrib_array(i as u32);
                }

                gl.bind_vertex_array(None);

                gl.bind_buffer(glow::ARRAY_BUFFER, None);
                gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);

                gl.use_program(None);
            }

            self.draws += 1;
            self.quads += self.batched_cnt as u32;

            self.batched.clear();
            self.batched_cnt = 0;
        }
    }
}

impl<V: VertexImpl> Renderer for DefaultRenderer<V> {
    fn use_program(&mut self, mut program: ShaderProgram) {
        if self.current_program.is_none() || self.current_program.unwrap() != program {
            self.draw_batch();

            program.activate();

            self.current_program = Some(program);
        }
    }

    fn set_clear_color(&mut self, clear_color: Color) {
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

    fn clear_screen(&self) {
        let gl = gl_context();
        unsafe {
            gl.clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);
        };
    }

    fn draw_texture(&mut self, x: f32, y: f32, texture: Texture2D, params: DrawTextureParams) {
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

        let texture_rect = params.source.unwrap_or_else(|| {
            let size = texture.size();
            Rect::new(0.0, 0.0, size.width as f32, size.height as f32)
        });

        let size = params.dest_size.unwrap_or_else(|| texture.size());

        let color = params.tint.unwrap_or_else(|| colors::WHITE);

        self.batched.push(V::new(
            vec3(x, y, 1.0),
            color,
            vec2(texture_rect.x, texture_rect.y),
        ));

        self.batched.push(V::new(
            vec3(x + size.width, y, 1.0),
            color,
            vec2(texture_rect.x + texture_rect.width, texture_rect.y),
        ));

        self.batched.push(V::new(
            vec3(x, y + size.height, 1.0),
            color,
            vec2(texture_rect.x, texture_rect.y + texture_rect.height),
        ));

        self.batched.push(V::new(
            vec3(x + size.width, y + size.height, 1.0),
            color,
            vec2(
                texture_rect.x + texture_rect.width,
                texture_rect.y + texture_rect.height,
            ),
        ));

        self.batched_cnt += 1;
    }

    fn end_frame(&mut self) -> Result<()> {
        while self.frame_time.len() >= FPS_FRAME_HISTORY_CNT - 1 {
            self.frame_time.remove(0);
        }

        self.frame_time.push(draw_delta_time());

        self.draw_batch();

        let viewport_size = viewport_size();

        let mut should_show_fps = self.should_show_fps;

        #[cfg(debug_assertions)]
        {
            draw_text(
                "quads:\ndraws:",
                viewport_size.width - 150.0,
                70.0,
                TextParams {
                    bounds: Some(Size::new(75.0, 100.0)),
                    ..Default::default()
                },
            );

            draw_text(
                &format!("{}\n{}", self.quads, self.draws),
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
                viewport_size.width - 150.0,
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

        self.quads = 0;
        self.draws = 0;

        Ok(())
    }

    fn apply_config(&mut self, config: &VideoConfig) {
        self.should_show_fps = config.should_show_fps;
    }

    fn frame_time(&self) -> &[Duration] {
        &self.frame_time
    }
}

static mut RENDERER: Option<Box<dyn Renderer>> = None;

pub fn init_renderer<V: 'static + VertexImpl>(config: &VideoConfig) -> Result<()> {
    let renderer = DefaultRenderer::<V>::new(config)?;
    unsafe { RENDERER = Some(Box::new(renderer)) }
    Ok(())
}

pub fn renderer() -> &'static mut Box<dyn Renderer> {
    unsafe {
        RENDERER.as_mut().unwrap_or_else(|| {
            panic!("ERROR: Attempted to access renderer but it has not been initialized yet!")
        })
    }
}
