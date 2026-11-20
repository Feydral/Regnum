use glam::Vec3;
use regnum_app::{Application, Input, Key};
use regnum_assets::{AssetManager, Handle};
use regnum_renderer::{Camera, Frame, Mesh};

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

impl Application for Game {
    fn update(&mut self, dt: f32, input: &Input, assets: &mut AssetManager) {
        self.controller.update(&mut self.camera, input, dt);

        if input.key_pressed(Key::R) {
            self.recolor_cube(assets);
        }
        if input.key_pressed(Key::X) {
            self.toggle_cube(assets);
        }
    }

    fn render(&self) -> Frame {
        Frame {
            camera: self.camera.clone(),
            meshes: vec![self.cube],
        }
    }
}
