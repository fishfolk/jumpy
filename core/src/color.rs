//! Color types and helpers.

use serde::{Serialize, Deserialize};

pub use crate::backend_impl::color::*;

pub use colors::*;

use crate::math::One;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Color {
    #[serde(rename = "red", alias = "r")]
    pub r: f32,
    #[serde(rename = "green", alias = "g")]
    pub g: f32,
    #[serde(rename = "blue", alias = "b")]
    pub b: f32,
    #[serde(default = "f32::one", rename = "alpha", alias = "a")]
    pub a: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Color { r, g, b, a }
    }

    pub fn from_bytes(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: a as f32 / 255.0,
        }
    }

    pub fn to_bytes(self) -> (u8, u8, u8, u8) {
        let r = self.r  * 255.0;
        let g = self.g  * 255.0;
        let b = self.b  * 255.0;
        let a = self.a  * 255.0;

        (r as u8, g as u8, b as u8, a as u8)
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let r;
        let g;
        let b;

        if s == 0.0 {  r = l; g = l; b = l; }
        else {
            let q = if l < 0.5 {
                l * (1.0 + s)
            } else {
                l + s - l * s
            };
            let p = 2.0 * l - q;
            r = hue_to_rgb(p, q, h + 1.0 / 3.0);
            g = hue_to_rgb(p, q, h);
            b = hue_to_rgb(p, q, h - 1.0 / 3.0);
        }

        Color::new(r, g, b, 1.0)
    }

    pub fn to_hsl(self) -> (f32, f32, f32) {
        let mut h: f32;
        let s: f32;
        let l: f32;

        let r = self.r;
        let g = self.g;
        let b = self.b;

        let min = r.min(g).min(b);
        let max = r.max(g).max(b);

        // Luminosity is the average of the max and min rgb color intensities.
        l = (max + min) / 2.0;

        // Saturation
        let delta: f32 = max - min;
        if delta == 0.0 {
            // it's gray
            return (0.0, 0.0, l);
        }

        // it's not gray
        if l < 0.5 {
            s = delta / (max + min);
        } else {
            s = delta / (2.0 - max - min);
        }

        // Hue
        let r2 = (((max - r) / 6.0) + (delta / 2.0)) / delta;
        let g2 = (((max - g) / 6.0) + (delta / 2.0)) / delta;
        let b2 = (((max - b) / 6.0) + (delta / 2.0)) / delta;

        h = match max {
            x if x == r => b2 - g2,
            x if x == g => (1.0 / 3.0) + r2 - b2,
            _ => (2.0 / 3.0) + g2 - r2,
        };

        // Fix wraparounds
        if h < 0 as f32 {
            h += 1.0;
        } else if h > 1 as f32 {
            h -= 1.0;
        }

        (h, s, l)
    }

    pub fn from_hex(str: &str) -> Color {
        let hex = if str.starts_with('#') {
            &str[1..str.len()]
        } else {
            str
        };

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();
        let a = if hex.len() > 6 {
            u8::from_str_radix(&hex[6..8], 16).unwrap()
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

    pub fn to_hex(self) -> String {
        let (r, g, b, _) = self.to_bytes();
        format!("{:X}{:X}{:X}", r, g, b)
    }

    pub fn to_hex_alpha(self) -> String {
        let (r, g, b, a) = self.to_bytes();
        format!("{:X}{:X}{:X}{:X}", r, g, b, a)
    }
}

impl Default for Color {
    fn default() -> Self {
        Color {
            r: 0.0,
            b: 0.0,
            g: 0.0,
            a: 1.0,
        }
    }
}

/// Build a color from 4 components of 0..255 values
/// This is a temporary solution and going to be replaced with const fn,
/// waiting for https://github.com/rust-lang/rust/issues/57241
#[macro_export]
macro_rules! color_u8 {
    ($r:expr, $g:expr, $b:expr, $a:expr) => {
        Color::new(
            $r as f32 / 255.0,
            $g as f32 / 255.0,
            $b as f32 / 255.0,
            $a as f32 / 255.0,
        )
    };

    ($r:expr, $g:expr, $b:expr) => {
        Color::new(
            $r as f32 / 255.0,
            $g as f32 / 255.0,
            $b as f32 / 255.0,
            1.0,
        )
    };
}

pub fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0 }
    if t > 1.0 { t -= 1.0 }
    if t < 1.0 / 6.0 { return p + (q - p) * 6.0 * t; }
    if t < 1.0 / 2.0 { return q; }
    if t < 2.0 / 3.0 { return p + (q - p) * (2.0 / 3.0 - t) * 6.0; }
    p
}

