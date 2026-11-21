use std::collections::HashMap;

use glam::{IVec3, UVec3};

use crate::world::block::Block;

#[derive(Clone)]
pub struct ChunkData {
    palette: Vec<Block>,
    palette_index: HashMap<Block, u32>,
    bits_per_index: u8,
    data: Vec<u32>,
}

impl ChunkData {
    pub const SIZE: u32 = 32;

    pub const VOLUME: u32 = Self::SIZE * Self::SIZE * Self::SIZE;

    pub const PALETTE_LIMIT: usize = Self::VOLUME as usize;

    pub const NEIGHBOR_OFFSETS: [IVec3; 6] = [
        IVec3::new(0, -1, 0),
        IVec3::new(0, 1, 0),
        IVec3::new(-1, 0, 0),
        IVec3::new(1, 0, 0),
        IVec3::new(0, 0, -1),
        IVec3::new(0, 0, 1),
    ];

    const BITS_PER_WORD: u32 = 32;

    pub fn new() -> Self {
        let mut palette_index = HashMap::new();
        palette_index.insert(Block::default(), 0);
        Self {
            palette: vec![Block::default()],
            palette_index,
            bits_per_index: 0,
            data: Vec::new(),
        }
    }

    pub fn get_block(&self, position: UVec3) -> Option<&Block> {
        let flat_idx = Self::index(position)?;
        let palette_idx = self.read_index(flat_idx);
        self.palette.get(palette_idx as usize)
    }

    pub fn set_block(&mut self, position: UVec3, block: Block) -> bool {
        let Some(flat_idx) = Self::index(position) else {
            return false;
        };

        let palette_idx = if let Some(&idx) = self.palette_index.get(&block) {
            idx
        } else {
            if self.palette.len() >= Self::PALETTE_LIMIT {
                self.compact();

                if let Some(&idx) = self.palette_index.get(&block) {
                    self.write_index(flat_idx, idx);
                    return true;
                }
            }

            let idx = self.palette.len() as u32;
            self.palette.push(block);
            self.palette_index.insert(block, idx);

            let required_bits = Self::bits_for_palette_len(self.palette.len());
            if required_bits != self.bits_per_index {
                self.repack(required_bits);
            }

            idx
        };

        self.write_index(flat_idx, palette_idx);
        true
    }

    pub fn fill(&mut self, block: Block) {
        self.palette = vec![block];
        self.palette_index.clear();
        self.palette_index.insert(block, 0);
        self.bits_per_index = 0;
        self.data = Vec::new();
    }

    pub fn is_all(&self, block: Block) -> bool {
        if self.palette.len() == 1 {
            return self.palette[0] == block;
        }

        (0..Self::VOLUME).all(|flat_idx| {
            let palette_idx = self.read_index(flat_idx);
            self.palette[palette_idx as usize] == block
        })
    }

    pub fn palette_len(&self) -> usize {
        self.palette.len()
    }

    pub fn bits_per_index(&self) -> u8 {
        self.bits_per_index
    }

    pub fn compact(&mut self) {
        let mut remap = vec![None; self.palette.len()];
        let mut new_palette = Vec::new();

        for flat_idx in 0..Self::VOLUME {
            let old_idx = self.read_index(flat_idx);
            if remap[old_idx as usize].is_none() {
                let new_idx = new_palette.len() as u32;
                new_palette.push(self.palette[old_idx as usize]);
                remap[old_idx as usize] = Some(new_idx);
            }
        }

        if new_palette.len() == self.palette.len() {
            return;
        }

        let new_bits = Self::bits_for_palette_len(new_palette.len());
        let old_bits = self.bits_per_index;

        if new_bits == 0 {
            self.palette = new_palette;
            self.bits_per_index = 0;
            self.data = Vec::new();
            return;
        }

        let mut new_data = vec![0u32; Self::word_count(new_bits)];

        for flat_idx in 0..Self::VOLUME {
            let old_idx = if old_bits == 0 {
                0
            } else {
                self.read_index(flat_idx)
            };
            let new_idx = remap[old_idx as usize].unwrap();
            Self::write_index_into(&mut new_data, new_bits, flat_idx, new_idx);
        }

        self.palette = new_palette;
        self.bits_per_index = new_bits;
        self.data = new_data;

        self.palette_index.clear();
        for (i, block) in self.palette.iter().enumerate() {
            self.palette_index.insert(*block, i as u32);
        }
    }

