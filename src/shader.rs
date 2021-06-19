use std::{collections::HashMap, ffi::CString, fs, path::Path};

use crate::check;

pub struct Shader {
    id: u32,
    uniform_cache: HashMap<String, i32>,
}

struct ShaderProgramSource {
    pub vertex: String,
    pub fragment: String,
}

impl ShaderProgramSource {
    pub fn parse(vertex_path: &Path, fragment_path: &Path) -> Self {
        let vertex = fs::read_to_string(vertex_path).unwrap();
        let fragment = fs::read_to_string(fragment_path).unwrap();

        ShaderProgramSource { vertex, fragment }
    }
}

#[allow(dead_code)]
pub enum Uniform<'a> {
    OneInteger {
        name: &'a str,
        v0: i32,
    },
    OneFloat {
        name: &'a str,
        v0: f32,
    },
    TwoFloat {
        name: &'a str,
        v0: f32,
        v1: f32,
    },
    ThreeFloat {
        name: &'a str,
        v0: f32,
        v1: f32,
        v2: f32,
    },
    FourFloat {
        name: &'a str,
        v0: f32,
        v1: f32,
        v2: f32,
        v3: f32,
    },
    MatrixThreeFv {
        name: &'a str,
        matrix: &'a nalgebra::Matrix3<f32>,
    },
    MatrixFourFv {
        name: &'a str,
        matrix: &'a nalgebra::Matrix4<f32>,
    },
}

impl Shader {
    pub fn new(vertex: impl AsRef<Path>, fragment: impl AsRef<Path>) -> Self {
        let source = ShaderProgramSource::parse(vertex.as_ref(), fragment.as_ref());

        let id = Self::create_shader(&source.vertex, &source.fragment);

        check!(unsafe { gl::UseProgram(id) });

        Self {
            id,
            uniform_cache: HashMap::new(),
        }
    }

    fn compile_shader(source: &str, kind: u32) -> u32 {
        let src = CString::new(source).unwrap();

        unsafe {
            let id = gl::CreateShader(kind);
            check!(gl::ShaderSource(id, 1, &src.as_ptr(), std::ptr::null()));
            check!(gl::CompileShader(id));

            let mut result = 0;
            gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut result);
            if result != gl::TRUE as i32 {
                let mut len = 0;
                gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut len);

                let mut message = Vec::with_capacity(len as usize);
                gl::GetShaderInfoLog(id, len, std::ptr::null_mut(), message.as_mut_ptr());
                message.set_len(len as usize);

                println!(
                    "{}",
                    String::from_utf8(message.into_iter().map(|n| n as u8).collect()).unwrap()
                );

                check!(gl::DeleteShader(id));
                return 0;
            }

            id
        }
    }

    fn create_shader(vertex_shader: &str, fragment_shader: &str) -> u32 {
        let program = check!(unsafe { gl::CreateProgram() });
        let vs = Self::compile_shader(vertex_shader, gl::VERTEX_SHADER);
        let fs = Self::compile_shader(fragment_shader, gl::FRAGMENT_SHADER);

        unsafe {
            check!(gl::AttachShader(program, vs));
            check!(gl::AttachShader(program, fs));

            check!(gl::LinkProgram(program));

            check!(gl::ValidateProgram(program));

            let mut status = gl::FALSE as i32;
            check!(gl::GetProgramiv(program, gl::LINK_STATUS, &mut status));

            assert_ne!(status, gl::FALSE as i32);

            check!(gl::DeleteShader(vs));
            check!(gl::DeleteShader(fs));
        }

        program
    }

    #[track_caller]
    pub fn set_uniform(&mut self, uniform: &Uniform) {
        match uniform {
            &Uniform::OneInteger { name, v0 } => unsafe {
                check!(gl::Uniform1i(self.uniform_location(name), v0))
            },
            &Uniform::OneFloat { name, v0 } => unsafe {
                check!(gl::Uniform1f(self.uniform_location(name), v0))
            },
            &Uniform::TwoFloat { name, v0, v1 } => unsafe {
                check!(gl::Uniform2f(self.uniform_location(name), v0, v1))
            },
            &Uniform::ThreeFloat { name, v0, v1, v2 } => unsafe {
                check!(gl::Uniform3f(self.uniform_location(name), v0, v1, v2))
            },
            &Uniform::FourFloat {
                name,
                v0,
                v1,
                v2,
                v3,
            } => unsafe {
                check!(gl::Uniform4f(self.uniform_location(name), v0, v1, v2, v3));
            },
            &Uniform::MatrixThreeFv { name, matrix } => unsafe {
                check!(gl::UniformMatrix3fv(
                    self.uniform_location(name),
                    1,
                    gl::FALSE,
                    matrix.data.as_ptr()
                ));
            },
            &Uniform::MatrixFourFv { name, matrix } => unsafe {
                check!(gl::UniformMatrix4fv(
                    self.uniform_location(name),
                    1,
                    gl::FALSE,
                    matrix.data.as_ptr()
                ));
            },
        }
    }

    pub fn bind(&self) {
        unsafe { gl::UseProgram(self.id) }
    }

    pub fn unbind(&self) {
        unsafe { gl::UseProgram(0) }
    }

    #[track_caller]
    fn uniform_location(&self, name: &str) -> i32 {
        if let Some(&location) = self.uniform_cache.get(name) {
            return location;
        }

        let name = CString::new(name).unwrap();

        let location = check!(unsafe { gl::GetUniformLocation(self.id, name.as_ptr()) });

        if location == -1 {
            println!("Could not find location for uniform {:?}", name);
        }

        location
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe { check!(gl::DeleteProgram(self.id)) }
    }
}

pub struct Material<'a> {
    shader: &'a mut Shader,
    uniforms: &'a [Uniform<'a>],
}

impl<'a> Material<'a> {
    pub fn new(shader: &'a mut Shader, uniforms: &'a [Uniform<'a>]) -> Self {
        Self { shader, uniforms }
    }

    pub fn bind(&mut self) {
        self.shader.bind();

        for uniform in self.uniforms {
            self.shader.set_uniform(uniform)
        }
    }
}
