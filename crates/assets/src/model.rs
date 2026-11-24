use crate::{Handle, Mesh, Texture, Transform};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Model {
    pub mesh: Handle<Mesh>,
    pub texture: Handle<Texture>,
    pub transform: Transform,
}
