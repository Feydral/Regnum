use std::collections::HashSet;

use winit::keyboard::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    W,
    A,
    S,
    D,
    R,
    X,
}

impl Key {
    pub(crate) fn from_winit(code: KeyCode) -> Option<Self> {
        match code {
            KeyCode::KeyW => Some(Self::W),
            KeyCode::KeyA => Some(Self::A),
            KeyCode::KeyS => Some(Self::S),
            KeyCode::KeyD => Some(Self::D),
            KeyCode::KeyR => Some(Self::R),
            KeyCode::KeyX => Some(Self::X),
            _ => None,
        }
    }
}

#[derive(Default)]
pub struct Input {
    down: HashSet<Key>,
    pressed: HashSet<Key>,
    mouse_delta: (f32, f32),
}

impl Input {
    pub fn key_down(&self, key: Key) -> bool {
        self.down.contains(&key)
    }

    pub fn key_pressed(&self, key: Key) -> bool {
        self.pressed.contains(&key)
    }

    pub fn mouse_delta(&self) -> (f32, f32) {
        self.mouse_delta
    }

    pub(crate) fn set_key(&mut self, key: Key, down: bool) {
        if down {
            if self.down.insert(key) {
                self.pressed.insert(key);
            }
        } else {
            self.down.remove(&key);
        }
    }

    pub(crate) fn add_mouse_delta(&mut self, dx: f32, dy: f32) {
        self.mouse_delta.0 += dx;
        self.mouse_delta.1 += dy;
    }

    pub(crate) fn end_frame(&mut self) {
        self.pressed.clear();
        self.mouse_delta = (0.0, 0.0);
    }

    pub(crate) fn clear(&mut self) {
        self.down.clear();
        self.end_frame();
    }
}
