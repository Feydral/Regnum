pub mod block;
pub mod chunk_data;
pub mod registry;
pub mod world_data;

pub use world_data::WorldData;

pub struct World {
    data: WorldData,
}

impl World {
    pub fn new() -> Self {
        Self {
            data: WorldData::new(),
        }
    }

    pub fn data(&self) -> &WorldData {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut WorldData {
        &mut self.data
    }
}
