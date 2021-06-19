use std::{
    fs::{File, OpenOptions},
    io::{self, Read, Write},
    path::Path,
};

use glutin::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, Event, ModifiersState, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{CursorIcon, Window},
    ContextWrapper, PossiblyCurrent,
};
use nalgebra::Vector3;

pub use buffer::{BufferElementType, IndexBuffer, VertexBuffer, VertexBufferLayout};
pub use camera::FlightCamera;
use nalgebra_glm::vec3;
pub use renderer::Renderer;
pub use shader::{Material, Shader, Uniform};
use stl::StlFile;
pub use texture::Texture;
pub use vertex_array::VertexArray;

mod buffer;
mod camera;
mod renderer;
mod shader;
mod texture;
mod vertex_array;

pub fn clear_error() {
    while unsafe { gl::GetError() } != gl::NO_ERROR {}
}

#[track_caller]
pub fn check_error() {
    assert_eq!(unsafe { gl::GetError() }, gl::NO_ERROR)
}

#[macro_export]
macro_rules! check {
    ($call:expr) => {{
        #[cfg(debug_assertions)]
        {
            crate::clear_error();
            let x = $call;
            crate::check_error();
            x
        }

        #[cfg(not(debug_assertions))]
        {
            $call
        }
    }};
}

struct Color {
    red: f32,
    green: f32,
    blue: f32,
}

struct Light {
    color: Color,
}

impl Light {
    pub const fn white() -> Self {
        Light {
            color: Color {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
            },
        }
    }
}

struct StationaryCamera {
    model: nalgebra::Matrix4<f32>,
    speed: f32,
}

impl StationaryCamera {
    pub fn new(model: nalgebra::Matrix4<f32>) -> Self {
        Self {
            model,
            speed: 1.0_f32.to_radians(),
        }
    }

    pub fn left(&mut self) {
        self.model = nalgebra_glm::rotate(&self.model, -self.speed, &Vector3::y_axis());
    }

    pub fn right(&mut self) {
        self.model = nalgebra_glm::rotate(&self.model, self.speed, &Vector3::y_axis());
    }

    pub fn up(&mut self) {
        self.model = nalgebra_glm::rotate(&self.model, -self.speed, &Vector3::x_axis());
    }

    pub fn down(&mut self) {
        self.model = nalgebra_glm::rotate(&self.model, self.speed, &Vector3::x_axis());
    }

    pub fn model(&self) -> &nalgebra::Matrix4<f32> {
        &self.model
    }

    pub fn move_mouse(&mut self, x_offset: f32, y_offset: f32) {
        self.model =
            nalgebra_glm::rotate(&self.model, x_offset.to_radians() / 2.0, &Vector3::y_axis());
        self.model = nalgebra_glm::rotate(
            &self.model,
            -y_offset.to_radians() / 2.0,
            &Vector3::x_axis(),
        );
    }

    pub fn pos(&self, bbox: stl::BoundingBox) -> [f32; 3] {
        let dimensions = bbox.delta();

        [dimensions.x * 2.0, dimensions.y * 2.0, dimensions.z * 2.0]
    }

    pub fn view(&self, bbox: stl::BoundingBox) -> nalgebra::Matrix4<f32> {
        let center = bbox.center();
        let dimensions = bbox.delta();

        nalgebra_glm::look_at(
            &(vec3(dimensions.x, dimensions.y, dimensions.z) * 2.0),
            &vec3(center.x, center.y, center.z),
            &Vector3::new(0.0, 1.0, 0.0),
        )
    }
}

fn main() {
    let event_loop = EventLoop::new();

    let path_queue = vec![
        "Eiffel_tower_sample.stl".to_owned(),
        "Utah_teapot_(solid).stl".to_owned(),
    ];

    let path_loader = PathLoader::new(path_queue, "./w", "./a", "./s", "./d");

    let program = Program::init(&event_loop, path_loader);

    program.run(event_loop);
}

struct PathLoader {
    w_file: File,
    a_file: File,
    s_file: File,
    d_file: File,
    queue: Vec<String>,
}

