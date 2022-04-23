pub struct LevelView {
    /// The view offset in pixels.
    pub position: macroquad::prelude::Vec2,
    /// The scale the level is viewed with. 1.0 == 1:1, bigger numbers mean bigger tiles.
    pub scale: f32,
}

impl Default for LevelView {
    fn default() -> Self {
        Self {
            position: Default::default(),
            scale: 1.,
        }
    }
}
