mod demo_world;
mod fly_cam;
mod material;
mod mesher;
mod models;
mod textures;
mod visuals;

use std::path::PathBuf;

use bevy::prelude::*;
use game::plugin::GamePlugin;

fn assets_dir() -> PathBuf {
    if let Some(manifest_dir) = std::env::var_os("CARGO_MANIFEST_DIR") {
        return PathBuf::from(manifest_dir).join("../../assets");
    }
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|dir| dir.join("assets")))
        .unwrap_or_else(|| PathBuf::from("assets"))
}

fn main() {
    let assets = assets_dir();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Regnum".into(),
                        ..default()
                    }),
                    ..default()
                })
                .set(AssetPlugin {
                    file_path: assets.to_string_lossy().into_owned(),
                    ..default()
                }),
        )
        .add_plugins(GamePlugin::new(assets.join("blocks")))
        .add_plugins(visuals::BlockVisualsPlugin::new(&assets))
        .add_plugins((fly_cam::FlyCamPlugin, demo_world::DemoWorldPlugin))
        .run();
}
