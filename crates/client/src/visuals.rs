use std::path::PathBuf;

use bevy::prelude::*;
use game::world::registry::BlockRegistry;

use crate::material::VoxelMaterial;
use crate::models::BlockModels;
use crate::textures::{TileAllocator, build_tile_array};

pub struct BlockVisualsPlugin {
    assets_dir: PathBuf,
}

impl BlockVisualsPlugin {
    pub fn new(assets_dir: impl Into<PathBuf>) -> Self {
        Self {
            assets_dir: assets_dir.into(),
        }
    }
}

#[derive(Resource)]
pub struct BlockMaterials {
    pub opaque: Handle<VoxelMaterial>,
    pub transparent: Handle<VoxelMaterial>,
}

impl Plugin for BlockVisualsPlugin {
    fn build(&self, app: &mut App) {
        let registry = app.world().resource::<BlockRegistry>().clone();

        let mut tiles = TileAllocator::default();
        let models = BlockModels::load(&self.assets_dir.join("blocks"), &registry, &mut tiles);
        let array = build_tile_array(
            &self.assets_dir.join("textures/TextureAtlas.png"),
            tiles.tiles(),
        );

        app.add_plugins(MaterialPlugin::<VoxelMaterial>::default());

        let tiles = app.world_mut().resource_mut::<Assets<Image>>().add(array);
        let mut materials = app.world_mut().resource_mut::<Assets<VoxelMaterial>>();
        let opaque = materials.add(VoxelMaterial {
            tiles: tiles.clone(),
            tint: LinearRgba::WHITE,
            alpha_mode: AlphaMode::Opaque,
        });
        let transparent = materials.add(VoxelMaterial {
            tiles,
            tint: LinearRgba::new(1.0, 1.0, 1.0, 0.65),
            alpha_mode: AlphaMode::Blend,
        });

        app.insert_resource(models).insert_resource(BlockMaterials {
            opaque,
            transparent,
        });
    }
}
