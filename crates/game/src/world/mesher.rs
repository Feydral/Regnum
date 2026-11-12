use bevy::math::{IVec3, UVec3};

use crate::world::{block::Block, chunk::Chunk};

pub const NEIGHBOR_DIRS: [IVec3; 6] = [
    IVec3::X,
    IVec3::NEG_X,
    IVec3::Y,
    IVec3::NEG_Y,
    IVec3::Z,
    IVec3::NEG_Z,
];

pub type Neighbors<'a> = [Option<&'a Chunk>; 6];

#[derive(Default, Debug)]
pub struct MeshBuffers {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub block_ids: Vec<u32>,
    pub indices: Vec<u32>,
}

impl MeshBuffers {
    pub fn is_empty(&self) -> bool {
        self.indices.is_empty()
    }
    pub fn face_count(&self) -> usize {
        self.indices.len() / 6
    }
}

struct Face {
    normal: [f32; 3],
    corners: [[f32; 3]; 4],
}

const FACES: [Face; 6] = [
    Face {
        normal: [1.0, 0.0, 0.0],
        corners: [[1., 0., 1.], [1., 0., 0.], [1., 1., 0.], [1., 1., 1.]],
    },
    Face {
        normal: [-1.0, 0.0, 0.0],
        corners: [[0., 0., 0.], [0., 0., 1.], [0., 1., 1.], [0., 1., 0.]],
    },
    Face {
        normal: [0.0, 1.0, 0.0],
        corners: [[0., 1., 1.], [1., 1., 1.], [1., 1., 0.], [0., 1., 0.]],
    },
    Face {
        normal: [0.0, -1.0, 0.0],
        corners: [[0., 0., 0.], [1., 0., 0.], [1., 0., 1.], [0., 0., 1.]],
    },
    Face {
        normal: [0.0, 0.0, 1.0],
        corners: [[0., 0., 1.], [1., 0., 1.], [1., 1., 1.], [0., 1., 1.]],
    },
    Face {
        normal: [0.0, 0.0, -1.0],
        corners: [[1., 0., 0.], [0., 0., 0.], [0., 1., 0.], [1., 1., 0.]],
    },
];

pub fn mesh_chunk(center: &Chunk, neighbors: Neighbors) -> MeshBuffers {
    let size = Chunk::SIZE as u32;
    let mut out = MeshBuffers::default();

    for y in 0..size {
        for z in 0..size {
            for x in 0..size {
                let Some(id) = center.get(UVec3::new(x, y, z)) else {
                    continue;
                };
                if id == Block::AIR {
                    continue;
                }
                let p = IVec3::new(x as i32, y as i32, z as i32);
                for (i, face) in FACES.iter().enumerate() {
                    if !is_solid(center, &neighbors, p + NEIGHBOR_DIRS[i]) {
                        push_face(&mut out, p, id, face);
                    }
                }
            }
        }
    }
    out
}

fn is_solid(center: &Chunk, neighbors: &Neighbors, p: IVec3) -> bool {
    let s = Chunk::SIZE as i32;

    if p.cmpge(IVec3::ZERO).all() && p.cmplt(IVec3::splat(s)).all() {
        return center.get(p.as_uvec3()).is_some_and(|b| b != Block::AIR);
    }

    let axis = |v: i32| {
        if v < 0 {
            -1
        } else if v >= s {
            1
        } else {
            0
        }
    };
    let dir = IVec3::new(axis(p.x), axis(p.y), axis(p.z));
    let Some(i) = NEIGHBOR_DIRS.iter().position(|d| *d == dir) else {
        return false;
    };
    let local = p.rem_euclid(IVec3::splat(s)).as_uvec3();
    neighbors[i]
        .and_then(|c| c.get(local))
        .is_some_and(|b| b != Block::AIR)
}

fn push_face(out: &mut MeshBuffers, p: IVec3, id: Block, face: &Face) {
    let base = out.positions.len() as u32;
    let origin = p.as_vec3();
    for corner in face.corners {
        out.positions
            .push((origin + bevy::math::Vec3::from(corner)).to_array());
        out.normals.push(face.normal);
        out.block_ids.push(id.0);
    }
    out.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}
