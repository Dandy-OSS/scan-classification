use rpng::Png;
use std::path::Path;

use crate::check;

pub struct Texture {
    id: u32,
}

impl Texture {
    pub fn new(p: impl AsRef<Path>) -> Self {
        let mut id = 0;
        check!(unsafe { gl::GenTextures(1, &mut id) });

        check!(unsafe { gl::BindTexture(gl::TEXTURE_2D, id) });

        check!(unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32)
        });
        check!(unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32)
        });
        check!(unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32)
        });
        check!(unsafe {
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32)
        });

        let png = Png::open(p).unwrap();
        let mut pixels = png.pixels().unwrap();

        pixels.flip();

        check!(unsafe {
            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                png.width() as i32,
                png.height() as i32,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.into_buffer().as_ptr() as *const _,
            )
        });
        check!(unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) });

        Self { id }
    }

    pub fn bind(&self, slot: u32) {
        check!(unsafe { gl::ActiveTexture(gl::TEXTURE0 + slot) });
        check!(unsafe { gl::BindTexture(gl::TEXTURE_2D, self.id) });
    }

    pub fn unbind(&self) {
        check!(unsafe { gl::BindTexture(gl::TEXTURE_2D, 0) });
    }
}

impl Drop for Texture {
    fn drop(&mut self) {
        check!(unsafe { gl::DeleteTextures(1, &self.id) });
    }
}
