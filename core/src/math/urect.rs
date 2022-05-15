use serde::{Deserialize, Serialize};

use super::{uvec2, Rect, UVec2};

#[derive(Debug, Copy, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct URect {
    pub x: u32,
    pub y: u32,
    #[serde(alias = "w")]
    pub width: u32,
    #[serde(alias = "h")]
    pub height: u32,
}

impl URect {
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        URect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn point(&self) -> UVec2 {
        uvec2(self.x, self.y)
    }

    pub fn size(&self) -> UVec2 {
        uvec2(self.width, self.height)
    }

    /// Returns the left edge of the `Rect`
    pub fn left(&self) -> u32 {
        self.x
    }

    /// Returns the right edge of the `Rect`
    pub fn right(&self) -> u32 {
        self.x + self.width
    }

    /// Returns the top edge of the `Rect`
    pub fn top(&self) -> u32 {
        self.y
    }

    /// Returns the bottom edge of the `Rect`
    pub fn bottom(&self) -> u32 {
        self.y + self.height
    }

    /// Moves the `Rect`'s origin to (x, y)
    pub fn move_to(&mut self, destination: UVec2) {
        self.x = destination.x;
        self.y = destination.y;
    }

    /// Scales the `Rect` by a factor of (sx, sy),
    /// growing towards the bottom-left
    pub fn scale(&mut self, sx: u32, sy: u32) {
        self.width *= sx;
        self.height *= sy;
    }

    /// Checks whether the `Rect` contains a `Point`
    pub fn contains(&self, point: UVec2) -> bool {
        point.x >= self.left()
            && point.x < self.right()
            && point.y < self.bottom()
            && point.y >= self.top()
    }

    /// Checks whether the `Rect` overlaps another `Rect`
    pub fn overlaps(&self, other: &URect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() <= other.bottom()
            && self.bottom() >= other.top()
    }

    /// Returns a new `Rect` that includes all points of these two `Rect`s.
    #[must_use]
    pub fn combine_with(self, other: URect) -> Self {
        let x = u32::min(self.x, other.x);
        let y = u32::min(self.y, other.y);
        let w = u32::max(self.right(), other.right()) - x;
        let h = u32::max(self.bottom(), other.bottom()) - y;
        URect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    /// Returns an intersection rect there is any intersection
    pub fn intersect(&self, other: URect) -> Option<Self> {
        let left = self.x.max(other.x);
        let top = self.y.max(other.y);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());

        if right < left || bottom < top {
            return None;
        }

        Some(URect {
            x: left,
            y: top,
            width: right - left,
            height: bottom - top,
        })
    }

    /// Translate rect origin be `offset` vector
    #[must_use]
    pub fn offset(self, offset: UVec2) -> Self {
        URect::new(
            self.x + offset.x,
            self.y + offset.y,
            self.width,
            self.height,
        )
    }
}

impl From<Rect> for URect {
    fn from(rect: Rect) -> Self {
        URect {
            x: rect.x as u32,
            y: rect.y as u32,
            width: rect.width as u32,
            height: rect.height as u32,
        }
    }
}

impl From<(UVec2, UVec2)> for URect {
    fn from((position, size): (UVec2, UVec2)) -> Self {
        URect {
            x: position.x,
            y: position.y,
            width: size.x,
            height: size.y,
        }
    }
}
