use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

pub const TILE: u32 = 16;
pub const TILES: u32 = 5;

pub fn tile_index(block: u32, face: u32) -> u32 {
    match (block, face) {
        (1, 1) => 0,
        (1, 0) => 2,
        (1, _) => 1,
        (2, _) => 2,
        (3, _) => 3,
        _ => 4,
    }
}

pub fn tile_pixel(tile: u32, x: u32, y: u32) -> [u8; 4] {
    let base: [f32; 3] = match tile {
        0 => [0.30, 0.65, 0.25],
        1 => {
            if y < 4 {
                [0.30, 0.65, 0.25]
            } else {
                [0.45, 0.30, 0.18]
            }
        }
        2 => [0.45, 0.30, 0.18],
        3 => [0.50, 0.50, 0.52],
        _ => {
            if (x / 4 + y / 4) % 2 == 0 {
                [1.0, 0.0, 1.0]
            } else {
                [0.0, 0.0, 0.0]
            }
        }
    };
    [
        (base[0] * 255.0) as u8,
        (base[1] * 255.0) as u8,
        (base[2] * 255.0) as u8,
        255,
    ]
}

pub fn build_atlas() -> Image {
    let mut data = Vec::with_capacity((TILE * TILES * TILE * 4) as usize);
    for y in 0..TILE {
        for x in 0..TILE * TILES {
            data.extend_from_slice(&tile_pixel(x / TILE, x % TILE, y));
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: TILE * TILES,
            height: TILE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}
