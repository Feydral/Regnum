use bevy::math::{IVec3, Vec3};

use crate::world::block::Block;
use crate::world::chunk::Chunk;

pub const NEIGHBOR_DIRS: [IVec3; 6] = [
    IVec3::NEG_Y,
    IVec3::Y,
    IVec3::NEG_X,
    IVec3::X,
    IVec3::NEG_Z,
    IVec3::Z,
];

const FACE_CORNERS: [[[f32; 3]; 4]; 6] = [
    [
        [0.0, 0.0, 0.0],
        [1.0, 0.0, 0.0],
        [1.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ],
    [
        [0.0, 1.0, 1.0],
        [1.0, 1.0, 1.0],
        [1.0, 1.0, 0.0],
        [0.0, 1.0, 0.0],
    ],
    [
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 1.0, 1.0],
        [0.0, 1.0, 0.0],
    ],
    [
        [1.0, 0.0, 1.0],
        [1.0, 0.0, 0.0],
        [1.0, 1.0, 0.0],
        [1.0, 1.0, 1.0],
    ],
    [
        [1.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [1.0, 1.0, 0.0],
    ],
    [
        [0.0, 0.0, 1.0],
        [1.0, 0.0, 1.0],
        [1.0, 1.0, 1.0],
        [0.0, 1.0, 1.0],
    ],
];

const QUAD_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];

pub type Neighbors<'a> = [Option<&'a Chunk>; 6];

#[derive(Default, Debug, Clone)]
pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub block_ids: Vec<u32>,
    pub faces: Vec<u32>,
    pub indices: Vec<u32>,
}

impl MeshBuffers {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }

    pub fn face_count(&self) -> usize {
        self.indices.len() / 6
    }

    fn push_quad(&mut self, local: IVec3, face: usize, block: Block) {
        let base = self.positions.len() as u32;
        let origin = local.as_vec3();
        let normal = NEIGHBOR_DIRS[face].as_vec3().to_array();

        for (corner, uv) in FACE_CORNERS[face].iter().zip(QUAD_UVS) {
            self.positions
                .push((origin + Vec3::from(*corner)).to_array());
            self.normals.push(normal);
            self.uvs.push(uv);
            self.block_ids.push(block.0);
            self.faces.push(face as u32);
        }

        self.indices
            .extend_from_slice(&[base, base + 1, base + 2, base + 2, base + 3, base]);
    }
}

pub fn mesh_chunk(center: &Chunk, neighbors: Neighbors) -> MeshBuffers {
    let mut data = MeshBuffers::default();

    for y in 0..Chunk::SIZE as i32 {
        for z in 0..Chunk::SIZE as i32 {
            for x in 0..Chunk::SIZE as i32 {
                let local = IVec3::new(x, y, z);
                let Some(block) = block_at(center, &neighbors, local) else {
                    continue;
                };

                if block == Block::AIR {
                    continue;
                }

                for face in 0..6 {
                    let neighbor = block_at(center, &neighbors, local + NEIGHBOR_DIRS[face]);
                    if neighbor.is_none_or(|b| b != Block::AIR) {
                        continue;
                    }
                    data.push_quad(local, face, block);
                }
            }
        }
    }

    data
}

fn block_at(center: &Chunk, neighbors: &Neighbors, local: IVec3) -> Option<Block> {
    let axis = |v: i32| {
        if v < 0 {
            -1
        } else if v >= Chunk::SIZE as i32 {
            1
        } else {
            0
        }
    };
    let dir = IVec3::new(axis(local.x), axis(local.y), axis(local.z));

    let chunk = if dir == IVec3::ZERO {
        center
    } else {
        let i = NEIGHBOR_DIRS.iter().position(|d| *d == dir)?;
        neighbors[i]?
    };

    chunk.get(
        local
            .rem_euclid(IVec3::splat(Chunk::SIZE as i32))
            .as_uvec3(),
    )
}
