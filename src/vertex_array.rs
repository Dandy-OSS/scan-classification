use crate::buffer::{VertexBuffer, VertexBufferLayout};

#[derive(Debug)]
pub struct VertexArray {
    id: u32,
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.id);
        }
    }
}

impl VertexArray {
    pub fn new() -> Self {
        let mut id = 0;
        unsafe {
            gl::GenVertexArrays(1, &mut id);
        }

        Self { id }
    }

    pub fn bind(&self) {
        unsafe {
            gl::BindVertexArray(self.id);
        }
    }

    pub fn unbind(&self) {
        unsafe {
            gl::BindVertexArray(0);
        }
    }

    pub fn add_buffer(&mut self, vb: &VertexBuffer, layout: &VertexBufferLayout) {
        vb.bind();
        self.bind();

        let elements = layout.elements();

        let mut offset = 0;

        for (idx, element) in elements.into_iter().enumerate() {
            unsafe {
                gl::EnableVertexAttribArray(idx as u32);
                gl::VertexAttribPointer(
                    idx as u32,
                    element.count as i32,
                    element.ty as u32,
                    element.normalized as u8,
                    layout.stride as i32,
                    offset as *const _,
                );
            }
            offset += element.count * element.ty.size_of() as u32;
        }
    }
}
