use glow::{HasContext, NativeProgram, NativeShader};
use std::io::Read;

use serde::{Deserialize, Serialize};

use crate::error::ErrorKind;
use crate::gl::gl_context;
use crate::prelude::renderer::renderer;
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

pub struct ShaderProgram {
    gl_program: NativeProgram,
}

impl ShaderProgram {
    pub fn new(&self, shaders: &[Shader]) -> Result<Self> {
        let gl = gl_context();
        let gl_program = unsafe {
            let program = gl.create_program()?;

            for shader in shaders {
                gl.attach_shader(self.gl_program, shader.gl_shader);
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
                gl.detach_shader(self.gl_program, shader.gl_shader);
            }

            program
        };

        Ok(ShaderProgram { gl_program })
    }

    pub fn set_active(&self) {
        let renderer = renderer();
        if renderer.current_program.is_none() || renderer.current_program.unwrap() != self {
            renderer.current_program = Some(self.gl_program);

            renderer.draw_batch();

            let gl = gl_context();
            unsafe { gl.use_program(Some(self.gl_program)) }
        }
    }
}

impl PartialEq for ShaderProgram {
    fn eq(&self, other: &Self) -> bool {
        self.gl_program == other.gl_program
    }
}

impl PartialEq<NativeProgram> for ShaderProgram {
    fn eq(&self, other: &NativeProgram) -> bool {
        self.gl_program == other
    }
}

impl Eq for ShaderProgram {}

impl Drop for ShaderProgram {
    fn drop(&mut self) {
        let gl = gl_context();
        unsafe {
            let renderer = renderer();
            if let Some(program) = renderer.current_program {
                if program == self {
                    renderer.draw_batch();

                    renderer.current_program = None;
                    gl.use_program(None);
                }
            }

            gl.delete_program(self.gl_program);
        }
    }
}
