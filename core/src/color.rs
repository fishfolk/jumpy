//! Color types and helpers.

use serde::{Deserialize, Serialize};

pub use crate::backend_impl::color::*;

pub use colors::*;

use crate::math::One;

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Color {
    #[serde(alias = "r")]
    pub red: f32,
    #[serde(alias = "g")]
    pub green: f32,
    #[serde(alias = "b")]
    pub blue: f32,
    #[serde(default = "f32::one", alias = "a")]
    pub alpha: f32,
}

impl Color {
    pub const fn new(red: f32, green: f32, blue: f32, alpha: f32) -> Self {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn from_bytes(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
        Color {
            red: red as f32 / 255.0,
            green: green as f32 / 255.0,
            blue: blue as f32 / 255.0,
            alpha: alpha as f32 / 255.0,
        }
    }

    pub fn to_bytes(self) -> (u8, u8, u8, u8) {
        let r = self.red * 255.0;
        let g = self.green * 255.0;
        let b = self.blue * 255.0;
        let a = self.alpha * 255.0;

        (r as u8, g as u8, b as u8, a as u8)
    }

    pub fn from_hsl(h: f32, s: f32, l: f32) -> Self {
        let r;
        let g;
        let b;

        if s == 0.0 {
            r = l;
            g = l;
            b = l;
        } else {
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

        let r = self.red;
        let g = self.green;
        let b = self.blue;

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

    pub fn to_array(self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

impl Default for Color {
    fn default() -> Self {
        Color {
            red: 0.0,
            blue: 0.0,
            green: 0.0,
            alpha: 1.0,
        }
    }
}

impl From<[f32; 3]> for Color {
    fn from(color: [f32; 3]) -> Self {
        Color::new(color[0], color[1], color[2], 1.0)
    }
}

impl From<[f32; 4]> for Color {
    fn from(color: [f32; 4]) -> Self {
        Color::new(color[0], color[1], color[2], color[3])
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
        Color::new($r as f32 / 255.0, $g as f32 / 255.0, $b as f32 / 255.0, 1.0)
    };
}

pub fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0
    }
    if t > 1.0 {
        t -= 1.0
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

pub mod colors {
    //! Constants for some common colors.

    use super::Color;

    pub const LIGHT_GREY: Color = Color {
        red: 0.78,
        green: 0.78,
        blue: 0.78,
        alpha: 1.00,
    };
    pub const GREY: Color = Color {
        red: 0.51,
        green: 0.51,
        blue: 0.51,
        alpha: 1.00,
    };
    pub const DARK_GREY: Color = Color {
        red: 0.31,
        green: 0.31,
        blue: 0.31,
        alpha: 1.00,
    };
    pub const YELLOW: Color = Color {
        red: 0.99,
        green: 0.98,
        blue: 0.00,
        alpha: 1.00,
    };
    pub const GOLD: Color = Color {
        red: 1.00,
        green: 0.80,
        blue: 0.00,
        alpha: 1.00,
    };
    pub const ORANGE: Color = Color {
        red: 1.00,
        green: 0.63,
        blue: 0.00,
        alpha: 1.00,
    };
    pub const PINK: Color = Color {
        red: 1.00,
        green: 0.43,
        blue: 0.76,
        alpha: 1.00,
    };
    pub const RED: Color = Color {
        red: 0.90,
        green: 0.16,
        blue: 0.22,
        alpha: 1.00,
    };
    pub const MAROON: Color = Color {
        red: 0.75,
        green: 0.13,
        blue: 0.22,
        alpha: 1.00,
    };
    pub const GREEN: Color = Color {
        red: 0.00,
        green: 0.89,
        blue: 0.19,
        alpha: 1.00,
    };
    pub const LIME: Color = Color {
        red: 0.00,
        green: 0.62,
        blue: 0.18,
        alpha: 1.00,
    };
    pub const DARK_GREEN: Color = Color {
        red: 0.00,
        green: 0.46,
        blue: 0.17,
        alpha: 1.00,
    };
    pub const SKY_BLUE: Color = Color {
        red: 0.40,
        green: 0.75,
        blue: 1.00,
        alpha: 1.00,
    };
    pub const BLUE: Color = Color {
        red: 0.00,
        green: 0.47,
        blue: 0.95,
        alpha: 1.00,
    };
    pub const DARK_BLUE: Color = Color {
        red: 0.00,
        green: 0.32,
        blue: 0.67,
        alpha: 1.00,
    };
    pub const PURPLE: Color = Color {
        red: 0.78,
        green: 0.48,
        blue: 1.00,
        alpha: 1.00,
    };
    pub const VIOLET: Color = Color {
        red: 0.53,
        green: 0.24,
        blue: 0.75,
        alpha: 1.00,
    };
    pub const DARK_PURPLE: Color = Color {
        red: 0.44,
        green: 0.12,
        blue: 0.49,
        alpha: 1.00,
    };
    pub const BEIGE: Color = Color {
        red: 0.83,
        green: 0.69,
        blue: 0.51,
        alpha: 1.00,
    };
    pub const BROWN: Color = Color {
        red: 0.50,
        green: 0.42,
        blue: 0.31,
        alpha: 1.00,
    };
    pub const DARK_BROWN: Color = Color {
        red: 0.30,
        green: 0.25,
        blue: 0.18,
        alpha: 1.00,
    };
    pub const WHITE: Color = Color {
        red: 1.00,
        green: 1.00,
        blue: 1.00,
        alpha: 1.00,
    };
    pub const BLACK: Color = Color {
        red: 0.00,
        green: 0.00,
        blue: 0.00,
        alpha: 1.00,
    };
    pub const BLANK: Color = Color {
        red: 0.00,
        green: 0.00,
        blue: 0.00,
        alpha: 1.00,
    };
    pub const MAGENTA: Color = Color {
        red: 1.00,
        green: 0.00,
        blue: 1.00,
        alpha: 1.00,
    };
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
        assert_eq!(Color::new(1.0, 0.5, 0.0, 1.0), color_u8!(255, 127.5, 0));
        assert_eq!(Color::new(0.0, 1.0, 0.5, 1.0), color_u8!(0, 255, 127.5));

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
