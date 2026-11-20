use crate::{assets::Handle, camera::Camera, mesh::Mesh};

pub struct Frame {
    pub camera: Camera,
    pub meshes: Vec<Handle<Mesh>>,
}
