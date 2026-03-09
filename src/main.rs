// Copyright 2024 Afonso Lage
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};

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
        .add_systems(Update, (camera_movement, camera_zoom))
        .run();
}

/// Stores the top-down 2D camera's movement speed and zoom speed.
#[derive(Component)]
struct TopDownCamera {
    speed: f32,
    zoom_speed: f32,
}

const MIN_ZOOM_SCALE: f32 = 0.1;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // 2D camera with orthographic projection
    commands.spawn((
        Camera2d,
        TopDownCamera {
            speed: 300.0,
            zoom_speed: 0.1,
        },
    ));

    // Square at the center of the scene
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(100.0, 100.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.3, 0.5, 0.8)))),
        Transform::from_xyz(0.0, 0.0, 1.0),
    ));

    // Background plane
    commands.spawn((
        Mesh2d(meshes.add(Rectangle::new(500.0, 500.0))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.5, 0.5, 0.5)))),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

/// Moves the camera with WASD keys along the 2D plane.
fn camera_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mut camera: Single<(&TopDownCamera, &mut Transform)>,
) {
    let (cam, ref mut transform) = *camera;

    let mut direction = Vec2::ZERO;

    if keys.pressed(KeyCode::KeyW) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction.x += 1.0;
    }

    let movement = direction.normalize_or_zero() * cam.speed * time.delta_secs();
    transform.translation.x += movement.x;
    transform.translation.y += movement.y;
}

/// Zooms the camera in and out with the mouse scroll wheel.
fn camera_zoom(
    accumulated_mouse_scroll: Res<AccumulatedMouseScroll>,
    mut camera: Single<(&TopDownCamera, &mut Projection)>,
) {
    let delta = accumulated_mouse_scroll.delta;
    if delta == Vec2::ZERO {
        return;
    }

    let (cam, ref mut projection) = *camera;

    if let Projection::Orthographic(ref mut ortho) = **projection {
        ortho.scale = (ortho.scale - delta.y * cam.zoom_speed).max(MIN_ZOOM_SCALE);
    }
}
