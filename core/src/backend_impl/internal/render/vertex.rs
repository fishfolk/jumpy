use glam::{vec2, Vec3};
use glow::{HasContext, NativeVertexArray};
use serde::{Deserialize, Serialize};

use crate::color::{colors, Color};
use crate::gl::gl_context;
use crate::math::Vec2;
use crate::result::Result;

#[derive(Debug, Copy, Clone)]
pub struct Vertex {
    pub position: Vec2,
    pub color: Color,
    pub texture_coords: Vec2,
}

impl Vertex {
    pub fn new<C, T>(position: Vec2, color: C, texture_coords: T) -> Self
    where
        C: Into<Option<Color>>,
        T: Into<Option<Vec2>>,
    {
        Vertex {
            position,
            color: color.into().unwrap_or_else(|| colors::WHITE),
            texture_coords: texture_coords.into().unwrap_or_else(|| Vec2::ZERO),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VertexLayout {
    pub entries: Vec<VertexLayoutEntry>,
}

impl VertexLayout {
    pub fn new(entries: &[VertexLayoutEntry]) -> Self {
        VertexLayout {
            entries: entries.to_vec(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VertexLayoutEntry {
    pub name: String,
    pub length: usize,
    pub stride: usize,
}

impl VertexLayoutEntry {
    pub fn new(name: &str, length: usize, stride: usize) -> Self {
        VertexLayoutEntry {
            name: name.to_string(),
            length,
            stride,
        }
    }
}

pub trait VertexImpl {
    fn layout() -> VertexLayout;
}

impl VertexImpl for Vertex {
    fn layout() -> VertexLayout {
        let stride = unsafe { core::mem::size_of::<Vertex>() };

        VertexLayout::new(&[
            VertexLayoutEntry::new("vertex_position", 2, stride),
            VertexLayoutEntry::new("vertex_color", 4, stride),
            VertexLayoutEntry::new("texture_coords", 2, stride),
        ])
    }
}

pub type Index = u32;
