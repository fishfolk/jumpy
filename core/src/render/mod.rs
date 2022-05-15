pub mod render_target;

pub use crate::backend_impl::render::*;
use crate::color::Color;
use crate::math::{Rect, Size, Vec2};
pub use render_target::RenderTarget;

#[derive(Debug, Default, Clone)]
pub struct DrawTextureParams {
    pub tint: Option<Color>,

    pub dest_size: Option<Size<f32>>,

    /// Part of texture to draw. If None - draw the whole texture.
    /// Good use example: drawing an image from texture atlas.
    /// Is None by default
    pub source: Option<Rect>,

    /// Rotation in radians
    pub rotation: f32,

    /// Mirror on the X axis
    pub flip_x: bool,

    /// Mirror on the Y axis
    pub flip_y: bool,

    /// Rotate around this point.
    /// When `None`, rotate around the texture's center.
    /// When `Some`, the coordinates are in screen-space.
    /// E.g. pivot (0,0) rotates around the top left corner of the screen, not of the
    /// texture.
    pub pivot: Option<Vec2>,
}
