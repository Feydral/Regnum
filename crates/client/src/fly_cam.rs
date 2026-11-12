use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions};

pub struct FlyCamPlugin;

impl Plugin for FlyCamPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, (toggle_cursor_grab, look, movement).chain());
    }
}

#[derive(Component)]
pub struct FlyCam {
    pub speed: f32,
    pub sensitivity: f32,
    yaw: f32,
    pitch: f32,
}

impl Default for FlyCam {
    fn default() -> Self {
        Self {
            speed: 10.0,
            sensitivity: 0.0016,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 3.0, 12.0),
        FlyCam::default(),
    ));
}

fn toggle_cursor_grab(
    mut cursor: Single<&mut CursorOptions, With<Window>>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if mouse.just_pressed(MouseButton::Left) {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
    if keys.just_pressed(KeyCode::Escape) {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }
}

fn look(
    cursor: Single<&CursorOptions, With<Window>>,
    motion: Res<AccumulatedMouseMotion>,
    mut cam: Single<(&mut Transform, &mut FlyCam)>,
) {
    if cursor.grab_mode != CursorGrabMode::Locked {
        return;
    }

    let (transform, fly) = &mut *cam;
    fly.yaw -= motion.delta.x * fly.sensitivity;
    fly.pitch = (fly.pitch - motion.delta.y * fly.sensitivity).clamp(-1.54, 1.54);
    transform.rotation = Quat::from_euler(EulerRot::YXZ, fly.yaw, fly.pitch, 0.0);
}

fn movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut cam: Single<(&mut Transform, &FlyCam)>,
) {
    let (transform, fly) = &mut *cam;
    let axis = |pos, neg| keys.pressed(pos) as i32 as f32 - keys.pressed(neg) as i32 as f32;

    let dir = *transform.forward() * axis(KeyCode::KeyW, KeyCode::KeyS)
        + *transform.right() * axis(KeyCode::KeyD, KeyCode::KeyA)
        + Vec3::Y * axis(KeyCode::Space, KeyCode::ShiftLeft);

    transform.translation += dir.normalize_or_zero() * fly.speed * time.delta_secs();
}
