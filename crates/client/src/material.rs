use bevy::prelude::*;
use bevy::reflect::TypePath;
use bevy::render::render_resource::AsBindGroup;
use bevy::shader::ShaderRef;

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct VoxelMaterial {
    #[texture(0, dimension = "2d_array")]
    #[sampler(1)]
    pub tiles: Handle<Image>,
    #[uniform(2)]
    pub tint: LinearRgba,
    pub alpha_mode: AlphaMode,
}

impl Material for VoxelMaterial {
    fn fragment_shader() -> ShaderRef {
        "shader/texture_array.wesl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }
}
