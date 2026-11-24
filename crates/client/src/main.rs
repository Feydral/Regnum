mod app;
mod camera_controller;
mod game;
mod input;

fn main() -> anyhow::Result<()> {
    app::run()
}
