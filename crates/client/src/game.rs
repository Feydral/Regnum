use glam::Vec3;
use regnum_app::{Application, Input, Key};
use regnum_assets::{AssetManager, Handle};
use regnum_renderer::{Camera, Frame, Mesh};

use crate::camera_controller::CameraController;

pub struct Game {
    camera: Camera,
    controller: CameraController,
    cube: Option<Handle<Mesh>>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(Vec3::new(0.0, 0.0, 2.0), -90.0, 0.0),
            controller: CameraController::new(2.0, 0.0022),
            cube: None,
        }
    }

    fn recolor_cube(&self, assets: &mut AssetManager) {
        let Some(cube) = self.cube else { return };

        if let Some(mesh) = assets.get_mut(cube) {
            for vertex in &mut mesh.vertices {
                let c = vertex.color;
                vertex.color = Vec3::new(c.z, c.x, c.y);
            }
        }
    }

    fn toggle_cube(&mut self, assets: &mut AssetManager) {
        match self.cube.take() {
            Some(cube) => {
                assets.remove(cube);
            }
            None => self.cube = Some(assets.add(Mesh::cube(0.5))),
        }
    }
}

impl Application for Game {
    fn load(&mut self, assets: &mut AssetManager) {
        self.cube = Some(assets.add(Mesh::cube(0.5)));
    }

    fn unload(&mut self, assets: &mut AssetManager) {
        if let Some(cube) = self.cube.take() {
            assets.remove(cube);
        }
    }
    
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
            meshes: self.cube.into_iter().collect(),
        }
    }
}
