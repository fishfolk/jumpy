// FIXME: This is very ugly, and shouldn't be passed into the editor state as parameter. Is there some
// better way to do this?
pub struct LevelView {
    /// The view offset in pixels.
    pub position: macroquad::prelude::Vec2,
    /// The scale the level is viewed with. 1.0 == 1:1, bigger numbers mean bigger tiles.
    pub scale: f32,
}
