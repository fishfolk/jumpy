mod uniform;

use glam::{IVec2, IVec3, IVec4, Mat2, Mat3, Mat4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};
use glow::{HasContext, NativeProgram, NativeShader};
use std::collections::HashMap;
use std::io::Read;
use std::ops::{Deref, DerefMut};

use serde::{Deserialize, Serialize};

use crate::color::Color;
pub use uniform::*;

use crate::error::ErrorKind;
use crate::gl::gl_context;
use crate::prelude::renderer::renderer;
use crate::rendering::renderer::Renderer;
use crate::result::Result;
use crate::texture::{Texture2D, TextureUnit};

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
    uniforms: HashMap<String, Uniform>,
}

impl ShaderProgramImpl {
    pub fn new(gl_program: NativeProgram, uniforms: &[(&str, UniformType)]) -> Self {
        ShaderProgramImpl {
            gl_program,
            uniforms: HashMap::from_iter(
                uniforms
                    .into_iter()
                    .map(|&(k, v)| (k.to_string(), Uniform::new(v))),
            ),
        }
    }

    pub fn gl_program(&self) -> NativeProgram {
        self.gl_program
    }

    pub fn activate(&mut self) {
        let gl = gl_context();
        unsafe {
            gl.use_program(Some(self.gl_program()));

            for (name, uniform) in &mut self.uniforms {
                let location = gl
                    .get_uniform_location(self.gl_program, name)
                    .unwrap_or_else(|| {
                        panic!("ERROR: Uniform location for '{}' could not be found!", name)
                    });

                uniform.location = Some(location);
            }
        }
    }

    pub fn set_uniform_f32(&self, name: &str, values: &[f32]) {
        assert!(
            values.len() > 0,
            "ERROR: Values must contain at least one element!"
        );

        let uniform = self
            .uniforms
            .get(name)
            .unwrap_or_else(|| panic!("ERROR: No uniform named '{}' exist!", name));

        assert!(
            uniform.value_type.is_float(),
            "ERROR: Uniform '{}' is not a float!",
            name
        );

        let location = uniform.location.unwrap();

        let gl = gl_context();

        unsafe {
            match values.len() {
                1 => gl.uniform_1_f32_slice(Some(&location), values),
                2 => gl.uniform_2_f32_slice(Some(&location), values),
                3 => gl.uniform_3_f32_slice(Some(&location), values),
                _ => gl.uniform_4_f32_slice(Some(&location), values),
            }
        }
    }

    pub fn set_uniform_i32(&self, name: &str, values: &[i32]) {
        assert!(
            values.len() > 0,
            "ERROR: Values must contain at least one element!"
        );

        let uniform = self
            .uniforms
            .get(name)
            .unwrap_or_else(|| panic!("ERROR: No uniform named '{}' exist!", name));

        assert!(
            uniform.value_type.is_int(),
            "ERROR: Uniform '{}' is not an int!",
            name
        );

        let location = uniform.location.unwrap();

        let gl = gl_context();

        unsafe {
            match values.len() {
                1 => gl.uniform_1_i32_slice(Some(&location), values),
                2 => gl.uniform_2_i32_slice(Some(&location), values),
                3 => gl.uniform_3_i32_slice(Some(&location), values),
                _ => gl.uniform_4_i32_slice(Some(&location), values),
            }
        }
    }

    pub fn set_uniform_u32(&self, name: &str, values: &[u32]) {
        assert!(
            values.len() > 0,
            "ERROR: Values must contain at least one element!"
        );

        let uniform = self
            .uniforms
            .get(name)
            .unwrap_or_else(|| panic!("ERROR: No uniform named '{}' exist!", name));

        assert!(
            uniform.value_type.is_uint(),
            "ERROR: Uniform '{}' is not a unsigned int!",
            name
        );

        let location = uniform.location.unwrap();

        let gl = gl_context();

        unsafe {
            match values.len() {
                1 => gl.uniform_1_u32_slice(Some(&location), values),
                2 => gl.uniform_2_u32_slice(Some(&location), values),
                3 => gl.uniform_3_u32_slice(Some(&location), values),
                4 => gl.uniform_4_u32_slice(Some(&location), values),
                _ => panic!("ERROR: Invalid amount of values!"),
            }
        }
    }

