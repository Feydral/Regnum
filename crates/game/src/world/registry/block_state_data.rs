use crate::world::registry::{model::BlockModel, shape::BlockShape};

#[derive(Debug, Clone)]
pub struct BlockStateData {
    pub model: BlockModel,
    pub collision_shape: BlockShape,
}

impl BlockStateData {
    pub fn new(model: BlockModel, collision_shape: BlockShape) -> Self {
        Self {
            model,
            collision_shape,
        }
    }

    pub fn empty() -> Self {
        Self {
            model: BlockModel::empty(),
            collision_shape: BlockShape::empty(),
        }
    }
}
