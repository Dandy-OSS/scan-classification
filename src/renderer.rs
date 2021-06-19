use crate::{buffer::IndexBuffer, check, shader::Material, vertex_array::VertexArray};

pub struct Renderer {}

impl Renderer {
    pub fn new() -> Self {
        Self {}
    }

    pub fn clear(&self) {
        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);

            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn draw(&self, va: &VertexArray, ib: &IndexBuffer, material: &mut Material) {
        material.bind();

        va.bind();
        ib.bind();

        check!(unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                ib.count as i32,
                gl::UNSIGNED_INT,
                std::ptr::null(),
            )
        });
    }
}