    pub fn set_uniform_vec2(&self, name: &str, vec: Vec2) {
        self.set_uniform_f32(name, &[vec.x, vec.y])
    }

    pub fn set_uniform_vec3(&self, name: &str, vec: Vec3) {
        self.set_uniform_f32(name, &[vec.x, vec.y, vec.z])
    }

    pub fn set_uniform_vec4(&self, name: &str, vec: Vec4) {
        self.set_uniform_f32(name, &[vec.x, vec.y, vec.z, vec.w])
    }

    pub fn set_uniform_ivec2(&self, name: &str, vec: IVec2) {
        self.set_uniform_i32(name, &[vec.x, vec.y])
    }

    pub fn set_uniform_ivec3(&self, name: &str, vec: IVec3) {
        self.set_uniform_i32(name, &[vec.x, vec.y, vec.z])
    }

    pub fn set_uniform_ivec4(&self, name: &str, vec: IVec4) {
        self.set_uniform_i32(name, &[vec.x, vec.y, vec.z, vec.w])
    }

    pub fn set_uniform_uvec2(&self, name: &str, vec: UVec2) {
        self.set_uniform_u32(name, &[vec.x, vec.y])
    }

    pub fn set_uniform_uvec3(&self, name: &str, vec: UVec3) {
        self.set_uniform_u32(name, &[vec.x, vec.y, vec.z])
    }

    pub fn set_uniform_uvec4(&self, name: &str, vec: UVec4) {
        self.set_uniform_u32(name, &[vec.x, vec.y, vec.z, vec.w])
    }

    pub fn set_uniform_color(&self, name: &str, color: Color) {
        self.set_uniform_f32(name, &[color.red, color.blue, color.green, color.alpha])
    }

    pub fn set_uniform_mat2(&self, name: &str, should_transpose: bool, mat: Mat2) {
        let uniform = self
            .uniforms
            .get(name)
            .unwrap_or_else(|| panic!("ERROR: No uniform named '{}' exist!", name));

        assert!(
            uniform.value_type.is_mat2(),
            "ERROR: Uniform '{}' is not a mat2!",
            name
        );

        let location = uniform.location.unwrap();

        let mut values = vec![0.0; 4];
        mat.write_cols_to_slice(&mut values);

        let gl = gl_context();
        unsafe {
            gl.uniform_matrix_2_f32_slice(Some(&location), should_transpose, &values);
        }
    }

    pub fn set_uniform_mat3(&self, name: &str, should_transpose: bool, mat: Mat3) {
        let uniform = self
            .uniforms
            .get(name)
            .unwrap_or_else(|| panic!("ERROR: No uniform named '{}' exist!", name));

        assert!(
            uniform.value_type.is_mat3(),
            "ERROR: Uniform '{}' is not a mat3!",
            name
        );

        let location = uniform.location.unwrap();

        let mut values = vec![0.0; 9];
        mat.write_cols_to_slice(&mut values);

        let gl = gl_context();
        unsafe {
            gl.uniform_matrix_3_f32_slice(Some(&location), should_transpose, &values);
        }
    }

    pub fn set_uniform_mat4(&self, name: &str, should_transpose: bool, mat: Mat4) {
        let uniform = self
            .uniforms
            .get(name)
            .unwrap_or_else(|| panic!("ERROR: No uniform named '{}' exist!", name));

        assert!(
            uniform.value_type.is_mat4(),
            "ERROR: Uniform '{}' is not a mat4!",
            name
        );

        let location = uniform.location.unwrap();

        let mut values = vec![0.0; 16];
        mat.write_cols_to_slice(&mut values);

        let gl = gl_context();
        unsafe {
            gl.uniform_matrix_4_f32_slice(Some(&location), should_transpose, &values);
        }
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
    pub fn new(shaders: &[Shader], uniforms: &[(&str, UniformType)]) -> Result<Self> {
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

        shader_map().insert(index, ShaderProgramImpl::new(gl_program, uniforms));

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
