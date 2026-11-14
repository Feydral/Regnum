#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Block {
    pub id: u16,
    pub data: u16,
}

impl Block {
    pub const AIR: Block = Block { id: 0, data: 0 };

    pub const fn new(id: u16, data: u16) -> Self {
        Self { id, data }
    }

    pub const fn is_air(self) -> bool {
        self.id == 0
    }
}
