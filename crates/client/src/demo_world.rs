use bevy::asset::RenderAssetUsages;
use bevy::light::GlobalAmbientLight;
use bevy::math::UVec3;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use game::world::{
    VoxelWorld,
    block::Block,
    chunk::{Chunk, ChunkPos},
    mesher::{MeshBuffers, mesh_chunk},
};

use crate::demo_texturing::{build_tile_array, tile_index};
use crate::material::VoxelMaterial;

pub struct DemoWorldPlugin;

impl Plugin for DemoWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            (spawn_demo_world, spawn_chunks, mesh_chunks).chain(),
        );
    }
}

fn spawn_demo_world(mut commands: Commands, mut ambient: ResMut<GlobalAmbientLight>) {
    ambient.brightness = 500.0;

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        VoxelWorld::new(),
        Transform::default(),
        Visibility::default(),
    ));
}

fn spawn_chunks(mut commands: Commands, world: Single<(Entity, &mut VoxelWorld)>) {
    let (world_entity, mut world) = world.into_inner();

    for x in -3..3 {
        for y in -3..3 {
            for z in -3..3 {
                let chunk_position = IVec3::new(x, y, z);
                world.insert_chunk(chunk_position, demo_chunk(chunk_position));
                commands.spawn((
                    ChunkPos(chunk_position),
                    Transform::from_translation(VoxelWorld::chunk_origin(chunk_position).as_vec3()),
                    ChildOf(world_entity),
                ));
            }
        }
    }
}

fn mesh_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VoxelMaterial>>,
    mut images: ResMut<Assets<Image>>,
    world: Single<&VoxelWorld>,
    chunks: Query<(Entity, &ChunkPos)>,
) {
    let material = materials.add(VoxelMaterial {
        tiles: images.add(build_tile_array()),
    });

    for (entity, position) in &chunks {
        let Some(center) = world.chunk(position.0) else {
            continue;
        };
        let neighbors = game::world::mesher::NEIGHBOR_DIRS.map(|d| world.chunk(position.0 + d));

        let buffers = mesh_chunk(center, neighbors);
        if buffers.is_empty() {
            continue;
        }

        commands.entity(entity).insert((
            Mesh3d(meshes.add(to_bevy_mesh(&buffers))),
            MeshMaterial3d(material.clone()),
        ));
    }
}

fn demo_chunk(pos: IVec3) -> Chunk {
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
                    continue;
                } else if wy == h {
                    Block::GRASS
                } else if wy >= h - 3 {
                    Block::DIRT
                } else {
                    Block::STONE
                };
                chunk.set(UVec3::new(x, y, z), block);
            }
        }
    }
    chunk
}

fn to_bevy_mesh(b: &MeshBuffers) -> Mesh {
    let uvs: Vec<[f32; 2]> = b.uvs.iter().map(|&[u, v]| [u, 1.0 - v]).collect();
    let layers: Vec<[f32; 4]> = (0..b.positions.len())
        .map(|i| [tile_index(b.block_ids[i], b.faces[i]) as f32, 0.0, 0.0, 1.0])
        .collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, b.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, b.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, layers);
    mesh.insert_indices(Indices::U32(b.indices.clone()));
    mesh
}
