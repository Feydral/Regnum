use std::collections::HashMap;
use std::path::Path;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use image::RgbaImage;
use tracing::error;

pub const TILE_SIZE: u32 = 16;

#[derive(Default)]
pub struct TileAllocator {
    tiles: Vec<UVec2>,
    layers: HashMap<UVec2, u32>,
}

impl TileAllocator {
    pub fn layer(&mut self, tile: UVec2) -> u32 {
        if let Some(&layer) = self.layers.get(&tile) {
            return layer;
        }
        let layer = self.tiles.len() as u32;
        self.tiles.push(tile);
        self.layers.insert(tile, layer);
        layer
    }

    pub fn tiles(&self) -> &[UVec2] {
        &self.tiles
    }
}

pub fn build_tile_array(atlas_path: &Path, tiles: &[UVec2]) -> Image {
    let atlas = match image::open(atlas_path) {
        Ok(image) => Some(image.to_rgba8()),
        Err(err) => {
            error!("{}: {err}", atlas_path.display());
            None
        }
    };

    let layers = tiles.len().max(1) as u32;
    let mut data = Vec::with_capacity((TILE_SIZE * TILE_SIZE * 4 * layers) as usize);

    for index in 0..layers as usize {
        let copied = match (tiles.get(index), &atlas) {
            (Some(&tile), Some(atlas)) => copy_tile(atlas, tile, &mut data),
            _ => false,
        };
        if !copied {
            push_checker_tile(&mut data);
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: TILE_SIZE,
            height: TILE_SIZE * layers,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image
        .reinterpret_stacked_2d_as_array(layers)
        .expect("tile array layout");
    image.sampler = ImageSampler::nearest();
    image
}

fn copy_tile(atlas: &RgbaImage, tile: UVec2, out: &mut Vec<u8>) -> bool {
    let x0 = tile.x * TILE_SIZE;
    let y0 = tile.y * TILE_SIZE;
    if x0 + TILE_SIZE > atlas.width() || y0 + TILE_SIZE > atlas.height() {
        error!("tile {tile} lies outside the texture atlas");
        return false;
    }

    let raw = atlas.as_raw();
    let row_bytes = (TILE_SIZE * 4) as usize;
    for row in 0..TILE_SIZE {
        let start = (((y0 + row) * atlas.width() + x0) * 4) as usize;
        out.extend_from_slice(&raw[start..start + row_bytes]);
    }
    true
}

fn push_checker_tile(out: &mut Vec<u8>) {
    for y in 0..TILE_SIZE {
        for x in 0..TILE_SIZE {
            if (x / 4 + y / 4) % 2 == 0 {
                out.extend_from_slice(&[255, 0, 255, 255]);
            } else {
                out.extend_from_slice(&[0, 0, 0, 255]);
            }
        }
    }
}
