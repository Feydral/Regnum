use std::sync::Arc;

use glam::{IVec3, UVec3, Vec2, Vec3};
use regnum_assets::{Mesh, Vertex};

use crate::world::{
    block::Block,
    chunk_data::ChunkData,
    registry::{
        BlockRegistry,
        model::{BlockModel, MeshType, face::Face},
    },
};

pub type Neighbors = [Option<Arc<ChunkData>>; 6];

const PADDED: usize = ChunkData::SIZE as usize + 2;

const ATLAS_SIZE: u32 = 256;
const UV_INSET: f32 = (1.0 / 64.0) / ATLAS_SIZE as f32;
const UV_INSET_DIRECTIONS: [Vec2; 4] = [
    Vec2::new(1.0, -1.0),
    Vec2::new(-1.0, -1.0),
    Vec2::new(-1.0, 1.0),
    Vec2::new(1.0, 1.0),
];

pub fn generate_mesh(
    chunk: Arc<ChunkData>,
    neighbors: Neighbors,
    registry: &BlockRegistry,
) -> Mesh {
    let mut mesh = Mesh::default();

    if chunk.palette_len() == 1
        && let Some(block) = chunk.get_block(UVec3::ZERO)
        && block.state(registry).model.mesh_type == MeshType::None
    {
        return mesh;
    }

    let models = padded_models(&chunk, &neighbors, registry);
    let strides = ChunkData::NEIGHBOR_OFFSETS.map(|o| {
        o.x as isize + o.y as isize * PADDED as isize + o.z as isize * (PADDED * PADDED) as isize
    });

    for z in 1..=ChunkData::SIZE as usize {
        for y in 1..=ChunkData::SIZE as usize {
            for x in 1..=ChunkData::SIZE as usize {
                let index = (z * PADDED + y) * PADDED + x;
                let model = models[index];

                if model.mesh_type == MeshType::None {
                    continue;
                }

                let origin = Vec3::new((x - 1) as f32, (y - 1) as f32, (z - 1) as f32);

                for face in &model.faces {
                    if let Some(side) = face.side {
                        let neighbor = models[index.wrapping_add_signed(strides[side as usize])];
                        if (std::ptr::eq(model, neighbor) && model.is_full_cube)
                            || model.should_cull_face(neighbor, side as usize)
                        {
                            continue;
                        }
                    }

                    push_face(&mut mesh, origin, face);
                }
            }
        }
    }

    mesh
}

fn padded_models<'a>(
    chunk: &ChunkData,
    neighbors: &Neighbors,
    registry: &'a BlockRegistry,
) -> Vec<&'a BlockModel> {
    let origin = chunk
        .get_block(UVec3::ZERO)
        .expect("origin is inside the chunk");
    let solid = &Block::by_name("default_opaque", registry)
        .state(registry)
        .model;
    let mut models = vec![&origin.state(registry).model; PADDED * PADDED * PADDED];

    for (position, block) in chunk.iter_blocks() {
        models[padded_index(position.as_ivec3())] = &block.state(registry).model;
    }

    for (side, neighbor) in neighbors.iter().enumerate() {
        let fixed = if side % 2 == 0 {
            -1
        } else {
            ChunkData::SIZE as i32
        };

        for a in 0..ChunkData::SIZE as i32 {
            for b in 0..ChunkData::SIZE as i32 {
                let local = match side / 2 {
                    0 => IVec3::new(a, fixed, b),
                    1 => IVec3::new(fixed, a, b),
                    _ => IVec3::new(a, b, fixed),
                };

                models[padded_index(local)] = neighbor.as_ref().map_or(solid, |neighbor| {
                    let block = neighbor
                        .get_block(
                            local
                                .rem_euclid(IVec3::splat(ChunkData::SIZE as i32))
                                .as_uvec3(),
                        )
                        .expect("position is wrapped into the chunk");
                    &block.state(registry).model
                });
            }
        }
    }

    models
}

fn padded_index(local: IVec3) -> usize {
    let padded = (local + IVec3::ONE).as_uvec3();
    (padded.z as usize * PADDED + padded.y as usize) * PADDED + padded.x as usize
}

fn push_face(mesh: &mut Mesh, origin: Vec3, face: &Face) {
    let base = mesh.vertices.len() as u32;
    let uvs = face.get_atlas_uvs(ATLAS_SIZE);

    for ((position, uv), inset) in face
        .to_vec3_positions()
        .into_iter()
        .zip(uvs)
        .zip(UV_INSET_DIRECTIONS)
    {
        mesh.vertices.push(Vertex {
            position: origin + position,
            normal: face.normal,
            uv: Vec2::from(uv) + inset * UV_INSET,
        });
    }

    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
