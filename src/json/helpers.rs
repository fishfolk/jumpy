use macroquad::prelude::*;

pub fn vec2_is_zero(val: &Vec2) -> bool {
    *val == Vec2::ZERO
}

pub fn uvec2_is_zero(val: &UVec2) -> bool {
    *val == UVec2::ZERO
}

pub fn is_false(val: &bool) -> bool {
    !*val
}
