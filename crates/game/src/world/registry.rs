pub mod block_state_data;
pub mod definition;
pub mod model;
pub mod property;
pub mod shape;

use std::collections::HashMap;

use crate::world::block::Block;
use crate::world::registry::{
    block_state_data::BlockStateData,
    definition::BlockDefinition,
    model::{BlockModel, MeshType, texture_rectangle::TextureRectangle},
    shape::BlockShape,
};

pub struct BlockRegistry {
    name_to_id: HashMap<String, u16>,
    defs: Vec<BlockDefinition>,
}

impl BlockRegistry {
    pub fn new() -> Self {
        let mut name_to_id = HashMap::new();
        let mut defs = Vec::new();
        Self::default_blocks(&mut name_to_id, &mut defs);

        Self { name_to_id, defs }
    }

    fn default_blocks(name_to_id: &mut HashMap<String, u16>, defs: &mut Vec<BlockDefinition>) {
        let fallback_uv = TextureRectangle::tile(0, 0);

        let default_blocks = [
            (
                "default_unculled",
                BlockStateData::new(
                    BlockModel::cube_uniform(fallback_uv, MeshType::Unculled),
                    BlockShape::full(),
                ),
            ),
            (
                "default_opaque",
                BlockStateData::new(
                    BlockModel::cube_uniform(fallback_uv, MeshType::Opaque),
                    BlockShape::full(),
                ),
            ),
            (
                "default_transparent",
                BlockStateData::new(
                    BlockModel::cube_uniform(fallback_uv, MeshType::Transparent),
                    BlockShape::full(),
                ),
            ),
            (
                "default_invisible",
                BlockStateData::new(BlockModel::empty(), BlockShape::empty()),
            ),
        ];

        for (name, state) in default_blocks {
            let id = defs.len() as u16;
            name_to_id.insert(name.to_string(), id);
            defs.push(BlockDefinition {
                name: name.to_string(),
                properties: Vec::new(),
                states: vec![state],
            });
        }
    }
}

impl Block {
    pub fn by_name(name: &str, registry: &BlockRegistry) -> Self {
        match registry.name_to_id.get(name) {
            Some(&id) => Self { id, data: 0 },
            None => {
                tracing::error!("tried creating unknown block name='{name}'");
                Self::fallback(registry)
            }
        }
    }

    pub fn by_id(id: u32, registry: &BlockRegistry) -> Self {
        if id < registry.defs.len() as u32 {
            Self {
                id: id as u16,
                data: 0,
            }
        } else {
            tracing::error!("tried creating unknown block id={id}");
            Self::fallback(registry)
        }
    }

    pub fn with_property(mut self, name: &str, value: u32, registry: &BlockRegistry) -> Self {
        match self.resolved_data_with_property(name, value, registry) {
            Some(data) => {
                self.data = data;
                self
            }
            None => Self::fallback(registry),
        }
    }

    pub fn modify_property(
        &mut self,
        name: &str,
        value: u32,
        registry: &BlockRegistry,
    ) -> &mut Self {
        match self.resolved_data_with_property(name, value, registry) {
            Some(data) => self.data = data,
            None => *self = Self::fallback(registry),
        }
        self
    }

    pub fn get_property(&self, name: &str, registry: &BlockRegistry) -> Option<u32> {
        let def = registry.defs.get(self.id as usize)?;
        let idx = def.properties.iter().position(|p| p.name == name)?;
        Some(property::decode(&def.properties, self.data as u32)[idx])
    }

    pub fn name<'a>(&self, registry: &'a BlockRegistry) -> &'a str {
        match registry.defs.get(self.id as usize) {
            Some(def) => &def.name,
            None => {
                tracing::error!("unknown block id '{}'", self.id);
                "default_opaque"
            }
        }
    }

    pub fn state<'a>(&self, registry: &'a BlockRegistry) -> &'a BlockStateData {
        registry
            .defs
            .get(self.id as usize)
            .and_then(|def| def.states.get(self.data as usize))
            .unwrap_or_else(|| {
                tracing::error!("unknown block state (id={}, data={})", self.id, self.data);
                let fallback_id = Self::fallback(registry).id;
                &registry.defs[fallback_id as usize].states[0]
            })
    }

    fn resolved_data_with_property(
        &self,
        name: &str,
        value: u32,
        registry: &BlockRegistry,
    ) -> Option<u16> {
        let Some(def) = registry.defs.get(self.id as usize) else {
            tracing::error!("unknown block id '{}'", self.id);
            return None;
        };

        let Some(idx) = def.properties.iter().position(|p| p.name == name) else {
            tracing::error!("block '{}' has no property '{name}'", def.name);
            return None;
        };

        let property = &def.properties[idx];
        if value >= property.values {
            tracing::error!(
                "block '{}' property '{name}' value {value} out of range (0..{})",
                def.name,
                property.values
            );
            return None;
        }

        let mut values = property::decode(&def.properties, self.data as u32);
        values[idx] = value;
        Some(property::encode(&def.properties, &values) as u16)
    }

    fn fallback(registry: &BlockRegistry) -> Self {
        let id = *registry
            .name_to_id
            .get("default_opaque")
            .expect("default_opaque is always registered by default_blocks");
        Self { id, data: 0 }
    }
}
