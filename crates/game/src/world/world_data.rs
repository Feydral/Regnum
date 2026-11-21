use std::collections::HashMap;
use std::sync::Arc;

use glam::{IVec3, UVec3};

use crate::world::{block::Block, chunk_data::ChunkData};

pub struct WorldData {
    chunks: HashMap<IVec3, Arc<ChunkData>>,
}

impl WorldData {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
        }
    }

    pub fn get_chunk(&self, position: IVec3) -> Option<&ChunkData> {
        self.chunks.get(&position).map(Arc::as_ref)
    }

    pub fn get_chunk_arc(&self, position: IVec3) -> Option<Arc<ChunkData>> {
        self.chunks.get(&position).cloned()
    }

    pub fn get_chunk_mut(&mut self, position: IVec3) -> Option<&mut ChunkData> {
        self.chunks.get_mut(&position).map(Arc::make_mut)
    }

    pub fn get_or_create_chunk(&mut self, position: IVec3) -> &ChunkData {
        &*self.get_or_create_chunk_mut(position)
    }

    pub fn get_or_create_chunk_mut(&mut self, position: IVec3) -> &mut ChunkData {
        Arc::make_mut(
            self.chunks
                .entry(position)
                .or_insert_with(|| Arc::new(ChunkData::new())),
        )
    }

    pub fn set_chunk(&mut self, position: IVec3, chunk: ChunkData) {
        self.chunks.insert(position, Arc::new(chunk));
    }

    pub fn remove_chunk(&mut self, position: IVec3) -> Option<Arc<ChunkData>> {
        self.chunks.remove(&position)
    }

    pub fn contains_chunk(&self, position: IVec3) -> bool {
        self.chunks.contains_key(&position)
    }

    pub fn get_block(&self, world_position: IVec3) -> Option<&Block> {
        let chunk_pos = Self::world_to_chunk(world_position);
        let local_pos = Self::world_to_local(world_position);
        self.get_chunk(chunk_pos)?.get_block(local_pos)
    }

    pub fn set_block(&mut self, world_position: IVec3, block: Block) -> bool {
        let chunk_pos = Self::world_to_chunk(world_position);
        let local_pos = Self::world_to_local(world_position);
        let Some(chunk) = self.get_chunk_mut(chunk_pos) else {
            return false;
        };
        chunk.set_block(local_pos, block)
    }

    pub fn set_block_or_create(&mut self, world_position: IVec3, block: Block) {
        let chunk_pos = Self::world_to_chunk(world_position);
        let local_pos = Self::world_to_local(world_position);
        self.get_or_create_chunk_mut(chunk_pos)
            .set_block(local_pos, block);
    }

    pub fn iter_loaded_positions(&self) -> impl Iterator<Item = IVec3> + '_ {
        self.chunks.keys().copied()
    }

    pub fn world_to_chunk(world_position: IVec3) -> IVec3 {
        let size = ChunkData::SIZE as i32;
        IVec3::new(
            world_position.x.div_euclid(size),
            world_position.y.div_euclid(size),
            world_position.z.div_euclid(size),
        )
    }

    pub fn world_to_local(world_position: IVec3) -> UVec3 {
        let size = ChunkData::SIZE as i32;
        UVec3::new(
            world_position.x.rem_euclid(size) as u32,
            world_position.y.rem_euclid(size) as u32,
            world_position.z.rem_euclid(size) as u32,
        )
    }

    pub fn chunk_to_world(chunk_position: IVec3) -> IVec3 {
        chunk_position * ChunkData::SIZE as i32
    }
}
