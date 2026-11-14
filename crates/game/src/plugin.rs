use std::path::PathBuf;

use bevy::prelude::*;

use crate::world::registry::BlockRegistry;

pub struct GamePlugin {
    blocks_dir: PathBuf,
}

impl GamePlugin {
    pub fn new(blocks_dir: impl Into<PathBuf>) -> Self {
        Self {
            blocks_dir: blocks_dir.into(),
        }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        let registry =
            BlockRegistry::load_from_directory(&self.blocks_dir).unwrap_or_else(|error| {
                panic!(
                    "cannot load block definitions from {}: {error}",
                    self.blocks_dir.display()
                )
            });
        app.insert_resource(registry);
    }
}
