use std::time::Instant;

use glutin::event::MouseScrollDelta;
use nalgebra::{Matrix4, Vector3};
use nalgebra_glm::vec3;

pub struct FlightCamera {
    pos: Vector3<f32>,
    front: Vector3<f32>,
    up: Vector3<f32>,

    speed: f32,
    speed_muliplier: f32,
    delta_time: f32,
    last_frame: f32,

    start: Instant,

    yaw: f32,
    pitch: f32,
    first_mouse_movement: bool,

    fov: f32,
}

impl FlightCamera {
    pub fn new(speed_muliplier: f32) -> Self {
        let pos = nalgebra_glm::vec3(0.0_f32, 0.0, 3.0);
        let front = nalgebra_glm::vec3(0.0_f32, 0.0, -1.0);
        let up = nalgebra_glm::vec3(0.0_f32, 1.0, 0.0);

        FlightCamera {
            pos,
            front,
            up,
            speed: 0.0,
            speed_muliplier,
            delta_time: 0.0,
            last_frame: 0.0,
            start: Instant::now(),
            yaw: -95.0,
            pitch: 4.0,
            fov: 45.0,
            first_mouse_movement: true,
        }
    }

    fn get_time(&self) -> f32 {
        (self.start.elapsed().as_secs() as f64 + self.start.elapsed().subsec_nanos() as f64 * 1e-9)
            as f32
    }

    pub fn move_mouse(&mut self, mut x_offset: f32, mut y_offset: f32) {
        if self.first_mouse_movement {
            self.first_mouse_movement = false;
            return;
        }

        let sensitivity = 0.25;

        x_offset *= sensitivity;
        y_offset *= sensitivity;

        self.yaw += x_offset;
        self.pitch += y_offset;

        self.pitch = self.pitch.clamp(-89.0, 89.0);

        let direction = nalgebra_glm::vec3(
            self.yaw.to_radians().cos() * self.pitch.to_radians().cos(),
            self.pitch.to_radians().sin(),
            self.yaw.to_radians().sin() * self.pitch.to_radians().cos(),
        );

        self.front = nalgebra_glm::normalize(&direction);
    }

    pub fn scroll(&mut self, delta: MouseScrollDelta) {
        self.fov -= match delta {
            MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
            MouseScrollDelta::LineDelta(_, y) => y,
        };

        self.fov = self.fov.clamp(1.0, 45.0);
    }

    pub fn next_frame(&mut self) {
        let current_frame = self.get_time();

        self.delta_time = current_frame - self.last_frame;
        self.last_frame = current_frame;

        self.speed = self.speed_muliplier * self.delta_time;
    }

    pub fn view(&self) -> Matrix4<f32> {
        nalgebra_glm::look_at(&self.pos, &(self.pos + self.front), &self.up)
    }

    pub fn fov(&self) -> f32 {
        self.fov.to_radians()
    }

    fn clamp_pos(&mut self) {
        let min = -20.0_f32;
        let max = 20.0_f32;

        self.pos = vec3(
            self.pos.x.clamp(min, max),
            self.pos.y.clamp(min, max),
            self.pos.z.clamp(min, max),
        );
    }

    pub fn move_left(&mut self) {
        self.pos -=
            nalgebra_glm::normalize(&nalgebra_glm::cross(&self.front, &self.up)) * self.speed;

        self.clamp_pos();
    }

    pub fn move_right(&mut self) {
        self.pos +=
            nalgebra_glm::normalize(&nalgebra_glm::cross(&self.front, &self.up)) * self.speed;

        self.clamp_pos();
    }

    pub fn move_forward(&mut self) {
        self.pos += self.speed * self.front;

        self.clamp_pos();
    }

    pub fn move_backward(&mut self) {
        self.pos -= self.speed * self.front;

        self.clamp_pos();
    }

    pub fn pos(&self) -> &[f32] {
        &self.pos.data
    }
}
