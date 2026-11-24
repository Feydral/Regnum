use glam::{IVec3, Vec3};
use regnum_app::{Application, Input, Key};
use regnum_assets::{AssetManager, Handle, Mesh, Model, Texture, Transform};
use regnum_game::world::{WorldData, registry::BlockRegistry};
use regnum_renderer::{Camera, Frame};

use crate::{camera_controller::CameraController, terrain};

const ATLAS_PATH: &str = "assets/textures/TextureAtlas.png";
const BLOCKS_PATH: &str = "assets/blocks";
const WORLD_RADIUS: i32 = 2;

pub struct Game {
    camera: Camera,
    controller: CameraController,
    cube: Option<Handle<Mesh>>,
    atlas: Option<Handle<Texture>>,
    registry: BlockRegistry,
    world: WorldData,
}

impl Game {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(Vec3::new(0.0, 0.0, 2.0), -90.0, 0.0),
            controller: CameraController::new(2.0, 0.0022),
            cube: None,
            atlas: None,
            registry: BlockRegistry::load_from_directory(BLOCKS_PATH),
            world: WorldData::new(),
        }
    }

    fn recolor_cube(&self, assets: &mut AssetManager) {
        let Some(cube) = self.cube else { return };

        if let Some(mesh) = assets.get_mut(cube) {
            for vertex in &mut mesh.vertices {
                let n = vertex.normal;
                vertex.normal = Vec3::new(n.z, n.x, n.y);
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
        let atlas = Texture::load_png(ATLAS_PATH)
            .unwrap_or_else(|_| Texture::new(1, 1, vec![255, 0, 255, 255]));
        self.atlas = Some(assets.add(atlas));

        for z in -WORLD_RADIUS..=WORLD_RADIUS {
            for x in -WORLD_RADIUS..=WORLD_RADIUS {
                let position = IVec3::new(x, 0, z);
                self.world
                    .set_chunk(position, terrain::generate_chunk(&self.registry, position));
            }
        }
    }

    fn unload(&mut self, assets: &mut AssetManager) {
        if let Some(cube) = self.cube.take() {
            assets.remove(cube);
        }
        if let Some(atlas) = self.atlas.take() {
            assets.remove(atlas);
        }

        let positions: Vec<IVec3> = self.world.iter_loaded_positions().collect();
        for position in positions {
            self.world.remove_chunk(position);
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
            models: self
                .cube
                .zip(self.atlas)
                .map(|(mesh, texture)| Model {
                    mesh,
                    texture,
                    transform: Transform::IDENTITY,
                })
                .into_iter()
                .collect(),
        }
    }
}
