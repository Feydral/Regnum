pub mod face;
pub mod texture_rectangle;

use serde::Deserialize;

use crate::world::registry::{
    model::{
        face::{Face, FaceJson},
        texture_rectangle::TextureRectangle,
    },
    property::Property,
    shape::{BlockShape, cull_mask::CullMask},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshType {
    None,
    Opaque,
    Transparent,
    Unculled,
}

#[derive(Debug, Clone)]
pub struct BlockModel {
    pub faces: Vec<Face>,
    pub mesh_type: MeshType,
    pub cull_masks: [CullMask; 6],
    pub is_full_cube: bool,
}

impl BlockModel {
    pub fn empty() -> Self {
        Self::build(vec![], MeshType::None)
    }

    pub fn cube_uniform(uv: TextureRectangle, mesh_type: MeshType) -> Self {
        Self::build(
            vec![
                Face::y_neg(uv),
                Face::y_pos(uv),
                Face::x_neg(uv),
                Face::x_pos(uv),
                Face::z_neg(uv),
                Face::z_pos(uv),
            ],
            mesh_type,
        )
    }

    pub fn cube_top_bottom_side(
        top: TextureRectangle,
        bottom: TextureRectangle,
        side: TextureRectangle,
        mesh_type: MeshType,
    ) -> Self {
        Self::build(
            vec![
                Face::y_neg(bottom),
                Face::y_pos(top),
                Face::x_neg(side),
                Face::x_pos(side),
                Face::z_neg(side),
                Face::z_pos(side),
            ],
            mesh_type,
        )
    }

    pub fn custom(faces: Vec<Face>, mesh_type: MeshType) -> Self {
        Self::build(faces, mesh_type)
    }

    pub fn cull_mask(&self, face: usize) -> CullMask {
        if self.mesh_type == MeshType::None {
            return CullMask::EMPTY;
        }
        self.cull_masks[face]
    }

    pub fn should_cull_face(&self, neighbor: &BlockModel, face_idx: usize) -> bool {
        let my_layer = self.mesh_type;
        let nb_layer = neighbor.mesh_type;

        match (my_layer, nb_layer) {
            (MeshType::None, _) | (_, MeshType::None) => false,
            (MeshType::Unculled, MeshType::Unculled) => {
                let my_mask = self.cull_mask(face_idx);
                let nb_mask = neighbor.cull_mask(BlockShape::opposite_face(face_idx));
                nb_mask.contains(my_mask)
            }
            (MeshType::Unculled, _) => false,
            (MeshType::Opaque, MeshType::Opaque)
            | (MeshType::Opaque, MeshType::Unculled)
            | (MeshType::Transparent, MeshType::Opaque)
            | (MeshType::Transparent, MeshType::Transparent)
            | (MeshType::Transparent, MeshType::Unculled) => {
                let my_mask = self.cull_mask(face_idx);
                let nb_mask = neighbor.cull_mask(BlockShape::opposite_face(face_idx));
                nb_mask.contains(my_mask)
            }
            _ => false,
        }
    }

    fn build(faces: Vec<Face>, mesh_type: MeshType) -> Self {
        let cull_masks = Self::compute_cull_masks(&faces);
        let is_full_cube =
            mesh_type != MeshType::None && cull_masks.iter().all(|m| m.contains(CullMask::FULL));

        Self {
            faces,
            mesh_type,
            cull_masks,
            is_full_cube,
        }
    }

    fn compute_cull_masks(faces: &[Face]) -> [CullMask; 6] {
        let mut masks = [CullMask::EMPTY; 6];
        for face in faces {
            if let Some(side) = face.side {
                masks[side as usize] = masks[side as usize].union(face.cull_mask());
            }
        }
        masks
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockModelJson {
    Empty,
    CubeUniform {
        uv: TextureRectangle,
    },
    CubeTopBottomSide {
        top: TextureRectangle,
        bottom: TextureRectangle,
        side: TextureRectangle,
    },
    Custom(Vec<FaceJson>),
}

impl BlockModelJson {
    pub fn resolve(
        &self,
        mesh_type: MeshType,
        properties: &[Property],
        values: &[u32],
    ) -> BlockModel {
        match self {
            Self::Empty => BlockModel::empty(),
            Self::CubeUniform { uv } => BlockModel::cube_uniform(*uv, mesh_type),
            Self::CubeTopBottomSide { top, bottom, side } => {
                BlockModel::cube_top_bottom_side(*top, *bottom, *side, mesh_type)
            }
            Self::Custom(faces) => {
                let faces = faces
                    .iter()
                    .map(|f| f.resolve(properties, values))
                    .collect();
                BlockModel::custom(faces, mesh_type)
            }
        }
    }
}
