pub mod bounding_box;
pub mod cull_mask;

use bounding_box::BoundingBox;
use cull_mask::CullMask;
use serde::Deserialize;

use crate::world::registry::{property::Property, shape::bounding_box::BoundingBoxJson};

#[derive(Debug, Clone)]
pub struct BlockShape {
    pub boxes: Vec<BoundingBox>,
}

impl BlockShape {
    pub fn empty() -> Self {
        Self { boxes: vec![] }
    }

    pub fn todo() -> Self {
        Self { boxes: vec![] }
    }

    pub fn full() -> Self {
        Self {
            boxes: vec![BoundingBox::full()],
        }
    }

    pub fn custom(boxes: &[BoundingBox]) -> Self {
        Self {
            boxes: boxes.to_vec(),
        }
    }

    pub fn compute_cull_masks(&self) -> [CullMask; 6] {
        let mut masks = [CullMask::EMPTY; 6];
        for (face, mask) in masks.iter_mut().enumerate() {
            for bb in self.boxes.iter() {
                *mask = mask.union(bb.to_cull_mask(face));
            }
        }
        masks
    }

    pub fn covers_face(&self, face: usize) -> bool {
        self.boxes.iter().any(|b| b.covers_face(face))
    }

    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }

    pub fn is_full_cube(&self) -> bool {
        self.boxes.len() == 1 && self.boxes[0].is_full_cube()
    }

    pub fn opposite_face(face: usize) -> usize {
        match face {
            0 => 1,
            1 => 0,
            2 => 3,
            3 => 2,
            4 => 5,
            5 => 4,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockShapeJson {
    Empty,
    Full,
    Boxes(Vec<BoundingBoxJson>),
}

impl BlockShapeJson {
    pub fn resolve(&self, properties: &[Property], values: &[u32]) -> BlockShape {
        match self {
            Self::Empty => BlockShape::empty(),
            Self::Full => BlockShape::full(),
            Self::Boxes(boxes) => BlockShape::custom(
                &boxes
                    .iter()
                    .map(|b| b.resolve(properties, values))
                    .collect::<Vec<_>>(),
            ),
        }
    }
}
