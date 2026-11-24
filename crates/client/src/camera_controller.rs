use regnum_renderer::Camera;
use winit::keyboard::KeyCode;

use crate::input::Input;

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

        let axis = |pos: KeyCode, neg: KeyCode| {
            input.key_down(pos) as i8 as f32 - input.key_down(neg) as i8 as f32
        };
        let wish = camera.forward() * axis(KeyCode::KeyW, KeyCode::KeyS)
            + camera.right() * axis(KeyCode::KeyD, KeyCode::KeyA);

        camera.position += wish.normalize_or_zero() * self.speed * dt;
    }
}