impl PathLoader {
    pub fn new(
        queue: Vec<String>,
        w_path: impl AsRef<Path>,
        a_path: impl AsRef<Path>,
        s_path: impl AsRef<Path>,
        d_path: impl AsRef<Path>,
    ) -> Self {
        let w_file = OpenOptions::new().append(true).open(w_path).unwrap();
        let a_file = OpenOptions::new().append(true).open(a_path).unwrap();
        let s_file = OpenOptions::new().append(true).open(s_path).unwrap();
        let d_file = OpenOptions::new().append(true).open(d_path).unwrap();

        Self {
            w_file,
            a_file,
            s_file,
            d_file,
            queue,
        }
    }
}

struct Program {
    window: ContextWrapper<PossiblyCurrent, Window>,
    stationary: StationaryCamera,
    camera: FlightCamera,
    window_state: WindowState,
    control_flow: ControlFlow,
    renderer: Renderer,
    stl_context: StlContext,
    shader: Shader,
    buffer_context: Option<BufferContext>,
}

struct StlContext {
    path_loader: PathLoader,
    cursor: usize,
    current: Option<stl::StlFile>,
    stl_buffer: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum ScanKind {
    W,
    A,
    S,
    D,
}

impl StlContext {
    pub fn new(path_loader: PathLoader) -> Self {
        Self {
            path_loader,
            stl_buffer: Vec::new(),
            current: None,
            cursor: 0,
        }
    }

    pub fn label(&mut self, scan_kind: ScanKind) -> io::Result<()> {
        if self.current.is_some() {
            if let Some(path) = self.path_loader.queue.get(self.cursor.saturating_sub(1)) {
                let file = match scan_kind {
                    ScanKind::W => &mut self.path_loader.w_file,
                    ScanKind::A => &mut self.path_loader.a_file,
                    ScanKind::S => &mut self.path_loader.s_file,
                    ScanKind::D => &mut self.path_loader.d_file,
                };

                file.write_all(path.as_bytes())?;
                file.write_all(&[b'\n'])?;
            }
        }

        Ok(())
    }

    pub fn load_next(&mut self) -> Option<&StlFile> {
        let next_path = self.path_loader.queue.get(self.cursor)?;

        let mut file = File::open(next_path).unwrap();

        self.stl_buffer.clear();
        file.read_to_end(&mut self.stl_buffer).unwrap();

        self.cursor += 1;

        let file = StlFile::parse(&self.stl_buffer).unwrap();

        self.current = Some(file);

        self.current.as_ref()
    }
}

#[derive(Debug)]
struct BufferContext {
    bbox: stl::BoundingBox,
    ib: IndexBuffer,
    va: VertexArray,
}

impl Program {
    pub fn init(event_loop: &EventLoop<()>, path_loader: PathLoader) -> Self {
        let window = glutin::window::WindowBuilder::new().with_title("");
        let gl_window = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_multisampling(4)
            .build_windowed(window, event_loop)
            .unwrap();

        let gl_window = unsafe { gl_window.make_current() }.unwrap();

        gl::load_with(|symbol| gl_window.get_proc_address(symbol));

        let dimensions = gl_window.window().inner_size();

        unsafe {
            gl::Viewport(0, 0, dimensions.width as i32, dimensions.height as i32);
        };

        check!(unsafe { gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA) });
        check!(unsafe { gl::Enable(gl::BLEND) });
        check!(unsafe { gl::Enable(gl::DEPTH_TEST) });
        check!(unsafe { gl::Enable(gl::MULTISAMPLE) });

        let model: nalgebra::Matrix4<f32> = nalgebra_glm::rotate(
            &nalgebra_glm::one(),
            -55.0_f32.to_radians(),
            &Vector3::x_axis(),
        );

        let camera = FlightCamera::new(50.0_f32);
        let stationary = StationaryCamera::new(model);

        let projection = nalgebra_glm::perspective(
            dimensions.width as f32 / dimensions.height as f32,
            camera.fov(),
            0.1,
            100.0,
        );

        let light = Light::white();

        let renderer = Renderer::new();

        let shader = Self::init_shaders(&model, &projection, &light);

