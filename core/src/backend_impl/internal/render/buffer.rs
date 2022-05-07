use glow::{HasContext, NativeBuffer};

use crate::gl::gl_context;
use crate::prelude::Vertex;
use crate::render::vertex::{Index, VertexImpl};
use crate::result::Result;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum BufferKind {
    Vertex,
    Element,
}

impl From<BufferKind> for u32 {
    fn from(kind: BufferKind) -> Self {
        match kind {
            BufferKind::Vertex => glow::ARRAY_BUFFER,
            BufferKind::Element => glow::ELEMENT_ARRAY_BUFFER,
        }
    }
}

impl From<u32> for BufferKind {
    fn from(target: u32) -> Self {
        match target {
            glow::ARRAY_BUFFER => Self::Vertex,
            glow::ELEMENT_ARRAY_BUFFER => Self::Element,
            _ => panic!("Invalid buffer target '{}'!", target),
        }
    }
}

#[derive(Debug)]
pub struct Buffer<T> {
    gl_buffer: NativeBuffer,
    length: usize,
    pub kind: BufferKind,
    _p: core::marker::PhantomData<T>,
}

impl<V: VertexImpl> Buffer<V> {
    pub fn new_vertex(length: usize) -> Result<Self> {
        Self::new(BufferKind::Vertex, length)
    }
}

impl Buffer<Index> {
    pub fn new_element(length: usize) -> Result<Self> {
        Self::new(BufferKind::Element, length)
    }
}

impl<T> Buffer<T> {
    pub fn new(kind: BufferKind, length: usize) -> Result<Self> {
        let target = kind.into();

        let gl = gl_context();
        let gl_buffer = unsafe {
            let buffer = gl.create_buffer()?;

            gl.bind_buffer(target, Some(buffer));
            gl.buffer_data_size(
                target,
                length as i32 * core::mem::size_of::<T>() as i32,
                glow::STREAM_DRAW,
            );

            buffer
        };

        Ok(Buffer {
            gl_buffer,
            length,
            kind,
            _p: core::marker::PhantomData,
        })
    }

    pub fn bind(&self) {
        let gl = gl_context();
        unsafe {
            gl.bind_buffer(self.kind.into(), Some(self.gl_buffer));
        }
    }

    /// Size in bytes
    pub fn size(&self) -> usize {
        self.length
    }

    /// Set buffer sub data, resizing the buffer if necessary
    pub fn set_data(&mut self, offset: usize, data: &[T]) {
        self.bind();

        let target = self.kind.into();

        let gl = gl_context();
        unsafe {
            let data_len = data.len();

            let bytes: &[u8] = core::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data_len * core::mem::size_of::<T>(),
            );

            if data_len >= self.length {
                gl.buffer_data_size(target, bytes.len() as i32, glow::STREAM_DRAW);
                self.length = data_len;
            }

            gl.buffer_sub_data_u8_slice(target, offset as i32, bytes)
        }
    }

    pub fn gl_buffer(&self) -> NativeBuffer {
        self.gl_buffer
    }
}

impl<T> PartialEq for Buffer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.gl_buffer == other.gl_buffer
    }
}

impl<T> Eq for Buffer<T> {}

impl<T> Drop for Buffer<T> {
    fn drop(&mut self) {
        let gl = gl_context();
        unsafe {
            gl.delete_buffer(self.gl_buffer);
        }
    }
}
