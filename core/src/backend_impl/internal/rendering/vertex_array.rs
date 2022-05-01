use glow::{HasContext, NativeVertexArray};

use crate::gl::gl_context;
use crate::rendering::vertex::{VertexImpl, VertexLayout};
use crate::{Result, FLOAT_SIZE};

pub struct VertexArray {
    gl_vertex_array: NativeVertexArray,
    layout: VertexLayout,
}

impl VertexArray {
    pub fn new<V: VertexImpl>() -> Result<Self> {
        let gl = gl_context();
        let gl_vertex_array = unsafe { gl.create_vertex_array() }?;

        Ok(VertexArray {
            gl_vertex_array,
            layout: V::layout(),
        })
    }

    pub fn bind(&self) {
        let gl = gl_context();

        let mut offset = 0;

        unsafe {
            gl.bind_vertex_array(Some(self.gl_vertex_array));

            for (i, entry) in self.layout.entries.iter().enumerate() {
                gl.vertex_attrib_pointer_f32(
                    i as u32,
                    entry.size as i32,
                    glow::FLOAT,
                    false,
                    0,
                    offset as i32,
                );

                offset += entry.size * FLOAT_SIZE;
            }
        }
    }

    pub fn attr_cnt(&self) -> usize {
        self.layout.entries.len()
    }
}
