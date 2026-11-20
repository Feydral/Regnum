use crate::{
    assets::{Assets, Handle},
    mesh::Mesh,
};

#[derive(Default)]
pub struct AssetManager {
    meshes: Assets<Mesh>,
}

impl AssetManager {
    pub fn create_mesh(&mut self, mesh: Mesh) -> Handle<Mesh> {
        self.meshes.add(mesh)
    }

    pub fn mesh_mut(&mut self, handle: Handle<Mesh>) -> Option<&mut Mesh> {
        self.meshes.get_mut(handle)
    }

    pub fn remove_mesh(&mut self, handle: Handle<Mesh>) -> Option<Mesh> {
        self.meshes.remove(handle)
    }

    pub fn meshes_mut(&mut self) -> &mut Assets<Mesh> {
        &mut self.meshes
    }
}
