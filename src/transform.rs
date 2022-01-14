use macroquad::prelude::*;

#[derive(Debug, Default)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
}

impl Transform {
    pub fn new(position: Vec2, rotation: f32) -> Self {
        Transform { position, rotation }
    }
}

impl From<Vec2> for Transform {
    fn from(position: Vec2) -> Self {
        Transform {
            position,
            rotation: 0.0,
        }
    }
}
