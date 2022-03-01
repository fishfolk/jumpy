use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

pub use num_traits::*;

pub use crate::backend_impl::math::*;

use crate::color::Color;
use crate::video::VideoMode;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Size<T: Num> {
    pub width: T,
    pub height: T,
}

impl<T> Size<T> where T: Num {
    pub fn new(width: T, height: T) -> Self {
        Size {
            width,
            height,
        }
    }
}

impl<T> From<(T, T)> for Size<T> where T: Num + Copy {
    fn from(tpl: (T, T)) -> Self {
        Size::new(tpl.0, tpl.1)
    }
}

impl<T> From<Size<T>> for (T, T) where T: Num + Copy {
    fn from(size: Size<T>) -> Self {
        (size.width, size.height)
    }
}

impl<T> From<&[T; 2]> for Size<T> where T: Num + Copy {
    fn from(slice: &[T; 2]) -> Self {
    Size::new(slice[0], slice[1])
}
}


impl<T> From<&Size<T>> for [T; 2] where T: Num + Copy {
    fn from(size: &Size<T>) -> Self {
        [size.width, size.height]
    }
}

impl From<IVec2> for Size<i32> {
    fn from(vec: IVec2) -> Self {
        Size::new(vec.x, vec.y)
    }
}

impl From<UVec2> for Size<u32> {
    fn from(vec: UVec2) -> Self {
        Size::new(vec.x, vec.y)
    }
}

impl From<Vec2> for Size<f32> {
    fn from(vec: Vec2) -> Self {
        Size::new(vec.x, vec.y)
    }
}

impl From<Size<i32>> for IVec2 {
    fn from(size: Size<i32>) -> Self {
        ivec2(size.width, size.height)
    }
}

impl From<Size<u32>> for UVec2 {
    fn from(size: Size<u32>) -> Self {
        uvec2(size.width, size.height)
    }
}

impl From<Size<f32>> for Vec2 {
    fn from(size: Size<f32>) -> Self {
        vec2(size.width, size.height)
    }
}

cfg_if! {
    if #[cfg(feature = "internal-backend")] {
        impl<T> From<winit::dpi::PhysicalSize<T>> for Size<T> where T: Num {
            fn from(size: winit::dpi::PhysicalSize<T>) -> Self {
                Size::new(size.width, size.height)
            }
        }

        impl<T> From<Size<T>> for winit::dpi::PhysicalSize<T> where T: Num {
            fn from(size: Size<T>) -> Self {
                winit::dpi::PhysicalSize::new(size.width, size.height)
            }
        }
    }
}


#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct URect {
    pub x: u32,
    pub y: u32,
    #[serde(rename = "width", alias = "w")]
    pub w: u32,
    #[serde(rename = "height", alias = "h")]
    pub h: u32,
}

impl URect {
    pub fn new(x: u32, y: u32, w: u32, h: u32) -> Self {
        URect { x, y, w, h }
    }

    pub fn point(&self) -> UVec2 {
        uvec2(self.x, self.y)
    }

    pub fn size(&self) -> UVec2 {
        uvec2(self.w, self.h)
    }

    /// Returns the left edge of the `Rect`
    pub fn left(&self) -> u32 {
        self.x
    }

    /// Returns the right edge of the `Rect`
    pub fn right(&self) -> u32 {
        self.x + self.w
    }

    /// Returns the top edge of the `Rect`
    pub fn top(&self) -> u32 {
        self.y
    }

    /// Returns the bottom edge of the `Rect`
    pub fn bottom(&self) -> u32 {
        self.y + self.h
    }

    /// Moves the `Rect`'s origin to (x, y)
    pub fn move_to(&mut self, destination: UVec2) {
        self.x = destination.x;
        self.y = destination.y;
    }

    /// Scales the `Rect` by a factor of (sx, sy),
    /// growing towards the bottom-left
    pub fn scale(&mut self, sx: u32, sy: u32) {
        self.w *= sx;
        self.h *= sy;
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
        URect { x, y, w, h }
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
            w: right - left,
            h: bottom - top,
        })
    }

    /// Translate rect origin be `offset` vector
    #[must_use]
    pub fn offset(self, offset: UVec2) -> Self {
        URect::new(self.x + offset.x, self.y + offset.y, self.w, self.h)
    }
}

impl From<Rect> for URect {
    fn from(rect: Rect) -> Self {
        URect {
            x: rect.x as u32,
            y: rect.y as u32,
            w: rect.w as u32,
            h: rect.h as u32,
        }
    }
}

impl From<(UVec2, UVec2)> for URect {
    fn from((position, size): (UVec2, UVec2)) -> Self {
        URect {
            x: position.x,
            y: position.y,
            w: size.x,
            h: size.y,
        }
    }
}

impl From<URect> for Rect {
    fn from(other: URect) -> Rect {
        Rect {
            x: other.x as f32,
            y: other.y as f32,
            w: other.w as f32,
            h: other.h as f32,
        }
    }
}

pub fn rotate_vector(vec: Vec2, rad: f32) -> Vec2 {
    let sa = rad.sin();
    let ca = rad.cos();
    vec2(ca * vec.x - sa * vec.y, sa * vec.x + ca * vec.y)
}

pub fn deg_to_rad(deg: f32) -> f32 {
    deg * std::f32::consts::PI / 180.0
}

pub fn rad_to_deg(rad: f32) -> f32 {
    (rad * 180.0) / std::f32::consts::PI
}
