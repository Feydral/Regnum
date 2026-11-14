use serde::Deserialize;

use super::property::{Property, ValueExpr};

pub const BOX_UNITS: u32 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    pub min: [u8; 3],
    pub max: [u8; 3],
}

impl BoundingBox {
    pub const FULL: BoundingBox = BoundingBox {
        min: [0, 0, 0],
        max: [16, 16, 16],
    };
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BlockShape {
    pub boxes: Vec<BoundingBox>,
}

impl BlockShape {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn full() -> Self {
        Self {
            boxes: vec![BoundingBox::FULL],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockShapeJson {
    #[default]
    Empty,
    Full,
    Boxes(Vec<BoundingBoxJson>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct BoundingBoxJson {
    pub min_x: ValueExpr,
    pub min_y: ValueExpr,
    pub min_z: ValueExpr,
    pub max_x: ValueExpr,
    pub max_y: ValueExpr,
    pub max_z: ValueExpr,
}

impl BlockShapeJson {
    pub fn resolve(&self, properties: &[Property], values: &[u32]) -> Result<BlockShape, String> {
        match self {
            BlockShapeJson::Empty => Ok(BlockShape::empty()),
            BlockShapeJson::Full => Ok(BlockShape::full()),
            BlockShapeJson::Boxes(boxes) => boxes
                .iter()
                .map(|b| b.resolve(properties, values))
                .collect::<Result<Vec<_>, _>>()
                .map(|boxes| BlockShape { boxes }),
        }
    }
}

impl BoundingBoxJson {
    fn resolve(&self, properties: &[Property], values: &[u32]) -> Result<BoundingBox, String> {
        let coordinate = |expr: &ValueExpr| -> Result<u8, String> {
            let value = expr.resolve(properties, values)?;
            if value > BOX_UNITS {
                return Err(format!("box coordinate {value} is outside 0..={BOX_UNITS}"));
            }
            Ok(value as u8)
        };

        let bounding_box = BoundingBox {
            min: [
                coordinate(&self.min_x)?,
                coordinate(&self.min_y)?,
                coordinate(&self.min_z)?,
            ],
            max: [
                coordinate(&self.max_x)?,
                coordinate(&self.max_y)?,
                coordinate(&self.max_z)?,
            ],
        };

        if (0..3).any(|axis| bounding_box.min[axis] > bounding_box.max[axis]) {
            return Err("box minimum is larger than its maximum".to_string());
        }
        Ok(bounding_box)
    }
}
