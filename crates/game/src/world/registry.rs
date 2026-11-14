pub mod property;
pub mod shape;

use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bevy::prelude::*;
use serde::Deserialize;
use tracing::{error, warn};

use super::block::Block;
use property::{Property, decode, encode, matches_when, validate_properties, validate_when};
use shape::{BlockShape, BlockShapeJson};

pub const AIR_NAME: &str = "air";

#[derive(Debug, Clone)]
pub struct BlockState {
    pub collision: BlockShape,
}

#[derive(Debug, Clone)]
pub struct BlockDefinition {
    pub name: String,
    pub properties: Vec<Property>,
    pub states: Vec<BlockState>,
}

impl BlockDefinition {
    pub fn air() -> Self {
        Self {
            name: AIR_NAME.to_string(),
            properties: Vec::new(),
            states: vec![BlockState {
                collision: BlockShape::empty(),
            }],
        }
    }
}

struct RegistryData {
    definitions: Vec<BlockDefinition>,
    ids: HashMap<String, u16>,
}

#[derive(Resource, Clone)]
pub struct BlockRegistry(Arc<RegistryData>);

impl BlockRegistry {
    pub fn from_definitions(definitions: Vec<BlockDefinition>) -> Self {
        let mut all = vec![BlockDefinition::air()];
        let mut ids = HashMap::from([(AIR_NAME.to_string(), 0u16)]);

        for definition in definitions {
            if definition.name == AIR_NAME {
                continue;
            }
            if ids.contains_key(&definition.name) {
                error!("duplicate block '{}' ignored", definition.name);
                continue;
            }
            if all.len() > u16::MAX as usize {
                error!("too many blocks, '{}' ignored", definition.name);
                continue;
            }
            ids.insert(definition.name.clone(), all.len() as u16);
            all.push(definition);
        }

        Self(Arc::new(RegistryData {
            definitions: all,
            ids,
        }))
    }

    pub fn load_from_directory(dir: &Path) -> io::Result<Self> {
        let mut paths: Vec<PathBuf> = fs::read_dir(dir)?
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
            .collect();
        paths.sort();

        let mut definitions = Vec::new();
        for path in paths {
            let text = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(err) => {
                    error!("{}: {err}", path.display());
                    continue;
                }
            };
            let file: BlockFile = match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
                Ok(file) => file,
                Err(err) => {
                    error!("{}: {err}", path.display());
                    continue;
                }
            };
            match build_definition(file) {
                Ok(definition) => definitions.push(definition),
                Err(err) => error!("{}: {err}", path.display()),
            }
        }

        Ok(Self::from_definitions(definitions))
    }

    pub fn len(&self) -> usize {
        self.0.definitions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.definitions.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (u16, &BlockDefinition)> {
        self.0
            .definitions
            .iter()
            .enumerate()
            .map(|(id, definition)| (id as u16, definition))
    }

    pub fn id(&self, name: &str) -> Option<u16> {
        self.0.ids.get(name).copied()
    }

    pub fn block(&self, name: &str) -> Option<Block> {
        self.id(name).map(|id| Block::new(id, 0))
    }

    pub fn block_with(&self, name: &str, values: &[(&str, u32)]) -> Option<Block> {
        values
            .iter()
            .try_fold(self.block(name)?, |block, (property, value)| {
                self.with_property(block, property, *value)
            })
    }

    pub fn definition(&self, block: Block) -> Option<&BlockDefinition> {
        self.0.definitions.get(block.id as usize)
    }

    pub fn name(&self, block: Block) -> Option<&str> {
        self.definition(block).map(|d| d.name.as_str())
    }

    pub fn state(&self, block: Block) -> Option<&BlockState> {
        self.definition(block)?.states.get(block.data as usize)
    }

    pub fn is_solid(&self, block: Block) -> bool {
        self.state(block)
            .is_some_and(|state| !state.collision.is_empty())
    }

    pub fn property(&self, block: Block, name: &str) -> Option<u32> {
        let definition = self.definition(block)?;
        let index = definition.properties.iter().position(|p| p.name == name)?;
        decode(&definition.properties, block.data as u32)
            .get(index)
            .copied()
    }

    pub fn with_property(&self, block: Block, name: &str, value: u32) -> Option<Block> {
        let definition = self.definition(block)?;
        let index = definition.properties.iter().position(|p| p.name == name)?;
        if value >= definition.properties[index].values {
            return None;
        }
        let mut values = decode(&definition.properties, block.data as u32);
        values[index] = value;
        Some(Block::new(
            block.id,
            encode(&definition.properties, &values) as u16,
        ))
    }
}

#[derive(Deserialize)]
struct BlockFile {
    name: String,
    #[serde(default)]
    properties: Vec<Property>,
    #[serde(default)]
    states: Vec<StateFile>,
}

#[derive(Deserialize)]
struct StateFile {
    #[serde(default)]
    when: HashMap<String, u32>,
    #[serde(default)]
    collision_shape: BlockShapeJson,
}

fn build_definition(file: BlockFile) -> Result<BlockDefinition, String> {
    if file.name.is_empty() {
        return Err("block has no name".to_string());
    }
    let count = validate_properties(&file.properties)?;
    for state in &file.states {
        validate_when(&state.when, &file.properties)?;
    }

    let mut states = Vec::with_capacity(count as usize);
    let mut unmatched = 0u32;

    for data in 0..count {
        let values = decode(&file.properties, data);
        let matched = file
            .states
            .iter()
            .find(|state| matches_when(&state.when, &file.properties, &values));

        match matched {
            Some(state) => states.push(BlockState {
                collision: state
                    .collision_shape
                    .resolve(&file.properties, &values)
                    .map_err(|err| format!("block '{}': {err}", file.name))?,
            }),
            None => {
                unmatched += 1;
                states.push(BlockState {
                    collision: BlockShape::empty(),
                });
            }
        }
    }

    if unmatched > 0 {
        warn!(
            "block '{}': {unmatched} of {count} states match no entry and use an empty shape",
            file.name
        );
    }

    Ok(BlockDefinition {
        name: file.name,
        properties: file.properties,
        states,
    })
}
