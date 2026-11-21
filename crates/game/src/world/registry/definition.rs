use crate::world::registry::{block_state_data::BlockStateData, property::Property};

pub struct BlockDefinition {
    pub name: String,
    pub properties: Vec<Property>,
    pub states: Vec<BlockStateData>,
}
