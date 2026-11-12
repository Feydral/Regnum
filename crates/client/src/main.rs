use bevy::prelude::*;

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
        .add_systems(Startup, setup_scene)
        .run();
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let mat = materials.add(Color::srgb(0.6, 0.7, 0.4));

    for x in -3..=3 {
        for z in -3..=3 {
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(mat.clone()),
                Transform::from_xyz(x as f32 * 3.0, 0.0, z as f32 * 3.0),
            ));
        }
    }

    commands.spawn((
        DirectionalLight::default(),
        Transform::from_xyz(4.0, 10.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