    pub fn index(position: UVec3) -> Option<u32> {
        if position.x < Self::SIZE && position.y < Self::SIZE && position.z < Self::SIZE {
            Some(position.x + Self::SIZE * (position.y + Self::SIZE * position.z))
        } else {
            None
        }
    }

    pub fn coords(index: u32) -> Option<UVec3> {
        if index >= Self::VOLUME {
            return None;
        }
        let z = index / (Self::SIZE * Self::SIZE);
        let y = (index % (Self::SIZE * Self::SIZE)) / Self::SIZE;
        let x = index % Self::SIZE;
        Some(UVec3::new(x, y, z))
    }

    pub fn iter_blocks(&self) -> impl Iterator<Item = (UVec3, &Block)> + '_ {
        (0..Self::VOLUME).map(move |flat_idx| {
            let palette_idx = self.read_index(flat_idx);
            (
                Self::coords(flat_idx).unwrap(),
                &self.palette[palette_idx as usize],
            )
        })
    }

    fn bits_for_palette_len(palette_len: usize) -> u8 {
        match palette_len {
            0 | 1 => 0,
            2 => 1,
            3..=4 => 2,
            5..=16 => 4,
            17..=256 => 8,
            _ => 16,
        }
    }

    fn word_count(bits: u8) -> usize {
        if bits == 0 {
            return 0;
        }
        let total_bits = Self::VOLUME as u64 * bits as u64;
        total_bits.div_ceil(Self::BITS_PER_WORD as u64) as usize
    }

    fn read_index(&self, flat_idx: u32) -> u32 {
        if self.bits_per_index == 0 {
            return 0;
        }
        Self::read_index_from(&self.data, self.bits_per_index, flat_idx)
    }

    fn write_index(&mut self, flat_idx: u32, value: u32) {
        if self.bits_per_index == 0 {
            debug_assert_eq!(value, 0);
            return;
        }
        Self::write_index_into(&mut self.data, self.bits_per_index, flat_idx, value);
    }

    fn read_index_from(data: &[u32], bits: u8, flat_idx: u32) -> u32 {
        let bits = bits as u64;
        let bit_pos = flat_idx as u64 * bits;
        let word_idx = (bit_pos / Self::BITS_PER_WORD as u64) as usize;
        let bit_offset = (bit_pos % Self::BITS_PER_WORD as u64) as u32;

        let mask = if bits == 32 {
            u32::MAX
        } else {
            (1u32 << bits) - 1
        };

        let low = data[word_idx] >> bit_offset;

        if bit_offset + bits as u32 <= Self::BITS_PER_WORD {
            low & mask
        } else {
            let bits_from_low = Self::BITS_PER_WORD - bit_offset;
            let high = data[word_idx + 1] << bits_from_low;
            (low | high) & mask
        }
    }

    fn write_index_into(data: &mut [u32], bits: u8, flat_idx: u32, value: u32) {
        let bits = bits as u64;
        let bit_pos = flat_idx as u64 * bits;
        let word_idx = (bit_pos / Self::BITS_PER_WORD as u64) as usize;
        let bit_offset = (bit_pos % Self::BITS_PER_WORD as u64) as u32;

        let mask = if bits == 32 {
            u32::MAX
        } else {
            (1u32 << bits) - 1
        };
        let value = value & mask;

        data[word_idx] &= !(mask << bit_offset);
        data[word_idx] |= value << bit_offset;

        if bit_offset + bits as u32 > Self::BITS_PER_WORD {
            let bits_from_low = Self::BITS_PER_WORD - bit_offset;
            let bits_from_high = bits as u32 - bits_from_low;
            let high_mask = (1u32 << bits_from_high) - 1;

            data[word_idx + 1] &= !high_mask;
            data[word_idx + 1] |= value >> bits_from_low;
        }
    }

    fn repack(&mut self, new_bits: u8) {
        let old_bits = self.bits_per_index;

        if new_bits == 0 {
            self.data = Vec::new();
            self.bits_per_index = 0;
            return;
        }

        let mut new_data = vec![0u32; Self::word_count(new_bits)];

        if old_bits != 0 {
            for flat_idx in 0..Self::VOLUME {
                let value = Self::read_index_from(&self.data, old_bits, flat_idx);
                Self::write_index_into(&mut new_data, new_bits, flat_idx, value);
            }
        }

        self.data = new_data;
        self.bits_per_index = new_bits;
    }
}
