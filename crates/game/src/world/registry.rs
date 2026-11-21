pub mod block_state_data;
pub mod definition;
pub mod model;
pub mod property;
pub mod shape;

use std::collections::HashMap;

use serde::Deserialize;

use crate::world::block::Block;
use crate::world::registry::{
    block_state_data::BlockStateData,
    definition::BlockDefinition,
    model::{BlockModel, BlockModelJson, MeshType, texture_rectangle::TextureRectangle},
    property::Property,
    shape::{BlockShape, BlockShapeJson},
};

#[derive(Debug, Deserialize)]
struct StateJson {
    #[serde(default)]
    when: HashMap<String, u32>,
    collision_shape: BlockShapeJson,
    model: BlockModelJson,
    mesh_type: MeshType,
}

#[derive(Debug, Deserialize)]
struct BlockJson {
    name: String,
    #[serde(default)]
    properties: Vec<Property>,
    #[serde(default)]
    states: Vec<StateJson>,
}

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

    pub fn load_from_directory(directory: &str) -> Self {
        let mut registry = Self::new();

        let mut paths: Vec<_> = match std::fs::read_dir(directory) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
                .collect(),
            Err(err) => {
                tracing::error!("could not read block directory '{directory}': {err}");
                Vec::new()
            }
        };
        paths.sort();

        for path in paths {
            let json = match std::fs::read_to_string(&path) {
                Ok(json) => json,
                Err(err) => {
                    tracing::error!("could not read block file '{}': {err}", path.display());
                    continue;
                }
            };

            let block: BlockJson = match serde_json::from_str(&json) {
                Ok(block) => block,
                Err(err) => {
                    tracing::error!("invalid block json '{}': {err}", path.display());
                    continue;
                }
            };

            if let Some(&existing_id) = registry.name_to_id.get(&block.name) {
                tracing::error!(
                    "duplicate block name '{}' in '{}' (already registered with id {existing_id}); overwriting",
                    block.name,
                    path.display()
                );
            }

            let id = registry.defs.len() as u16;
            let states = Self::build_states(&block);

            registry.name_to_id.insert(block.name.clone(), id);
            registry.defs.push(BlockDefinition {
                name: block.name,
                properties: block.properties,
                states,
            });
        }

        registry
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

    fn build_states(block: &BlockJson) -> Vec<BlockStateData> {
        let count = property::state_count(&block.properties).max(1);
        let mut states = Vec::with_capacity(count as usize);

        for data in 0..count {
            let values = property::decode(&block.properties, data);

            let matched = block
                .states
                .iter()
                .find(|state| property::matches_when(&state.when, &block.properties, &values));

            let state_data = match matched {
                Some(state) => {
                    let shape = state.collision_shape.resolve(&block.properties, &values);
                    let model = state
                        .model
                        .resolve(state.mesh_type, &block.properties, &values);
                    BlockStateData::new(model, shape)
                }
                None => BlockStateData::empty(),
            };

            states.push(state_data);
        }

        states
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
