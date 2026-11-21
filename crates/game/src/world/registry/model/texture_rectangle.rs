use glam::UVec2;
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct TextureRectangle {
    pub min: UVec2,
    pub max: UVec2,
}

impl TextureRectangle {
    pub const fn new(min: UVec2, max: UVec2) -> Self {
        Self { min, max }
    }

    pub const fn tile(col: u32, row: u32) -> Self {
        Self {
            min: UVec2::new(col * 16, row * 16),
            max: UVec2::new(col * 16 + 16, row * 16 + 16),
        }
    }
}
