use glam::{vec2, Vec3};
use glow::{HasContext, NativeVertexArray};
use serde::{Deserialize, Serialize};

use crate::color::{colors, Color};
use crate::gl::gl_context;
use crate::math::Vec2;
use crate::result::Result;
use crate::FLOAT_SIZE;

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
        let mut entries = entries.to_vec();

        let mut offset = 0;
        for entry in &mut entries {
            entry.offset = offset;
            offset += entry.size * FLOAT_SIZE;
        }

        VertexLayout { entries }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct VertexLayoutEntry {
    pub name: String,
    pub size: usize,
    pub offset: usize,
}

impl VertexLayoutEntry {
    pub fn new(name: &str, size: usize) -> Self {
        VertexLayoutEntry {
            name: name.to_string(),
            size,
            offset: 0,
        }
    }

    pub(crate) fn with_offset(self, offset: usize) -> Self {
        VertexLayoutEntry { offset, ..self }
    }
}

pub trait VertexImpl {
    fn layout() -> VertexLayout;
}

impl VertexImpl for Vertex {
    fn layout() -> VertexLayout {
        VertexLayout::new(&[
            VertexLayoutEntry::new("vertex_position", 2),
            VertexLayoutEntry::new("vertex_color", 4),
            VertexLayoutEntry::new("texture_coords", 2),
        ])
    }
}

pub type Index = u32;
