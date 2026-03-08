// Copyright 2024 Afonso Lage
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::f32::consts::FRAC_PI_2;

use bevy::{
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions},
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alvorada".into(),
                // Bind the canvas element used in the web build
                canvas: Some("#bevy".to_owned()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_movement, camera_look, toggle_cursor_grab))
        .run();
}

/// Stores the FPS camera's current look angles and movement speed.
#[derive(Component)]
struct FpsCamera {
    speed: f32,
    sensitivity: Vec2,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    // Lock the cursor for FPS-style mouse look
    cursor_options.grab_mode = CursorGrabMode::Locked;
    cursor_options.visible = false;

    // Camera positioned back and slightly elevated, aimed at the cube in the center.
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 2.0, 5.0).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
        FpsCamera {
            speed: 5.0,
            sensitivity: Vec2::new(0.003, 0.002),
        },
    ));

    // Cube at the center of the scene, resting on the plane
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.3, 0.5, 0.8),
            ..default()
        })),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));

    // Ground plane
    commands.spawn((
        Mesh3d(meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(5.0)))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgb(0.5, 0.5, 0.5),
            ..default()
        })),
    ));

    // Directional light to illuminate the scene
    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Moves the camera with WASD keys (horizontal plane only, typical FPS style).
fn camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Single<(&FpsCamera, &mut Transform)>,
) {
    let (fps_camera, ref mut transform) = *camera;

    // Project forward/right onto the XZ plane so WASD never changes altitude
    let fwd = *transform.forward();
    let forward = Vec3::new(fwd.x, 0.0, fwd.z).normalize_or_zero();
    let rgt = *transform.right();
    let right = Vec3::new(rgt.x, 0.0, rgt.z).normalize_or_zero();

    let mut direction = Vec3::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right;
    }

    transform.translation += direction.normalize_or_zero() * fps_camera.speed * time.delta_secs();
}

/// Rotates the camera based on accumulated mouse movement (FPS mouse look).
fn camera_look(
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    mut camera: Single<(&FpsCamera, &mut Transform)>,
) {
    let delta = accumulated_mouse_motion.delta;
    if delta == Vec2::ZERO {
        return;
    }

    let (fps_camera, ref mut transform) = *camera;

    let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
    let yaw = yaw - delta.x * fps_camera.sensitivity.x;
    let pitch = (pitch - delta.y * fps_camera.sensitivity.y)
        .clamp(-FRAC_PI_2 + 0.01, FRAC_PI_2 - 0.01);

    transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
}

/// Toggles cursor lock/visibility with the Escape key.
fn toggle_cursor_grab(
    keys: Res<ButtonInput<KeyCode>>,
    mut cursor_options: Single<&mut CursorOptions>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        let locked = cursor_options.grab_mode == CursorGrabMode::Locked;
        cursor_options.grab_mode = if locked {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Locked
        };
        cursor_options.visible = locked;
    }
}
