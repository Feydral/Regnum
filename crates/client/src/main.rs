use bevy::prelude::*;

mod demo_texturing;
mod demo_world;
mod fly_cam;
mod material;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Regnum".into(),
                        ..default()
                    }),
                    ..Default::default()
                })
                .set(AssetPlugin {
                    file_path: "../../assets".into(),
                    ..Default::default()
                }),
        )
        .add_plugins(MaterialPlugin::<material::VoxelMaterial>::default())
        .add_plugins(fly_cam::FlyCamPlugin)
        .add_plugins(demo_world::DemoWorldPlugin)
        .run();
}
