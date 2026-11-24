use glam::IVec3;
use regnum_game::world::{block::Block, chunk_data::ChunkData, registry::BlockRegistry};

const STONE_TOP: i32 = 16;
const DIRT_TOP: i32 = 20;
const GRASS_LEVEL: i32 = 20;

pub fn generate_chunk(registry: &BlockRegistry, position: IVec3) -> ChunkData {
    let air = Block::by_name("air", registry);
    let stone = Block::by_name("stone_block", registry);
    let dirt = Block::by_name("dirt_block", registry);
    let grass = Block::by_name("grass_block", registry);

    let size = ChunkData::SIZE as i32;
    let mut blocks = Vec::with_capacity(ChunkData::VOLUME as usize);

    for _z in 0..size {
        for y in 0..size {
            let world_y = position.y * size + y;
            let block = if world_y < STONE_TOP {
                stone
            } else if world_y < DIRT_TOP {
                dirt
            } else if world_y == GRASS_LEVEL {
                grass
            } else {
                air
            };

            for _x in 0..size {
                blocks.push(block);
            }
        }
    }

    ChunkData::from_blocks(&blocks).expect("block buffer has the chunk volume")
}
