use bevy::prelude::*;

use super::block::Block;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos(pub IVec3);

#[derive(Clone)]
pub struct Chunk {
    blocks: Box<[Block]>,
}

impl Chunk {
    pub const SIZE: usize = 32;
    pub const VOLUME: usize = Self::SIZE.pow(3);

    pub fn new() -> Self {
        Self {
            blocks: vec![Block::AIR; Self::VOLUME].into_boxed_slice(),
        }
    }

    fn index(p: UVec3) -> Option<usize> {
        let (x, y, z) = (p.x as usize, p.y as usize, p.z as usize);
        (x < Self::SIZE && y < Self::SIZE && z < Self::SIZE)
            .then(|| (y * Self::SIZE + z) * Self::SIZE + x)
    }

    pub fn get(&self, local: UVec3) -> Option<Block> {
        Self::index(local).map(|i| self.blocks[i])
    }

    pub fn set(&mut self, local: UVec3, block: Block) -> bool {
        match Self::index(local) {
            Some(i) => {
                self.blocks[i] = block;
                true
            }
            None => false,
        }
    }

    pub fn fill(&mut self, block: Block) {
        self.blocks.fill(block);
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}
