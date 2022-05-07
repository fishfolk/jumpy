use glow::{HasContext, NativeVertexArray};

use crate::gl::gl_context;
use crate::render::vertex::{VertexImpl, VertexLayout};
use crate::result::Result;

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
        unsafe {
            gl.bind_vertex_array(Some(self.gl_vertex_array));
        }
    }

    pub fn enable_layout(&self) {
        self.bind();

        let mut offset = 0;

        let gl = gl_context();
        unsafe {
            let float_size = core::mem::size_of::<f32>();

            for (i, entry) in self.layout.entries.iter().enumerate() {
                gl.enable_vertex_attrib_array(i as u32);

                gl.vertex_attrib_pointer_f32(
                    i as u32,
                    entry.length as i32,
                    glow::FLOAT,
                    false,
                    entry.stride as i32,
                    offset,
                );

                offset += (entry.length * float_size) as i32;
            }
        }
    }

    pub fn attr_cnt(&self) -> usize {
        self.layout.entries.len()
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        for i in 0..self.attr_cnt() {
            let gl = gl_context();
            unsafe {
                gl.disable_vertex_attrib_array(i as u32);
            }
        }
    }
}
