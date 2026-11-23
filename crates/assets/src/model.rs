use crate::{Handle, Mesh, Texture};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Model {
    pub mesh: Handle<Mesh>,
    pub texture: Handle<Texture>,
}
