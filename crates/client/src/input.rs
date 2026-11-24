use std::collections::HashSet;

use winit::keyboard::KeyCode;

#[derive(Default)]
pub struct Input {
    down: HashSet<KeyCode>,
    mouse_delta: (f32, f32),
}

impl Input {
    pub fn key_down(&self, key: KeyCode) -> bool {
        self.down.contains(&key)
    }

    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    pub fn set_key(&mut self, key: KeyCode, down: bool) {
        if down {
            self.down.insert(key);
        } else {
            self.down.remove(&key);
        }
    }

    pub fn add_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
    }

    pub fn end_frame(&mut self) {
        self.mouse_delta = (0.0, 0.0);
    }

    pub fn clear(&mut self) {
        self.down.clear();
        self.end_frame();
    }
}
