use regnum_app::{Input, Key};
use regnum_renderer::Camera;

pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self { speed, sensitivity }
    }

    pub fn update(&mut self, camera: &mut Camera, input: &Input, dt: f32) {
        let (dx, dy) = input.mouse_delta();
        camera.yaw += dx * self.sensitivity;
        camera.pitch =
            (camera.pitch - dy * self.sensitivity).clamp(-89f32.to_radians(), 89f32.to_radians());

        let axis = |pos: Key, neg: Key| {
            input.key_down(pos) as i8 as f32 - input.key_down(neg) as i8 as f32
        };
        let wish = camera.forward() * axis(Key::W, Key::S) + camera.right() * axis(Key::D, Key::A);

        camera.position += wish.normalize_or_zero() * self.speed * dt;
    }
}
