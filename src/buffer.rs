use std::mem;

use crate::check;

pub struct VertexBuffer {
    id: u32,
}

impl VertexBuffer {
    pub fn new(positions: &[f32]) -> Self {
        let mut id = 0;
        unsafe {
            gl::GenBuffers(1, &mut id);
            gl::BindBuffer(gl::ARRAY_BUFFER, id);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (mem::size_of::<f32>() * positions.len()) as isize,
                mem::transmute(&positions[0]),
                gl::STATIC_DRAW,
            );
        }

        VertexBuffer { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, 0);
        }
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

#[derive(Debug)]
pub struct IndexBuffer {
    id: u32,
    pub count: u32,
}

impl IndexBuffer {
    pub fn new(indices: &[u32]) -> Self {
        let mut id = 0;
        unsafe {
            check!(gl::GenBuffers(1, &mut id));
            check!(gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id));
            check!(gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (mem::size_of::<u32>() * indices.len()) as isize,
                mem::transmute(&indices[0]),
                gl::STATIC_DRAW,
            ));
        }

        IndexBuffer {
            id,
            count: indices.len() as u32,
        }
    }

    pub fn bind(&self) {
        check!(unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, self.id);
        })
    }

    pub fn unbind(&self) {
        check!(unsafe {
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0);
        })
    }
}

impl Drop for IndexBuffer {
    fn drop(&mut self) {
        check!(unsafe {
            gl::DeleteBuffers(1, &self.id);
        })
    }
}

#[repr(u32)]
#[derive(Debug, Clone, Copy)]
pub enum BufferElementType {
    Float = gl::FLOAT,
    UnsignedInt = gl::UNSIGNED_INT,
}

impl BufferElementType {
    pub fn size_of(&self) -> usize {
        match self {
            Self::Float => mem::size_of::<f32>(),
            Self::UnsignedInt => mem::size_of::<u32>(),
        }
    }
}

pub struct VertexBufferElement {
    pub count: u32,
    pub ty: BufferElementType,
    pub normalized: bool,
}

pub struct VertexBufferLayout {
    elements: Vec<VertexBufferElement>,
    pub stride: u32,
}

impl VertexBufferLayout {
    pub fn new() -> Self {
        VertexBufferLayout {
            elements: Vec::new(),
            stride: 0,
        }
    }

    pub fn push(&mut self, ty: BufferElementType, count: u32, normalized: bool) {
        self.stride += ty.size_of() as u32 * count;
        self.elements.push(VertexBufferElement {
            count,
            ty,
            normalized,
        });
    }

    pub fn elements(&self) -> &[VertexBufferElement] {
        &self.elements
    }
}
