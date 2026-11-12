#[derive(Clone, Copy, Default, Debug)]
pub struct Block(u32);

impl Block {
    pub const AIR: Self = Block(0);
    pub const GRASS: Self = Block(1);
    pub const DIRT: Self = Block(2);
    pub const STONE: Self = Block(3);
}
