use regnum_renderer::Camera;
use winit::keyboard::KeyCode;

pub struct CameraController {
    pub speed: f32,
    pub sensitivity: f32,

    forward: bool,
    back: bool,
    left: bool,
    right: bool,
    mouse_delta: (f32, f32),
}

impl CameraController {
    pub fn new(speed: f32, sensitivity: f32) -> Self {
        Self {
            speed,
            sensitivity,
            forward: false,
            back: false,
            left: false,
            right: false,
            mouse_delta: (0.0, 0.0),
        }
    }

    pub fn process_key(&mut self, code: KeyCode, pressed: bool) -> bool {
        let flag = match code {
            KeyCode::KeyW => &mut self.forward,
            KeyCode::KeyS => &mut self.back,
            KeyCode::KeyA => &mut self.left,
            KeyCode::KeyD => &mut self.right,
            _ => return false,
        };
        *flag = pressed;
        true
    }

    pub fn process_mouse(&mut self, dx: f64, dy: f64) {
        self.mouse_delta.0 += dx as f32;
        self.mouse_delta.1 += dy as f32;
    }

    pub fn reset(&mut self) {
        *self = Self::new(self.speed, self.sensitivity);
    }

    pub fn update(&mut self, camera: &mut Camera, dt: f32) {
        let (dx, dy) = std::mem::take(&mut self.mouse_delta);
        camera.yaw += dx * self.sensitivity;
        camera.pitch =
            (camera.pitch - dy * self.sensitivity).clamp(-89f32.to_radians(), 89f32.to_radians());

        let axis = |pos: bool, neg: bool| pos as i8 as f32 - neg as i8 as f32;
        let wish = camera.forward() * axis(self.forward, self.back)
            + camera.right() * axis(self.right, self.left);

        camera.position += wish.normalize_or_zero() * self.speed * dt;
    }
}
