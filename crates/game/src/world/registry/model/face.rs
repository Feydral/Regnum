use glam::{IVec3, Vec3};
use serde::Deserialize;

use crate::world::registry::{
    model::texture_rectangle::TextureRectangle,
    property::{self, Property, ValueExpr},
    shape::cull_mask::CullMask,
};

#[derive(Debug, Clone, Deserialize)]
pub struct Face {
    pub positions: [IVec3; 4],
    pub normal: Vec3,
    pub uv: TextureRectangle,

    pub side: Option<u8>,
}

impl Face {
    pub fn new(positions: [IVec3; 4], uv: TextureRectangle) -> Self {
        Self {
            normal: Self::compute_normal(&positions),
            side: Self::compute_side(&positions),
            positions,
            uv,
        }
    }

    pub fn y_neg(uv: TextureRectangle) -> Self {
        Self::new(
            [
                IVec3::new(0, 0, 0),
                IVec3::new(16, 0, 0),
                IVec3::new(16, 0, 16),
                IVec3::new(0, 0, 16),
            ],
            uv,
        )
    }

    pub fn y_pos(uv: TextureRectangle) -> Self {
        Self::new(
            [
                IVec3::new(0, 16, 0),
                IVec3::new(0, 16, 16),
                IVec3::new(16, 16, 16),
                IVec3::new(16, 16, 0),
            ],
            uv,
        )
    }

    pub fn x_neg(uv: TextureRectangle) -> Self {
        Self::new(
            [
                IVec3::new(0, 0, 0),
                IVec3::new(0, 0, 16),
                IVec3::new(0, 16, 16),
                IVec3::new(0, 16, 0),
            ],
            uv,
        )
    }

    pub fn x_pos(uv: TextureRectangle) -> Self {
        Self::new(
            [
                IVec3::new(16, 0, 16),
                IVec3::new(16, 0, 0),
                IVec3::new(16, 16, 0),
                IVec3::new(16, 16, 16),
            ],
            uv,
        )
    }

    pub fn z_neg(uv: TextureRectangle) -> Self {
        Self::new(
            [
                IVec3::new(16, 0, 0),
                IVec3::new(0, 0, 0),
                IVec3::new(0, 16, 0),
                IVec3::new(16, 16, 0),
            ],
            uv,
        )
    }

    pub fn z_pos(uv: TextureRectangle) -> Self {
        Self::new(
            [
                IVec3::new(0, 0, 16),
                IVec3::new(16, 0, 16),
                IVec3::new(16, 16, 16),
                IVec3::new(0, 16, 16),
            ],
            uv,
        )
    }

    pub fn to_vec3_positions(&self) -> [Vec3; 4] {
        std::array::from_fn(|i| self.positions[i].as_vec3() / 16.0)
    }

    pub fn get_atlas_uvs(&self, atlas_size: u32) -> [[f32; 2]; 4] {
        let u0 = self.uv.min.x as f32 / atlas_size as f32;
        let v0 = self.uv.min.y as f32 / atlas_size as f32;
        let u1 = self.uv.max.x as f32 / atlas_size as f32;
        let v1 = self.uv.max.y as f32 / atlas_size as f32;
        [[u0, v1], [u1, v1], [u1, v0], [u0, v0]]
    }

    pub fn cull_mask(&self) -> CullMask {
        let Some(side) = self.side else {
            return CullMask::EMPTY;
        };

        #[allow(clippy::type_complexity)]
        let (a, b): (fn(&IVec3) -> i32, fn(&IVec3) -> i32) = match side {
            0 | 1 => (|p| p.x, |p| p.z),
            2 | 3 => (|p| p.z, |p| p.y),
            4 | 5 => (|p| p.x, |p| p.y),
            _ => unreachable!(),
        };

        let a_vals = self.positions.map(|p| a(&p));
        let b_vals = self.positions.map(|p| b(&p));

        let a_min = *a_vals.iter().min().unwrap();
        let a_max = *a_vals.iter().max().unwrap();
        let b_min = *b_vals.iter().min().unwrap();
        let b_max = *b_vals.iter().max().unwrap();

        let mut mask = CullMask::EMPTY;
        for v in b_min..b_max {
            for u in a_min..a_max {
                mask.set(u as u8, v as u8);
            }
        }
        mask
    }

    fn compute_normal(p: &[IVec3; 4]) -> Vec3 {
        let p0 = p[0].as_vec3();
        let edge1 = p[1].as_vec3() - p0;
        let edge2 = p[2].as_vec3() - p0;
        edge1.cross(edge2).normalize()
    }

    fn compute_side(p: &[IVec3; 4]) -> Option<u8> {
        let const_axis = |get: fn(&IVec3) -> i32| {
            let v0 = get(&p[0]);
            p.iter().all(|q| get(q) == v0).then_some(v0)
        };

        if let Some(x) = const_axis(|v| v.x) {
            if x == 0 {
                return Some(2);
            }
            if x == 16 {
                return Some(3);
            }
        }
        if let Some(y) = const_axis(|v| v.y) {
            if y == 0 {
                return Some(0);
            }
            if y == 16 {
                return Some(1);
            }
        }
        if let Some(z) = const_axis(|v| v.z) {
            if z == 0 {
                return Some(4);
            }
            if z == 16 {
                return Some(5);
            }
        }
        None
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FaceJson {
    positions: [[ValueExpr; 3]; 4],
    uv: TextureRectangle,
}

impl FaceJson {
    pub fn resolve(&self, properties: &[Property], values: &[u32]) -> Face {
        let positions: [IVec3; 4] = std::array::from_fn(|i| {
            let [x, y, z] = &self.positions[i];
            IVec3::new(
                property::resolve(x, properties, values) as i32,
                property::resolve(y, properties, values) as i32,
                property::resolve(z, properties, values) as i32,
            )
        });

        Face::new(positions, self.uv)
    }
}
