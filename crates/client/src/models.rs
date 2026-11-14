use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use bevy::prelude::*;
use game::world::block::Block;
use game::world::registry::property::{Property, ValueExpr, decode, matches_when, validate_when};
use game::world::registry::{AIR_NAME, BlockRegistry};
use serde::Deserialize;
use tracing::{error, warn};

use crate::textures::{TILE_SIZE, TileAllocator};

pub const UNITS: f32 = 16.0;

pub const SIDE_DIRS: [IVec3; 6] = [
    IVec3::NEG_Y,
    IVec3::Y,
    IVec3::NEG_X,
    IVec3::X,
    IVec3::NEG_Z,
    IVec3::Z,
];

const SIDE_AXES: [(usize, f32); 6] = [(1, 0.0), (1, 1.0), (0, 0.0), (0, 1.0), (2, 0.0), (2, 1.0)];

const CUBE_CORNERS: [[[f32; 3]; 4]; 6] = [
    [
        [0.0, 0.0, 0.0],
        [16.0, 0.0, 0.0],
        [16.0, 0.0, 16.0],
        [0.0, 0.0, 16.0],
    ],
    [
        [0.0, 16.0, 0.0],
        [0.0, 16.0, 16.0],
        [16.0, 16.0, 16.0],
        [16.0, 16.0, 0.0],
    ],
    [
        [0.0, 0.0, 0.0],
        [0.0, 0.0, 16.0],
        [0.0, 16.0, 16.0],
        [0.0, 16.0, 0.0],
    ],
    [
        [16.0, 0.0, 16.0],
        [16.0, 0.0, 0.0],
        [16.0, 16.0, 0.0],
        [16.0, 16.0, 16.0],
    ],
    [
        [16.0, 0.0, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 16.0, 0.0],
        [16.0, 16.0, 0.0],
    ],
    [
        [0.0, 0.0, 16.0],
        [16.0, 0.0, 16.0],
        [16.0, 16.0, 16.0],
        [0.0, 16.0, 16.0],
    ],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeshType {
    #[default]
    None,
    Opaque,
    Transparent,
    Unculled,
}

#[derive(Debug, Clone)]
pub struct Face {
    pub positions: [Vec3; 4],
    pub normal: Vec3,
    pub uvs: [Vec2; 4],
    pub side: Option<usize>,
    pub layer: u32,
}

#[derive(Debug, Clone)]
pub struct BlockModel {
    pub mesh_type: MeshType,
    pub faces: Vec<Face>,
    pub full_cube: bool,
}

static EMPTY_MODEL: BlockModel = BlockModel {
    mesh_type: MeshType::None,
    faces: Vec::new(),
    full_cube: false,
};

#[derive(Resource)]
pub struct BlockModels {
    models: Vec<Vec<BlockModel>>,
}

impl BlockModels {
    pub fn model(&self, block: Block) -> &BlockModel {
        self.models
            .get(block.id as usize)
            .and_then(|states| states.get(block.data as usize))
            .unwrap_or(&EMPTY_MODEL)
    }

    pub fn load(dir: &Path, registry: &BlockRegistry, tiles: &mut TileAllocator) -> Self {
        let mut models: Vec<Vec<BlockModel>> = vec![Vec::new(); registry.len()];

        let mut paths: Vec<PathBuf> = match fs::read_dir(dir) {
            Ok(entries) => entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
                .collect(),
            Err(err) => {
                error!("{}: {err}", dir.display());
                Vec::new()
            }
        };
        paths.sort();

        for path in paths {
            let text = match fs::read_to_string(&path) {
                Ok(text) => text,
                Err(err) => {
                    error!("{}: {err}", path.display());
                    continue;
                }
            };
            let file: ModelFile = match serde_json::from_str(text.trim_start_matches('\u{feff}')) {
                Ok(file) => file,
                Err(err) => {
                    error!("{}: {err}", path.display());
                    continue;
                }
            };
            if file.name == AIR_NAME {
                continue;
            }
            let Some(id) = registry.id(&file.name) else {
                warn!(
                    "{}: block '{}' is not in the registry",
                    path.display(),
                    file.name
                );
                continue;
            };

            match build_block_models(&file, registry, id, tiles) {
                Ok(states) => models[id as usize] = states,
                Err(err) => error!("{}: {err}", path.display()),
            }
        }

        for (id, definition) in registry.iter() {
            if models[id as usize].is_empty() && definition.name != AIR_NAME {
                warn!("block '{}' has no model", definition.name);
            }
        }

        Self { models }
    }
}

#[derive(Deserialize)]
struct ModelFile {
    name: String,
    #[serde(default)]
    states: Vec<ModelStateFile>,
}

#[derive(Deserialize)]
struct ModelStateFile {
    #[serde(default)]
    when: HashMap<String, u32>,
    #[serde(default)]
    model: ModelJson,
    #[serde(default)]
    mesh_type: MeshType,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ModelJson {
    #[default]
    Empty,
    CubeUniform {
        uv: TextureRect,
    },
    CubeTopBottomSide {
        top: TextureRect,
        bottom: TextureRect,
        side: TextureRect,
    },
    Custom(Vec<FaceJson>),
}

#[derive(Deserialize)]
struct TextureRect {
    min: [u32; 2],
}

impl TextureRect {
    fn tile(&self) -> UVec2 {
        UVec2::new(self.min[0] / TILE_SIZE, self.min[1] / TILE_SIZE)
    }
}

#[derive(Deserialize)]
struct FaceJson {
    positions: [[ValueExpr; 3]; 4],
    uv: TextureRect,
}

fn build_block_models(
    file: &ModelFile,
    registry: &BlockRegistry,
    id: u16,
    tiles: &mut TileAllocator,
) -> Result<Vec<BlockModel>, String> {
    let definition = registry
        .definition(Block::new(id, 0))
        .ok_or_else(|| format!("block '{}' has no definition", file.name))?;
    let properties = &definition.properties;

    for state in &file.states {
        validate_when(&state.when, properties)?;
    }

    (0..definition.states.len() as u32)
        .map(|data| {
            let values = decode(properties, data);
            let matched = file
                .states
                .iter()
                .find(|state| matches_when(&state.when, properties, &values));

            match matched {
                Some(state) => build_model(state, &file.name, properties, &values, tiles),
                None => Ok(BlockModel {
                    mesh_type: MeshType::None,
                    faces: Vec::new(),
                    full_cube: false,
                }),
            }
        })
        .collect()
}

fn build_model(
    state: &ModelStateFile,
    name: &str,
    properties: &[Property],
    values: &[u32],
    tiles: &mut TileAllocator,
) -> Result<BlockModel, String> {
    let faces = match &state.model {
        ModelJson::Empty => Vec::new(),
        ModelJson::CubeUniform { uv } => cube_faces([uv.tile(); 6], tiles),
        ModelJson::CubeTopBottomSide { top, bottom, side } => cube_faces(
            [
                bottom.tile(),
                top.tile(),
                side.tile(),
                side.tile(),
                side.tile(),
                side.tile(),
            ],
            tiles,
        ),
        ModelJson::Custom(faces) => {
            let mut out = Vec::with_capacity(faces.len());
            for face in faces {
                let mut positions = [[0.0f32; 3]; 4];
                for (corner, expressions) in face.positions.iter().enumerate() {
                    for (axis, expression) in expressions.iter().enumerate() {
                        let value = expression
                            .resolve(properties, values)
                            .map_err(|err| format!("block '{name}': {err}"))?;
                        if value as f32 > UNITS {
                            return Err(format!(
                                "block '{name}': position {value} is outside 0..={UNITS}"
                            ));
                        }
                        positions[corner][axis] = value as f32;
                    }
                }
                let layer = tiles.layer(face.uv.tile());
                out.extend(make_face(positions, layer));
            }
            out
        }
    };

    let mesh_type = if faces.is_empty() {
        MeshType::None
    } else {
        state.mesh_type
    };
    let full_cube = mesh_type != MeshType::None && is_full_cube(&faces);

    Ok(BlockModel {
        mesh_type,
        faces,
        full_cube,
    })
}

fn cube_faces(tile_per_side: [UVec2; 6], tiles: &mut TileAllocator) -> Vec<Face> {
    (0..6)
        .filter_map(|side| make_face(CUBE_CORNERS[side], tiles.layer(tile_per_side[side])))
        .collect()
}

fn make_face(corners: [[f32; 3]; 4], layer: u32) -> Option<Face> {
    let positions = corners.map(|corner| Vec3::from(corner) / UNITS);
    let normal = (positions[1] - positions[0])
        .cross(positions[2] - positions[0])
        .normalize_or_zero();
    if normal == Vec3::ZERO {
        return None;
    }

    let side = face_side(&positions, normal);
    let projection = side.unwrap_or_else(|| dominant_side(normal));
    let uvs = positions.map(|position| side_uv(projection, position));

    Some(Face {
        positions,
        normal,
        uvs,
        side,
        layer,
    })
}

fn face_side(positions: &[Vec3; 4], normal: Vec3) -> Option<usize> {
    let side = SIDE_AXES
        .iter()
        .position(|&(axis, value)| positions.iter().all(|p| p[axis] == value))?;
    (normal.dot(SIDE_DIRS[side].as_vec3()) > 0.5).then_some(side)
}

fn dominant_side(normal: Vec3) -> usize {
    let abs = normal.abs();
    if abs.y >= abs.x && abs.y >= abs.z {
        if normal.y < 0.0 { 0 } else { 1 }
    } else if abs.x >= abs.z {
        if normal.x < 0.0 { 2 } else { 3 }
    } else if normal.z < 0.0 {
        4
    } else {
        5
    }
}

fn side_uv(side: usize, p: Vec3) -> Vec2 {
    match side {
        0 => Vec2::new(p.x, 1.0 - p.z),
        1 => Vec2::new(p.x, p.z),
        2 => Vec2::new(p.z, 1.0 - p.y),
        3 => Vec2::new(1.0 - p.z, 1.0 - p.y),
        4 => Vec2::new(1.0 - p.x, 1.0 - p.y),
        _ => Vec2::new(p.x, 1.0 - p.y),
    }
}

fn is_full_cube(faces: &[Face]) -> bool {
    (0..6).all(|side| {
        faces
            .iter()
            .any(|face| face.side == Some(side) && covers_side(face, side))
    })
}

fn covers_side(face: &Face, side: usize) -> bool {
    let fixed = SIDE_AXES[side].0;
    (0..3).filter(|&axis| axis != fixed).all(|axis| {
        let min = face
            .positions
            .iter()
            .map(|p| p[axis])
            .fold(f32::MAX, f32::min);
        let max = face
            .positions
            .iter()
            .map(|p| p[axis])
            .fold(f32::MIN, f32::max);
        min == 0.0 && max == 1.0
    })
}
