use crate::gl::gl_context;
use crate::{FLOAT_SIZE, INT_SIZE};
use glow::{HasContext, NativeUniformLocation};
use num_traits::Num;

#[derive(Copy, Clone)]
pub struct Uniform {
    pub value_type: UniformType,
    pub location: Option<NativeUniformLocation>,
}

impl Uniform {
    pub fn new(uniform_type: UniformType) -> Self {
        Uniform {
            value_type: uniform_type,
            location: None,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum UniformType {
    Sampler2D,
    Color,
    Mat2,
    Mat3,
    Mat4,
    Float,
    Vec2,
    Vec3,
    Vec4,
    Int,
    IVec2,
    IVec3,
    IVec4,
    UInt,
    UVec2,
    UVec3,
    UVec4,
}

impl UniformType {
    pub fn is_sampler_2d(&self) -> bool {
        matches!(self, Self::Sampler2D)
    }

    pub fn is_float(&self) -> bool {
        matches!(
            self,
            Self::Float
                | Self::Vec2
                | Self::Vec3
                | Self::Vec4
                | Self::Mat2
                | Self::Mat3
                | Self::Mat4
        )
    }

    pub fn is_mat(&self) -> bool {
        matches!(self, Self::Mat2 | Self::Mat3 | Self::Mat4)
    }

    pub fn is_mat2(&self) -> bool {
        matches!(self, Self::Mat2)
    }

    pub fn is_mat3(&self) -> bool {
        matches!(self, Self::Mat3)
    }

    pub fn is_mat4(&self) -> bool {
        matches!(self, Self::Mat4)
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Self::Int | Self::IVec2 | Self::IVec3 | Self::IVec4)
    }

    pub fn is_uint(&self) -> bool {
        matches!(self, Self::UInt | Self::UVec2 | Self::UVec3 | Self::UVec4)
    }
}
