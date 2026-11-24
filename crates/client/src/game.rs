use std::collections::HashMap;

use glam::{IVec3, Vec3};
use regnum_assets::{AssetManager, Handle, Mesh, Model, Texture, Transform};
use regnum_game::world::{
    WorldData, chunk_data::ChunkData, mesher::generate_mesh, registry::BlockRegistry, terrain,
};
use regnum_renderer::{Camera, Frame};

use crate::{camera_controller::CameraController, input::Input};

const ATLAS_PATH: &str = "assets/textures/TextureAtlas.png";
const BLOCKS_PATH: &str = "assets/blocks";
const WORLD_RADIUS: i32 = 2;

pub struct Game {
    camera: Camera,
    controller: CameraController,
    atlas: Option<Handle<Texture>>,
    registry: BlockRegistry,
    world: WorldData,
    chunk_meshes: HashMap<IVec3, Handle<Mesh>>,
}

impl Game {
    pub fn new() -> Self {
        Self {
            camera: Camera::new(Vec3::new(16.0, 40.0, 120.0), -90.0, -20.0),
            controller: CameraController::new(20.0, 0.0022),
            atlas: None,
            registry: BlockRegistry::load_from_directory(BLOCKS_PATH),
            world: WorldData::new(),
            chunk_meshes: HashMap::new(),
        }
    }

    fn generate_world(&mut self) {
        for z in -WORLD_RADIUS..=WORLD_RADIUS {
            for x in -WORLD_RADIUS..=WORLD_RADIUS {
                let position = IVec3::new(x, 0, z);
                self.world
                    .set_chunk(position, terrain::generate_chunk(&self.registry, position));
            }
        }
    }

    fn mesh_world(&mut self, assets: &mut AssetManager) {
        for position in self.world.iter_loaded_positions() {
            let Some(chunk) = self.world.get_chunk_arc(position) else {
                continue;
            };

            let neighbors = ChunkData::NEIGHBOR_OFFSETS
                .map(|offset| self.world.get_chunk_arc(position + offset));

            let mesh = generate_mesh(chunk, neighbors, &self.registry);
            if mesh.indices.is_empty() {
                continue;
            }

            self.chunk_meshes.insert(position, assets.add(mesh));
        }
    }

    pub fn load(&mut self, assets: &mut AssetManager) {
        let atlas = Texture::load_png(ATLAS_PATH)
            .unwrap_or_else(|_| Texture::new(1, 1, vec![255, 0, 255, 255]));
        self.atlas = Some(assets.add(atlas));

        self.generate_world();
        self.mesh_world(assets);
    }

    pub fn unload(&mut self, assets: &mut AssetManager) {
        if let Some(atlas) = self.atlas.take() {
            assets.remove(atlas);
        }

        for (_, mesh) in self.chunk_meshes.drain() {
            assets.remove(mesh);
        }

        let positions: Vec<IVec3> = self.world.iter_loaded_positions().collect();
        for position in positions {
            self.world.remove_chunk(position);
        }
    }

    pub fn update(&mut self, dt: f32, input: &Input) {
        self.controller.update(&mut self.camera, input, dt);
    }

    pub fn render(&self) -> Frame {
        let models = self
            .atlas
            .map(|texture| {
                self.chunk_meshes
                    .iter()
                    .map(|(&position, &mesh)| Model {
                        mesh,
                        texture,
                        transform: Transform::from_position(
                            WorldData::chunk_to_world(position).as_vec3(),
                        ),
                    })
                    .collect()
            })
            .unwrap_or_default();

        Frame {
            camera: self.camera.clone(),
            models,
        }
    }
}
