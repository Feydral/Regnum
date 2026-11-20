use glam::Vec3;
use regnum_assets::{AssetManager, Handle};
use regnum_renderer::{Camera, Frame, Mesh};
use winit::keyboard::KeyCode;

use crate::camera_controller::CameraController;

pub struct Game {
    camera: Camera,
    controller: CameraController,
    cube: Handle<Mesh>,
}

impl Game {
    pub fn new(assets: &mut AssetManager) -> Self {
        Self {
            camera: Camera::new(Vec3::new(0.0, 0.0, 2.0), -90.0, 0.0),
            controller: CameraController::new(2.0, 0.0022),
            cube: assets.add(Mesh::cube(0.5)),
        }
    }

    pub fn key(&mut self, code: KeyCode, pressed: bool, assets: &mut AssetManager) {
        if self.controller.process_key(code, pressed) || !pressed {
            return;
        }

        match code {
            KeyCode::KeyR => self.recolor_cube(assets),
            KeyCode::KeyX => self.toggle_cube(assets),
            _ => {}
        }
    }

    pub fn mouse_motion(&mut self, dx: f64, dy: f64) {
        self.controller.process_mouse(dx, dy);
    }

    pub fn release_input(&mut self) {
        self.controller.reset();
    }

    pub fn update(&mut self, dt: f32) {
        self.controller.update(&mut self.camera, dt);
    }

    pub fn render(&self) -> Frame {
        Frame {
            camera: self.camera.clone(),
            meshes: vec![self.cube],
        }
    }

    fn recolor_cube(&self, assets: &mut AssetManager) {
        if let Some(mesh) = assets.get_mut(self.cube) {
            for vertex in &mut mesh.vertices {
                let c = vertex.color;
                vertex.color = Vec3::new(c.z, c.x, c.y);
            }
        }
    }

    fn toggle_cube(&mut self, assets: &mut AssetManager) {
        if assets.remove(self.cube).is_none() {
            self.cube = assets.add(Mesh::cube(0.5));
        }
    }
}
