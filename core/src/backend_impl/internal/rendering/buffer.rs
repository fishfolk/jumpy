use glow::{HasContext, NativeBuffer};

use crate::gl::gl_context;
use crate::prelude::Vertex;
use crate::rendering::vertex::Index;
use crate::Result;

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

impl Buffer<Vertex> {
    pub fn new_vertex() -> Result<Self> {
        let gl = gl_context();
        let gl_buffer = unsafe { gl.create_buffer()? };

        Ok(Buffer {
            gl_buffer,
            length: 0,
            kind: BufferKind::Vertex,
            _p: core::marker::PhantomData,
        })
    }
}

impl Buffer<Index> {
    pub fn new_element() -> Result<Self> {
        Self::new(BufferKind::Element)
    }
}

impl<T> Buffer<T> {
    pub fn new(kind: BufferKind) -> Result<Self> {
        let gl = gl_context();
        let gl_buffer = unsafe { gl.create_buffer()? };

        Ok(Buffer {
            gl_buffer,
            length: 0,
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

    pub fn unbind(&self) {
        let gl = gl_context();
        unsafe {
            gl.bind_buffer(self.kind.into(), None);
        }
    }

    /// Size in bytes
    pub fn size(&self) -> usize {
        self.length
    }

    /// Set buffer data, resizing the buffer if necessary
    pub fn set_data(&mut self, data: &[T]) {
        self.bind();

        let target = self.kind.into();

        let gl = gl_context();
        unsafe {
            let data: &[u8] = core::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * core::mem::size_of::<T>(),
            );

            let data_length = data.len();

            if data_length >= self.length {
                gl.buffer_data_size(target, data_length as i32, glow::STREAM_DRAW);
                self.length = data_length;
            }

            gl.buffer_sub_data_u8_slice(target, 0, data)
        }
    }

    /// Set buffer sub data, resizing the buffer if necessary
    pub fn set_sub_data(&mut self, offset: usize, data: &[T]) {
        self.bind();

        let target = self.kind.into();

        let gl = gl_context();
        unsafe {
            let data: &[u8] = core::slice::from_raw_parts(
                data.as_ptr() as *const u8,
                data.len() * core::mem::size_of::<T>(),
            );

            let data_length = offset + data.len();

            if data_length >= self.length {
                gl.buffer_data_size(target, data_length as i32, glow::STREAM_DRAW);
                self.length = data_length;
            }

            gl.buffer_sub_data_u8_slice(target, offset as i32, data)
        }
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
