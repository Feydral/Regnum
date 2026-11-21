#[derive(Debug, Clone, Copy)]
pub struct CullMask(pub [u128; 2]);

impl CullMask {
    pub const FULL: Self = Self([u128::MAX, u128::MAX]);
    pub const EMPTY: Self = Self([0, 0]);

    pub fn set(&mut self, u: u8, v: u8) {
        let (word, shift) = Self::index(u, v);
        self.0[word] |= 1u128 << shift;
    }

    pub fn is_set(&self, u: u8, v: u8) -> bool {
        let (word, shift) = Self::index(u, v);
        (self.0[word] >> shift) & 1 != 0
    }

    pub fn contains(&self, other: CullMask) -> bool {
        (self.0[0] & other.0[0]) == other.0[0] && (self.0[1] & other.0[1]) == other.0[1]
    }

    pub fn union(&self, other: CullMask) -> Self {
        Self([self.0[0] | other.0[0], self.0[1] | other.0[1]])
    }

    fn index(u: u8, v: u8) -> (usize, u32) {
        let bit = (v as u32) * 16 + (u as u32);
        ((bit / 128) as usize, bit % 128)
    }
}
