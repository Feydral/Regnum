use glam::Vec3;
use serde::Deserialize;

use crate::world::registry::{
    property::{self, Property, ValueExpr},
    shape::cull_mask::CullMask,
};

#[derive(Debug, Clone, Copy, Deserialize)]
pub struct BoundingBox {
    pub min_x: u8,
    pub min_y: u8,
    pub min_z: u8,
    pub max_x: u8,
    pub max_y: u8,
    pub max_z: u8,
}

impl BoundingBox {
    pub const fn new(min_x: u8, min_y: u8, min_z: u8, max_x: u8, max_y: u8, max_z: u8) -> Self {
        Self {
            min_x,
            min_y,
            min_z,
            max_x,
            max_y,
            max_z,
        }
    }

    pub const fn full() -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            min_z: 0,
            max_x: 16,
            max_y: 16,
            max_z: 16,
        }
    }

    pub fn to_vec3_min(self) -> Vec3 {
        Vec3::new(
            self.min_x as f32 / 16.0,
            self.min_y as f32 / 16.0,
            self.min_z as f32 / 16.0,
        )
    }

    pub fn to_vec3_max(self) -> Vec3 {
        Vec3::new(
            self.max_x as f32 / 16.0,
            self.max_y as f32 / 16.0,
            self.max_z as f32 / 16.0,
        )
    }

    pub fn face_projection(&self, face: usize) -> (u8, u8, u8, u8) {
        match face {
            0 | 1 => (self.min_x, self.max_x, self.min_z, self.max_z),
            2 | 3 => (self.min_z, self.max_z, self.min_y, self.max_y),
            4 | 5 => (self.min_x, self.max_x, self.min_y, self.max_y),
            _ => (0, 0, 0, 0),
        }
    }

    pub fn covers_face(&self, face: usize) -> bool {
        let (u0, u1, v0, v1) = self.face_projection(face);
        if !(u0 == 0 && u1 == 16 && v0 == 0 && v1 == 16) {
            return false;
        }

        match face {
            0 => self.min_y == 0,
            1 => self.max_y == 16,
            2 => self.min_x == 0,
            3 => self.max_x == 16,
            4 => self.min_z == 0,
            5 => self.max_z == 16,
            _ => false,
        }
    }

    pub fn is_full_cube(&self) -> bool {
        self.min_x == 0
            && self.max_x == 16
            && self.min_y == 0
            && self.max_y == 16
            && self.min_z == 0
            && self.max_z == 16
    }

    pub fn to_cull_mask(self, face: usize) -> CullMask {
        let (u0, u1, v0, v1) = self.face_projection(face);
        let mut mask = CullMask::EMPTY;
        for v in v0..v1 {
            for u in u0..u1 {
                mask.set(u, v);
            }
        }
        mask
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoundingBoxJson {
    min_x: ValueExpr,
    min_y: ValueExpr,
    min_z: ValueExpr,
    max_x: ValueExpr,
    max_y: ValueExpr,
    max_z: ValueExpr,
}

impl BoundingBoxJson {
    pub fn resolve(&self, properties: &[Property], values: &[u32]) -> BoundingBox {
        BoundingBox {
            min_x: property::resolve(&self.min_x, properties, values) as u8,
            min_y: property::resolve(&self.min_y, properties, values) as u8,
            min_z: property::resolve(&self.min_z, properties, values) as u8,
            max_x: property::resolve(&self.max_x, properties, values) as u8,
            max_y: property::resolve(&self.max_y, properties, values) as u8,
            max_z: property::resolve(&self.max_z, properties, values) as u8,
        }
    }
}
