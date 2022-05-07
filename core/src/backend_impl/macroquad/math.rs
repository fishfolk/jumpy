pub use macroquad::math::{
    ivec2, ivec3, ivec4, uvec2, uvec3, uvec4, vec2, vec3, vec4, IVec2, IVec3, IVec4, Mat2, Mat3,
    Mat4, RectOffset, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4,
};

use crate::math::{AsIVec2, AsUVec2, AsVec2, Circle, Rect};

impl AsVec2 for IVec2 {
    fn as_vec2(&self) -> Vec2 {
        self.as_f32()
    }
}

impl AsVec2 for UVec2 {
    fn as_vec2(&self) -> Vec2 {
        self.as_f32()
    }
}

impl AsIVec2 for Vec2 {
    fn as_ivec2(&self) -> IVec2 {
        self.as_i32()
    }
}

impl AsIVec2 for UVec2 {
    fn as_ivec2(&self) -> IVec2 {
        self.as_i32()
    }
}

impl AsUVec2 for Vec2 {
    fn as_uvec2(&self) -> UVec2 {
        self.as_u32()
    }
}

impl AsUVec2 for IVec2 {
    fn as_uvec2(&self) -> UVec2 {
        self.as_u32()
    }
}

impl From<Rect> for macroquad::math::Rect {
    fn from(rect: Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            w: rect.width,
            h: rect.height,
        }
    }
}

impl From<macroquad::math::Rect> for Rect {
    fn from(rect: macroquad::math::Rect) -> Self {
        Self {
            x: rect.x,
            y: rect.y,
            width: rect.w,
            height: rect.h,
        }
    }
}

impl From<Circle> for macroquad::math::Circle {
    fn from(circle: Circle) -> Self {
        Self {
            x: circle.x,
            y: circle.y,
            r: circle.radius,
        }
    }
}

impl From<macroquad::math::Circle> for Circle {
    fn from(circle: macroquad::math::Circle) -> Self {
        Self {
            x: circle.x,
            y: circle.y,
            radius: circle.r,
        }
    }
}
