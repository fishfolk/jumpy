use crate::math::{Size, URect};
use serde::{Deserialize, Serialize};

use super::{vec2, Vec2};

/// A 2D rectangle, defined by its top-left corner, width and height.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    #[serde(alias = "w")]
    pub width: f32,
    #[serde(alias = "h")]
    pub height: f32,
}

impl Rect {
    /// Creates a new rectangle from its top-left corner, width and height.
    ///
    /// # Arguments:
    ///   * `x` - x-coordinate of the top-left corner.
    ///   * `y` - y-coordinate of the top-left corner.
    ///   * `w` - width of the `Rect`, going to the right.
    ///   * `h` - height of the `Rect`, going down.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Rect {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the top-left corner of the `Rect`.
    pub fn point(&self) -> Vec2 {
        vec2(self.x, self.y)
    }

    /// Returns the size (width and height) of the `Rect`.
    pub fn size(&self) -> Vec2 {
        vec2(self.width, self.height)
    }

    /// Returns the left edge of the `Rect`
    pub fn left(&self) -> f32 {
        self.x
    }

    /// Returns the right edge of the `Rect`
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Returns the top edge of the `Rect`
    pub fn top(&self) -> f32 {
        self.y
    }

    /// Returns the bottom edge of the `Rect`
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }

    /// Moves the `Rect`'s origin to (x, y)
    pub fn move_to(&mut self, destination: Vec2) {
        self.x = destination.x;
        self.y = destination.y;
    }

    /// Scales the `Rect` by a factor of (sx, sy),
    /// growing towards the bottom-left
    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.width *= sx;
        self.height *= sy;
    }

    /// Checks whether the `Rect` contains a `Point`
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.left()
            && point.x < self.right()
            && point.y < self.bottom()
            && point.y >= self.top()
    }

    /// Checks whether the `Rect` overlaps another `Rect`
    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    /// Returns a new `Rect` that includes all points of these two `Rect`s.
    pub fn combine_with(self, other: Rect) -> Rect {
        let x = f32::min(self.x, other.x);
        let y = f32::min(self.y, other.y);
        let w = f32::max(self.right(), other.right()) - x;
        let h = f32::max(self.bottom(), other.bottom()) - y;
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    /// Returns an intersection rect there is any intersection
    pub fn intersect(&self, other: Rect) -> Option<Rect> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right < left || bottom < top {
            return None;
        }

        Some(Rect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        })
    }

    /// Translate rect origin be `offset` vector
    pub fn offset(self, offset: Vec2) -> Rect {
        Rect::new(
            self.x + offset.x,
            self.y + offset.y,
            self.width,
            self.height,
        )
    }
}

impl From<URect> for Rect {
    fn from(urect: URect) -> Self {
        Rect {
            x: urect.x as f32,
            y: urect.y as f32,
            width: urect.width as f32,
            height: urect.height as f32,
        }
    }
}

impl From<(Vec2, Size<f32>)> for Rect {
    fn from((pos, size): (Vec2, Size<f32>)) -> Self {
        Rect::new(pos.x, pos.y, size.width, size.height)
    }
}
