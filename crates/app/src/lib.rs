mod app;
mod input;
mod timestep;

pub use app::run;
pub use input::{Input, Key};

use regnum_assets::AssetManager;
use regnum_renderer::Frame;

pub trait Application {
    fn load(&mut self, _assets: &mut AssetManager) {}

    fn unload(&mut self, _assets: &mut AssetManager) {}

    fn resize(&mut self, _width: u32, _height: u32) {}

    fn fixed_timestep(&self) -> f32 {
        1.0 / 60.0
    }

    fn fixed_update(&mut self, _dt: f32, _assets: &mut AssetManager) {}

    fn pre_update(&mut self, _dt: f32, _input: &Input, _assets: &mut AssetManager) {}

    fn update(&mut self, _dt: f32, _input: &Input, _assets: &mut AssetManager) {}

    fn post_update(&mut self, _dt: f32, _input: &Input, _assets: &mut AssetManager) {}

    fn render(&self) -> Frame;
}
