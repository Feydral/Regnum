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

pub struct DemoWorldPlugin;

impl Plugin for DemoWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_demo_world);
    }
}

fn spawn_demo_world(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient: ResMut<GlobalAmbientLight>,
) {
    ambient.brightness = 500.0;

    let chunk_pos = IVec3::ZERO;
    let mut world = VoxelWorld::new();
    world.insert_chunk(chunk_pos, demo_chunk());

    let snapshot = world.chunk_arc(chunk_pos).unwrap();
    let buffers = mesh_chunk(&snapshot, [None; 6]);

    let world_entity = commands
        .spawn((world, Transform::default(), Visibility::default()))
        .id();

    commands.spawn((
        ChunkPos(chunk_pos),
        Mesh3d(meshes.add(to_bevy_mesh(&buffers))),
        MeshMaterial3d(materials.add(StandardMaterial {
            perceptual_roughness: 1.0,
            reflectance: 0.0,
            ..default()
        })),
        Transform::from_translation(VoxelWorld::chunk_origin(chunk_pos).as_vec3()),
        ChildOf(world_entity),
    ));
}

fn demo_chunk() -> Chunk {
    let (stone, dirt, grass) = (Block(1), Block(2), Block(3));
    let mut chunk = Chunk::new();
    for z in 0..Chunk::SIZE as u32 {
        for x in 0..Chunk::SIZE as u32 {
            let h = 12.0 + (x as f32 * 0.35).sin() * 3.0 + (z as f32 * 0.25).cos() * 3.0;
            let h = h as u32;
            for y in 0..=h {
                let id = if y == h {
                    grass
                } else if y + 3 >= h {
                    dirt
                } else {
                    stone
                };
                chunk.set(UVec3::new(x, y, z), id);
            }
        }
    }
    chunk
}

fn to_bevy_mesh(b: &MeshBuffers) -> Mesh {
    let colors: Vec<[f32; 4]> = b.block_ids.iter().map(|&id| block_color(id)).collect();

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, b.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, b.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(b.indices.clone()));
    mesh
}

fn block_color(id: u32) -> [f32; 4] {
    match id {
        1 => [0.5, 0.5, 0.52, 1.0],
        2 => [0.45, 0.30, 0.18, 1.0],
        3 => [0.30, 0.65, 0.25, 1.0],
        _ => [1.0, 0.0, 1.0, 1.0],
    }
}
