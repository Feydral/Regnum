mod camera_controller;
mod game;

use game::Game;

fn main() -> anyhow::Result<()> {
    regnum_app::run(Game::new)
}