        Self {
            window: gl_window,
            camera,
            stationary,
            renderer,
            shader,
            buffer_context: None,
            window_state: WindowState::new(),
            control_flow: ControlFlow::Wait,
            stl_context: StlContext::new(path_loader),
        }
    }

    fn dimensions(&self) -> PhysicalSize<u32> {
        self.window.window().inner_size()
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        self.load_next_stl();

        event_loop.run(move |event, _, control_flow| {
            *control_flow = self.control_flow;

            match event {
                Event::LoopDestroyed => return,
                Event::WindowEvent { event, .. } => self.handle_window_event(event),
                Event::DeviceEvent { event, .. } => self.handle_device_event(event),
                Event::RedrawRequested(_) => {
                    self.renderer.clear();

                    let buffer_context = match &self.buffer_context {
                        Some(b) => b,
                        None => return,
                    };

                    let dimensions = self.dimensions();

                    self.renderer.draw(
                        &buffer_context.va,
                        &buffer_context.ib,
                        &mut Material::new(
                            &mut self.shader,
                            &[
                                Uniform::MatrixFourFv {
                                    name: "model",
                                    matrix: self.stationary.model(),
                                },
                                Uniform::MatrixFourFv {
                                    name: "view",
                                    matrix: &self.stationary.view(buffer_context.bbox),
                                },
                                Uniform::MatrixFourFv {
                                    name: "projection",
                                    matrix: &nalgebra_glm::perspective(
                                        dimensions.width as f32 / dimensions.height as f32,
                                        self.camera.fov(),
                                        1.0,
                                        1000.0,
                                    ),
                                },
                                Uniform::ThreeFloat {
                                    name: "light_pos",
                                    v0: self.stationary.pos(buffer_context.bbox)[0],
                                    v1: self.stationary.pos(buffer_context.bbox)[1],
                                    v2: self.stationary.pos(buffer_context.bbox)[2],
                                },
                            ],
                        ),
                    );

                    self.window.swap_buffers().unwrap();
                }
                _ => {}
            }

            self.window.window().request_redraw();
        });
    }

    fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => self.control_flow = ControlFlow::Exit,
            WindowEvent::ModifiersChanged(state) => {
                self.window_state.modifiers = state;
            }
            WindowEvent::Resized(new_dimensions) => {
                unsafe {
                    gl::Viewport(
                        0,
                        0,
                        new_dimensions.width as i32,
                        new_dimensions.height as i32,
                    );
                };

                self.renderer.clear();
            }
            WindowEvent::Focused(focused) => {
                self.window_state.is_window_focused = focused;
            }
            WindowEvent::CursorEntered { .. } => {
                self.window_state.is_window_hovered = true;
            }
            WindowEvent::CursorLeft { .. } => {
                self.window_state.is_window_hovered = false;
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                ..
            } => {
                self.window.window().set_cursor_icon(CursorIcon::Grabbing);
                self.window_state.is_mouse_pressed = true;
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                ..
            } => {
                self.window.window().set_cursor_icon(CursorIcon::Default);
                self.window_state.is_mouse_pressed = false;
            }
            WindowEvent::KeyboardInput { input, .. } => {
                match (input.virtual_keycode, input.state) {
                    (Some(VirtualKeyCode::Left), ElementState::Pressed) => {
                        self.stationary.left();
                    }
                    (Some(VirtualKeyCode::Right), ElementState::Pressed) => {
                        self.stationary.right();
                    }
                    (Some(VirtualKeyCode::Up), ElementState::Pressed) => {
                        self.stationary.up();
                    }
                    (Some(VirtualKeyCode::Down), ElementState::Pressed) => {
                        self.stationary.down();
                    }
                    (Some(VirtualKeyCode::P), ElementState::Pressed) => {
                        self.window_state.toggle_paused();
                    }
                    (Some(VirtualKeyCode::Q), ElementState::Pressed) => {
                        self.control_flow = ControlFlow::Exit;
                    }
                    (Some(VirtualKeyCode::C), ElementState::Pressed)
                        if self.window_state.modifiers.ctrl() =>
                    {
                        self.control_flow = ControlFlow::Exit;
                    }
                    (Some(VirtualKeyCode::W), ElementState::Pressed) => {
                        self.label(ScanKind::W);
                    }
                    (Some(VirtualKeyCode::A), ElementState::Pressed) => {
                        self.label(ScanKind::A);
                    }
                    (Some(VirtualKeyCode::S), ElementState::Pressed) => {
                        self.label(ScanKind::S);
                    }
                    (Some(VirtualKeyCode::D), ElementState::Pressed) => {
                        self.label(ScanKind::D);
                    }
                    _ => (),
                }
            }
            _ => (),
        }
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseWheel { delta } => {
                self.camera.scroll(delta);
            }
            DeviceEvent::MouseMotion { delta } => {
                if !self.window_state.is_paused && self.window_state.is_window_focused {
                    if self.window_state.is_mouse_pressed {
                        self.stationary.move_mouse(delta.0 as f32, -delta.1 as f32);
                    }
                }
            }
            _ => {}
        }
    }

    fn init_shaders(
        model: &nalgebra::Matrix4<f32>,
        projection: &nalgebra::Matrix4<f32>,
        light: &Light,
    ) -> Shader {
        let mut shader = Shader::new("src/shaders/basic-vs.shader", "src/shaders/basic-fs.shader");
        let uniforms = vec![];

        let mut material = Material::new(&mut shader, &uniforms);
        material.bind();

        shader.set_uniform(&Uniform::MatrixFourFv {
            name: "model",
            matrix: model,
        });
        shader.set_uniform(&Uniform::MatrixFourFv {
            name: "view",
            matrix: &nalgebra::Matrix4::identity(),
        });
        shader.set_uniform(&Uniform::MatrixFourFv {
            name: "projection",
            matrix: projection,
        });
        shader.set_uniform(&Uniform::ThreeFloat {
            name: "object_color",
            v0: 0.8,
            v1: 0.8,
            v2: 0.8,
        });
        shader.set_uniform(&Uniform::ThreeFloat {
            name: "light_color",
            v0: light.color.red,
            v1: light.color.green,
            v2: light.color.blue,
        });
        shader.set_uniform(&Uniform::ThreeFloat {
            name: "light_pos",
            v0: 0.0,
            v1: 0.0,
            v2: 0.0,
        });

        shader.unbind();

        shader
    }

    fn load_next_stl(&mut self) {
        let stl_file = match self.stl_context.load_next() {
            Some(f) => f,
            None => {
                self.control_flow = ControlFlow::Exit;
                return;
            }
        };

        let index = stl_file.index_buffer_vertex_and_normal();
        let positions = index.vertices();
        let indices = index.indices();

        let mut va = VertexArray::new();
        let vb = VertexBuffer::new(&positions);
        let mut layout = VertexBufferLayout::new();

        layout.push(BufferElementType::Float, 3, false);
        layout.push(BufferElementType::Float, 3, false);
        va.add_buffer(&vb, &layout);

        let ib = IndexBuffer::new(indices);

        ib.unbind();
        va.unbind();
        vb.unbind();

        let buffer_context = BufferContext {
            va,
            ib,
            bbox: stl_file.bounding_box(),
        };

        self.buffer_context = Some(buffer_context);
    }

    fn label(&mut self, scan_kind: ScanKind) {
        self.stl_context.label(scan_kind).unwrap();
        self.load_next_stl();
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        println!("Stopped at file #{}", self.stl_context.cursor);
    }
}

#[derive(Debug)]
struct WindowState {
    is_paused: bool,
    is_window_focused: bool,
    is_window_hovered: bool,
    is_mouse_pressed: bool,
    modifiers: ModifiersState,
}

impl WindowState {
    pub fn new() -> Self {
        Self {
            is_paused: false,
            is_window_focused: false,
            is_window_hovered: false,
            is_mouse_pressed: false,
            modifiers: ModifiersState::empty(),
        }
    }

    pub fn toggle_paused(&mut self) {
        self.is_paused = !self.is_paused;
    }
}
