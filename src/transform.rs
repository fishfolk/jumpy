use macroquad::prelude::*;

#[derive(Debug, Default)]
pub struct Transform {
    pub position: Vec2,
    pub rotation: f32,
}

impl From<Vec2> for Transform {
    fn from(position: Vec2) -> Self {
        Transform {
            position,
            rotation: 0.0,
        }
    }
}
