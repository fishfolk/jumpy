use crate::color::Color;
use crate::math::Vec2;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Color,
    pub texture_coords: Vec2,
}

pub type Index = u32;
