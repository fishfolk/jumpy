//! Re-usable math utilities

use crate::prelude::*;

#[derive(Debug, Copy, Clone, Pod, Zeroable, Default)]
#[repr(C)]
pub struct Rect {
    pub min: Vec2,
    pub max: Vec2,
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        let half_size = vec2(width / 2.0, height / 2.0);
        let min = vec2(x, y) - half_size;
        let max = min + vec2(width, height);
        Self { min, max }
    }

    #[inline]
    pub fn width(&self) -> f32 {
        self.max.x - self.min.x
    }

    #[inline]
    pub fn height(&self) -> f32 {
        self.max.y - self.min.y
    }

    #[inline]
    pub fn size(&self) -> Vec2 {
        vec2(self.width(), self.height())
    }

    #[inline]
    pub fn left(&self) -> f32 {
        self.min.x
    }

    #[inline]
    pub fn right(&self) -> f32 {
        self.max.x
    }

    #[inline]
    pub fn top(&self) -> f32 {
        self.max.y
    }

    #[inline]
    pub fn bottom(&self) -> f32 {
        self.min.y
    }

    #[inline]
    pub fn top_left(&self) -> Vec2 {
        vec2(self.min.x, self.max.y)
    }

    #[inline]
    pub fn top_right(&self) -> Vec2 {
        vec2(self.max.x, self.max.y)
    }

    #[inline]
    pub fn bottom_left(&self) -> Vec2 {
        vec2(self.min.x, self.min.y)
    }

    #[inline]
    pub fn bottom_right(&self) -> Vec2 {
        vec2(self.max.x, self.min.y)
    }

    pub fn overlaps(&self, other: &Rect) -> bool {
        self.left() <= other.right()
            && self.right() >= other.left()
            && self.top() >= other.bottom()
            && self.bottom() <= other.top()
    }

    pub fn contains(&self, point: Vec2) -> bool {
        point.x <= self.right()
            && point.x >= self.left()
            && point.y >= self.bottom()
            && point.y <= self.top()
    }

    #[inline]
    pub fn center(&self) -> Vec2 {
        let half_size = self.size() / 2.0;
        self.min + half_size
    }

    pub fn min(&self) -> Vec2 {
        self.min
    }

    pub fn max(&self) -> Vec2 {
        self.max
    }
}
