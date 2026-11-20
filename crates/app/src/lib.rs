mod app;
mod input;

pub use app::run;
pub use input::{Input, Key};

use regnum_assets::AssetManager;
use regnum_renderer::Frame;

pub trait Application {
    fn update(&mut self, dt: f32, input: &Input, assets: &mut AssetManager);

    fn render(&self) -> Frame;
}
