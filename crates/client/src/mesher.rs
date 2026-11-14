use bevy::prelude::*;
use game::world::block::Block;
use game::world::chunk::Chunk;

use crate::models::{BlockModel, BlockModels, Face, MeshType, SIDE_DIRS};

pub type Neighbors<'a> = [Option<&'a Chunk>; 6];

#[derive(Default)]
pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub layers: Vec<u32>,
    pub indices: Vec<u32>,
}

impl MeshBuffers {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    fn push_face(&mut self, origin: Vec3, face: &Face) {
        let base = self.positions.len() as u32;
        for i in 0..4 {
            self.positions.push((origin + face.positions[i]).to_array());
            self.normals.push(face.normal.to_array());
            self.uvs.push(face.uvs[i].to_array());
            self.layers.push(face.layer);
        }
        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }
}

#[derive(Default)]
pub struct ChunkMeshData {
    pub opaque: MeshBuffers,
    pub transparent: MeshBuffers,
}

pub fn mesh_chunk(center: &Chunk, neighbors: Neighbors, models: &BlockModels) -> ChunkMeshData {
    let mut data = ChunkMeshData::default();
    let size = Chunk::SIZE as i32;

    for y in 0..size {
        for z in 0..size {
            for x in 0..size {
                let local = IVec3::new(x, y, z);
                let Some(block) = block_at(center, &neighbors, local) else {
                    continue;
                };
                if block.is_air() {
                    continue;
                }

                let model = models.model(block);
                let target = match model.mesh_type {
                    MeshType::None => continue,
                    MeshType::Transparent => &mut data.transparent,
                    MeshType::Opaque | MeshType::Unculled => &mut data.opaque,
                };

                for face in &model.faces {
                    if let Some(side) = face.side {
                        let neighbor = block_at(center, &neighbors, local + SIDE_DIRS[side]);
                        if is_culled(model, block, neighbor, models) {
                            continue;
                        }
                    }
                    target.push_face(local.as_vec3(), face);
                }
            }
        }
    }

    data
}

fn is_culled(
    model: &BlockModel,
    block: Block,
    neighbor: Option<Block>,
    models: &BlockModels,
) -> bool {
    let Some(neighbor) = neighbor else {
        return true;
    };
    let neighbor_model = models.model(neighbor);
    if neighbor_model.mesh_type == MeshType::Opaque && neighbor_model.full_cube {
        return true;
    }
    model.mesh_type == MeshType::Transparent && neighbor == block
}

fn block_at(center: &Chunk, neighbors: &Neighbors, local: IVec3) -> Option<Block> {
    let size = Chunk::SIZE as i32;
    let axis = |v: i32| {
        if v < 0 {
            -1
        } else if v >= size {
            1
        } else {
            0
        }
    };
    let dir = IVec3::new(axis(local.x), axis(local.y), axis(local.z));

    let chunk = if dir == IVec3::ZERO {
        center
    } else {
        let i = SIDE_DIRS.iter().position(|d| *d == dir)?;
        neighbors[i]?
    };

    chunk.get(local.rem_euclid(IVec3::splat(size)).as_uvec3())
}
