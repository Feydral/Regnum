use bevy::prelude::*;

mod demo_texturing;
mod demo_world;
mod fly_cam;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Regnum".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(fly_cam::FlyCamPlugin)
        .add_plugins(demo_world::DemoWorldPlugin)
        .run();
}
