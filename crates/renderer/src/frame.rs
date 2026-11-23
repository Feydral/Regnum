use regnum_assets::{Handle, Mesh};

use crate::camera::Camera;

pub struct Frame {
    pub camera: Camera,
    pub meshes: Vec<Handle<Mesh>>,
}
