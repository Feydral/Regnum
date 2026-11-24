mod app;
mod camera_controller;
mod game;
mod input;
mod terrain;

fn main() -> anyhow::Result<()> {
    app::run()
}
