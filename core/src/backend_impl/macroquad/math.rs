pub use macroquad::math::*;

use crate::math::{AsIVec2, AsUVec2, AsVec2};

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
