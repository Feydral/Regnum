pub mod block;
pub mod chunk;
pub mod mesher;

use std::{collections::HashMap, sync::Arc};

use bevy::ecs::component::Component;
use bevy::math::{IVec3, UVec3};

use crate::world::{block::Block, chunk::Chunk};

#[derive(Component, Default)]
pub struct VoxelWorld {
    chunks: HashMap<IVec3, Arc<Chunk>>,
}

impl VoxelWorld {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn world_to_chunk(p: IVec3) -> IVec3 {
        let s = Chunk::SIZE as i32;
        IVec3::new(p.x.div_euclid(s), p.y.div_euclid(s), p.z.div_euclid(s))
    }

    pub fn world_to_local(p: IVec3) -> UVec3 {
        let s = Chunk::SIZE as i32;
        UVec3::new(
            p.x.rem_euclid(s) as u32,
            p.y.rem_euclid(s) as u32,
            p.z.rem_euclid(s) as u32,
        )
    }

    pub fn chunk_origin(chunk: IVec3) -> IVec3 {
        chunk * Chunk::SIZE as i32
    }

    pub fn chunk(&self, pos: IVec3) -> Option<&Chunk> {
        self.chunks.get(&pos).map(Arc::as_ref)
    }

    pub fn chunk_arc(&self, pos: IVec3) -> Option<Arc<Chunk>> {
        self.chunks.get(&pos).cloned()
    }

    pub fn contains_chunk(&self, pos: IVec3) -> bool {
        self.chunks.contains_key(&pos)
    }

    pub fn insert_chunk(&mut self, pos: IVec3, chunk: Chunk) {
        self.chunks.insert(pos, Arc::new(chunk));
    }

    pub fn remove_chunk(&mut self, pos: IVec3) -> Option<Arc<Chunk>> {
        self.chunks.remove(&pos)
    }

    pub fn block(&self, p: IVec3) -> Option<Block> {
        self.chunk(Self::world_to_chunk(p))?
            .get(Self::world_to_local(p))
    }

    pub fn set_block(&mut self, p: IVec3, id: Block) -> bool {
        let Some(chunk) = self.chunks.get_mut(&Self::world_to_chunk(p)) else {
            return false;
        };
        Arc::make_mut(chunk).set(Self::world_to_local(p), id)
    }
}
