use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use game::world::{
    VoxelWorld,
    block::Block,
    chunk::{Chunk, ChunkPos},
    registry::BlockRegistry,
};

use crate::mesher::{MeshBuffers, mesh_chunk};
use crate::models::{BlockModels, SIDE_DIRS};
use crate::visuals::BlockMaterials;

const SEA_LEVEL: i32 = 26;

pub struct DemoWorldPlugin;

impl Plugin for DemoWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_demo_world, spawn_chunks, mesh_chunks).chain(),
        );
    }
}

struct TerrainBlocks {
    grass: Block,
    dirt: Block,
    stone: Block,
    water: Block,
}

impl TerrainBlocks {
    fn new(registry: &BlockRegistry) -> Self {
        let block = |name: &str| {
            registry
                .block(name)
                .unwrap_or_else(|| panic!("missing block definition: {name}"))
        };
        Self {
            grass: block("grass_block"),
            dirt: block("dirt_block"),
            stone: block("stone_block"),
            water: registry
                .block_with("water", &[("level", 16)])
                .expect("missing block definition: water"),
        }
    }
}

fn spawn_demo_world(mut commands: Commands) {
    commands.spawn((
        VoxelWorld::new(),
        Transform::default(),
        Visibility::default(),
    ));
}

fn spawn_chunks(
    mut commands: Commands,
    registry: Res<BlockRegistry>,
    world: Single<(Entity, &mut VoxelWorld)>,
) {
    let (world_entity, mut world) = world.into_inner();
    let blocks = TerrainBlocks::new(&registry);

    for x in -3..3 {
        for y in 0..2 {
            for z in -3..3 {
                let pos = IVec3::new(x, y, z);
                world.insert_chunk(pos, demo_chunk(pos, &blocks));
                commands.spawn((
                    ChunkPos(pos),
                    Transform::from_translation(VoxelWorld::chunk_origin(pos).as_vec3()),
                    Visibility::default(),
                    ChildOf(world_entity),
                ));
            }
        }
    }
}

fn mesh_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    models: Res<BlockModels>,
    materials: Res<BlockMaterials>,
    world: Single<&VoxelWorld>,
    chunks: Query<(Entity, &ChunkPos)>,
) {
    for (entity, position) in &chunks {
        let Some(center) = world.chunk(position.0) else {
            continue;
        };
        let neighbors = SIDE_DIRS.map(|dir| world.chunk(position.0 + dir));
        let data = mesh_chunk(center, neighbors, &models);

        if !data.opaque.is_empty() {
            commands.entity(entity).insert((
                Mesh3d(meshes.add(to_bevy_mesh(&data.opaque))),
                MeshMaterial3d(materials.opaque.clone()),
            ));
        }
        if !data.transparent.is_empty() {
            let mesh = meshes.add(to_bevy_mesh(&data.transparent));
            let material = materials.transparent.clone();
            commands.entity(entity).with_children(|parent| {
                parent.spawn((Mesh3d(mesh), MeshMaterial3d(material)));
            });
        }
    }
}

fn demo_chunk(pos: IVec3, blocks: &TerrainBlocks) -> Chunk {
    let origin = pos * Chunk::SIZE as i32;
    let mut chunk = Chunk::new();

    for z in 0..Chunk::SIZE as u32 {
        for x in 0..Chunk::SIZE as u32 {
            let wx = (origin.x + x as i32) as f32;
            let wz = (origin.z + z as i32) as f32;
            let h = 28 + ((wx * 0.15).sin() * 4.0 + (wz * 0.11).cos() * 4.0) as i32;

            for y in 0..Chunk::SIZE as u32 {
                let wy = origin.y + y as i32;
                let block = if wy > h {
                    if wy > SEA_LEVEL {
                        continue;
                    }
                    blocks.water
                } else if wy == h {
                    blocks.grass
                } else if wy >= h - 3 {
                    blocks.dirt
                } else {
                    blocks.stone
                };
                chunk.set(UVec3::new(x, y, z), block);
            }
        }
    }
    chunk
}

fn to_bevy_mesh(buffers: &MeshBuffers) -> Mesh {
    let layers: Vec<[f32; 4]> = buffers
        .layers
        .iter()
        .map(|&layer| [layer as f32, 0.0, 0.0, 1.0])
        .collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, buffers.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, buffers.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, buffers.uvs.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, layers);
    mesh.insert_indices(Indices::U32(buffers.indices.clone()));
    mesh
}
