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

use std::f32::consts::PI;

/// Simple easing calculator
pub struct Ease {
    pub ease_in: bool,
    pub ease_out: bool,
    pub function: EaseFunction,
    pub progress: f32,
}

pub enum EaseFunction {
    Quadratic,
    Cubic,
    Sinusoidial,
}

impl Default for Ease {
    fn default() -> Self {
        Self {
            ease_in: true,
            ease_out: true,
            function: EaseFunction::Quadratic,
            progress: 0.0,
        }
    }
}

impl Ease {
    pub fn output(&self) -> f32 {
        let mut k = self.progress;

        // Reference for easing functions:
        // https://echarts.apache.org/examples/en/editor.html?c=line-easing&lang=ts
        //
        // TODO: Add tests to make sure easings are correct
        match (&self.function, self.ease_in, self.ease_out) {
            (EaseFunction::Quadratic, true, true) => {
                k *= 2.0;
                if k < 1.0 {
                    0.5 * k * k
                } else {
                    k -= 1.0;
                    -0.5 * (k * (k - 2.0) - 1.0)
                }
            }
            (EaseFunction::Quadratic, true, false) => k * k,
            (EaseFunction::Quadratic, false, true) => k * (2.0 - k),
            (EaseFunction::Cubic, true, true) => {
                k *= 2.0;
                if k < 1.0 {
                    0.5 * k * k * k
                } else {
                    k -= 2.0;
                    0.5 * (k * k * k + 2.0)
                }
            }
            (EaseFunction::Cubic, true, false) => k * k * k,
            (EaseFunction::Cubic, false, true) => {
                k -= 1.0;
                k * k * k + 1.0
            }
            (EaseFunction::Sinusoidial, true, true) => 0.5 * (1.0 - f32::cos(PI * k)),
            (EaseFunction::Sinusoidial, true, false) => 1.0 - f32::cos((k * PI) / 2.0),
            (EaseFunction::Sinusoidial, false, true) => f32::sin((k * PI) / 2.0),
            (_, false, false) => k,
        }
    }
}
