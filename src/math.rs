use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

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

pub fn color_from_hex_string(str: &str) -> Color {
    let str = if str.starts_with('#') {
        str[1..str.len()].to_string()
    } else {
        str.to_string()
    };

    let r = u8::from_str_radix(&str[0..2], 16).unwrap();
    let g = u8::from_str_radix(&str[2..4], 16).unwrap();
    let b = u8::from_str_radix(&str[4..6], 16).unwrap();
    let a = if str.len() > 6 {
        u8::from_str_radix(&str[6..8], 16).unwrap()
    } else {
        255
    };

    Color::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex_string_no_hash() {
        assert_eq!(
            color_from_hex_string("12ab6f"),
            Color::new(
                18 as f32 / 255.0,
                171 as f32 / 255.0,
                111 as f32 / 255.0,
                255 as f32 / 255.0,
            )
        );
    }

    #[test]
    fn test_color_from_hex_string_hash() {
        assert_eq!(
            color_from_hex_string("#12ab6f"),
            Color::new(
                18 as f32 / 255.0,
                171 as f32 / 255.0,
                111 as f32 / 255.0,
                255 as f32 / 255.0,
            )
        );
    }

    #[test]
    fn test_color_from_hex_string_alpha() {
        assert_eq!(
            color_from_hex_string("12ab6fb2"),
            Color::new(
                18 as f32 / 255.0,
                171 as f32 / 255.0,
                111 as f32 / 255.0,
                178 as f32 / 255.0,
            )
        );
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

/// Use this in serde tags to skip serialization for zero values
pub trait IsZero {
    fn is_zero(&self) -> bool;
}

impl IsZero for f32 {
    fn is_zero(&self) -> bool {
        *self == 0.0
    }
}

impl IsZero for u32 {
    fn is_zero(&self) -> bool {
        *self == 0
    }
}

impl IsZero for Vec2 {
    fn is_zero(&self) -> bool {
        *self == Vec2::ZERO
    }
}