pub mod colors {
    //! Constants for some common colors.

    use super::Color;

    pub const LIGHT_GREY: Color = Color { r: 0.78, g: 0.78, b: 0.78, a: 1.00 };
    pub const GREY: Color = Color { r: 0.51, g: 0.51, b: 0.51, a: 1.00 };
    pub const DARK_GREY: Color = Color { r: 0.31, g: 0.31, b: 0.31, a: 1.00 };
    pub const YELLOW: Color = Color { r: 0.99, g: 0.98, b: 0.00, a: 1.00 };
    pub const GOLD: Color = Color { r: 1.00, g: 0.80, b: 0.00, a: 1.00 };
    pub const ORANGE: Color = Color { r: 1.00, g: 0.63, b: 0.00, a: 1.00 };
    pub const PINK: Color = Color { r: 1.00, g: 0.43, b: 0.76, a: 1.00 };
    pub const RED: Color = Color { r: 0.90, g: 0.16, b: 0.22, a: 1.00 };
    pub const MAROON: Color = Color { r: 0.75, g: 0.13, b: 0.22, a: 1.00 };
    pub const GREEN: Color = Color { r: 0.00, g: 0.89, b: 0.19, a: 1.00 };
    pub const LIME: Color = Color { r: 0.00, g: 0.62, b: 0.18, a: 1.00 };
    pub const DARK_GREEN: Color = Color { r: 0.00, g: 0.46, b: 0.17, a: 1.00 };
    pub const SKY_BLUE: Color = Color { r: 0.40, g: 0.75, b: 1.00, a: 1.00 };
    pub const BLUE: Color = Color { r: 0.00, g: 0.47, b: 0.95, a: 1.00 };
    pub const DARK_BLUE: Color = Color { r: 0.00, g: 0.32, b: 0.67, a: 1.00 };
    pub const PURPLE: Color = Color { r: 0.78, g: 0.48, b: 1.00, a: 1.00 };
    pub const VIOLET: Color = Color { r: 0.53, g: 0.24, b: 0.75, a: 1.00 };
    pub const DARK_PURPLE: Color = Color { r: 0.44, g: 0.12, b: 0.49, a: 1.00 };
    pub const BEIGE: Color = Color { r: 0.83, g: 0.69, b: 0.51, a: 1.00 };
    pub const BROWN: Color = Color { r: 0.50, g: 0.42, b: 0.31, a: 1.00 };
    pub const DARK_BROWN: Color = Color { r: 0.30, g: 0.25, b: 0.18, a: 1.00 };
    pub const WHITE: Color = Color { r: 1.00, g: 1.00, b: 1.00, a: 1.00 };
    pub const BLACK: Color = Color { r: 0.00, g: 0.00, b: 0.00, a: 1.00 };
    pub const BLANK: Color = Color { r: 0.00, g: 0.00, b: 0.00, a: 1.00 };
    pub const MAGENTA: Color = Color { r: 1.00, g: 0.00, b: 1.00, a: 1.00 };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_from_hex_string_no_hash() {
        assert_eq!(
            Color::from_hex("12ab6f"),
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
            Color::from_hex("#12ab6f"),
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
            Color::from_hex("12ab6fb2"),
            Color::new(
                18 as f32 / 255.0,
                171 as f32 / 255.0,
                111 as f32 / 255.0,
                178 as f32 / 255.0,
            )
        );
    }

    #[test]
    fn test_color_from_bytes() {
        assert_eq!(Color::new(1.0, 0.0, 0.0, 1.0), color_u8!(255, 0, 0));
        assert_eq!(
            Color::new(1.0, 0.5, 0.0, 1.0),
            color_u8!(255, 127.5, 0)
        );
        assert_eq!(
            Color::new(0.0, 1.0, 0.5, 1.0),
            color_u8!(0, 255, 127.5)
        );

        assert_eq!(Color::new(1.0, 0.0, 0.0, 1.0), color_u8!(255, 0, 0, 255));
        assert_eq!(
            Color::new(1.0, 0.5, 0.0, 1.0),
            color_u8!(255, 127.5, 0, 255)
        );
        assert_eq!(
            Color::new(0.0, 1.0, 0.5, 1.0),
            color_u8!(0, 255, 127.5, 255)
        );
    }
}
