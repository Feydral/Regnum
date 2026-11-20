use regnum_assets::Handle;

use crate::{camera::Camera, mesh::Mesh};

pub struct Frame {
    pub camera: Camera,
    pub meshes: Vec<Handle<Mesh>>,
}
