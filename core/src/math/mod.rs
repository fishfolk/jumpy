use cfg_if::cfg_if;
use serde::{Deserialize, Serialize};

pub use num_traits::*;

pub use crate::backend_impl::math::*;

pub mod size;
pub mod urect;

pub use size::*;
pub use urect::*;

use crate::color::Color;
use crate::video::VideoMode;

pub trait AsVec2 {
    fn as_vec2(&self) -> Vec2;
}

pub trait AsIVec2 {
    fn as_ivec2(&self) -> IVec2;
}

pub trait AsUVec2 {
    fn as_uvec2(&self) -> UVec2;
}

pub fn polar_to_cartesian(rho: f32, theta: f32) -> Vec2 {
    vec2(rho * theta.cos(), rho * theta.sin())
}

/// Converts 2d cartesian coordinates to 2d polar coordinates.
pub fn cartesian_to_polar(cartesian: Vec2) -> Vec2 {
    vec2(
        (cartesian.x.powi(2) + cartesian.y.powi(2)).sqrt(),
        cartesian.y.atan2(cartesian.x),
    )
}

/// Returns value, bounded in range [min, max].
pub fn clamp<T: PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
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
