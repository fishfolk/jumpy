use glow::{HasContext, NativeProgram, NativeShader};
use std::collections::HashMap;
use std::io::Read;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::error::ErrorKind;
use crate::gl::gl_context;
use crate::prelude::renderer::renderer;
use crate::rendering::renderer::Renderer;
use crate::Result;

#[derive(Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShaderKind {
    Vertex,
    TesselationControl,
    TesselationEval,
    Geometry,
    Fragment,
    Compute,
}

impl From<ShaderKind> for u32 {
    fn from(kind: ShaderKind) -> Self {
        match kind {
            ShaderKind::Vertex => glow::VERTEX_SHADER,
            ShaderKind::TesselationControl => glow::TESS_CONTROL_SHADER,
            ShaderKind::TesselationEval => glow::TESS_EVALUATION_SHADER,
            ShaderKind::Geometry => glow::GEOMETRY_SHADER,
            ShaderKind::Fragment => glow::FRAGMENT_SHADER,
            ShaderKind::Compute => glow::COMPUTE_SHADER,
        }
    }
}

impl From<u32> for ShaderKind {
    fn from(kind: u32) -> Self {
        match kind {
            glow::VERTEX_SHADER => Self::Vertex,
            glow::TESS_CONTROL_SHADER => Self::TesselationControl,
            glow::TESS_EVALUATION_SHADER => Self::TesselationEval,
            glow::GEOMETRY_SHADER => Self::Geometry,
            glow::FRAGMENT_SHADER => Self::Fragment,
            glow::COMPUTE_SHADER => Self::Compute,
            _ => panic!("ERROR: Invalid shader type '{}'", kind),
        }
    }
}

pub struct Shader {
    pub kind: ShaderKind,
    gl_shader: NativeShader,
}

impl Shader {
    pub fn new(kind: ShaderKind, mut src: &[u8]) -> Result<Self> {
        let mut bytes = "".to_string();
        src.read_to_string(&mut bytes);

        let gl = gl_context();
        let gl_shader = unsafe {
            let shader = gl.create_shader(kind.into())?;

            gl.shader_source(shader, &bytes);

            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                let err = gl.get_shader_info_log(shader);
                return Err(formaterr!(
                    ErrorKind::Shader,
                    "Unable to compile shader: {}",
                    err
                ));
            }

            shader
        };

        Ok(Shader { kind, gl_shader })
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        let gl = gl_context();
        unsafe {
            gl.delete_shader(self.gl_shader);
        }
    }
}

pub struct ShaderProgramImpl {
    gl_program: NativeProgram,
}

impl ShaderProgramImpl {
    pub fn gl_program(&self) -> NativeProgram {
        self.gl_program
    }
}

impl PartialEq for ShaderProgramImpl {
    fn eq(&self, other: &Self) -> bool {
        self.gl_program == other.gl_program
    }
}

impl PartialEq<NativeProgram> for ShaderProgramImpl {
    fn eq(&self, other: &NativeProgram) -> bool {
        self.gl_program == *other
    }
}

impl Eq for ShaderProgramImpl {}

impl Drop for ShaderProgramImpl {
    fn drop(&mut self) {
        let gl = gl_context();
        unsafe {
            gl.delete_program(self.gl_program);
        }
    }
}

static mut NEXT_SHADER_INDEX: usize = 0;
static mut SHADERS: Option<HashMap<usize, ShaderProgramImpl>> = None;

fn shader_map() -> &'static mut HashMap<usize, ShaderProgramImpl> {
    unsafe { SHADERS.get_or_insert_with(HashMap::new) }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct ShaderProgram(usize);

impl ShaderProgram {
    pub fn new(shaders: &[Shader]) -> Result<Self> {
        let gl = gl_context();
        let gl_program = unsafe {
            let program = gl.create_program()?;

            for shader in shaders {
                gl.attach_shader(program, shader.gl_shader);
            }

            gl.link_program(program);
            if !gl.get_program_link_status(program) {
                let err = gl.get_program_info_log(program);
                return Err(formaterr!(
                    ErrorKind::Shader,
                    "Unable to link shader program: {}",
                    err
                ));
            }

            for shader in shaders {
                gl.detach_shader(program, shader.gl_shader);
            }

            program
        };

        let index = unsafe {
            let index = NEXT_SHADER_INDEX;
            NEXT_SHADER_INDEX += 1;
            index
        };

        shader_map().insert(index, ShaderProgramImpl { gl_program });

        Ok(ShaderProgram(index))
    }
}

impl Deref for ShaderProgram {
    type Target = ShaderProgramImpl;

    fn deref(&self) -> &Self::Target {
        shader_map().get(&self.0).unwrap()
    }
}

impl DerefMut for ShaderProgram {
    fn deref_mut(&mut self) -> &mut Self::Target {
        shader_map().get_mut(&self.0).unwrap()
    }
}
