//! Top-down 2D camera that follows the player and supports mouse-wheel zoom.
//!
//! The camera automatically tracks the player's world position each frame by
//! running its follow system in [`PostUpdate`] — after all movement systems
//! have completed — to avoid one-frame lag.
//!
//! # Controls
//! | Input           | Action      |
//! |-----------------|-------------|
//! | Mouse scroll up | Zoom in     |
//! | Mouse scroll down | Zoom out  |
//!
//! # Usage
//! Add [`CameraPlugin`] to your [`bevy::app::App`].

use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*};

use crate::player::Player;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Scroll-wheel zoom sensitivity (orthographic scale change per scroll unit).
const ZOOM_SPEED: f32 = 0.1;

/// Minimum orthographic scale value (maximum zoom-in).
const MIN_ZOOM: f32 = 0.2;

/// Maximum orthographic scale value (maximum zoom-out).
const MAX_ZOOM: f32 = 10.0;

/// Initial orthographic scale; starts zoomed out to show a wide map area.
const INITIAL_ZOOM: f32 = 3.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component for the main game camera.
///
/// Query with `With<GameCamera>` to target the camera entity specifically.
#[derive(Component)]
pub struct GameCamera;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the game camera entity and its follow/zoom systems.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            // Follow runs in PostUpdate so the camera always reads the final
            // player position for the current frame (no one-frame lag).
            .add_systems(PostUpdate, camera_follow_player)
            .add_systems(Update, camera_zoom);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the orthographic 2D camera with [`INITIAL_ZOOM`] applied.
fn spawn_camera(mut commands: Commands) {
    let mut projection = OrthographicProjection::default_2d();
    projection.scale = INITIAL_ZOOM;

    commands.spawn((GameCamera, Camera2d, Projection::Orthographic(projection)));
}

/// Moves the camera to match the player's XY position every frame.
///
/// Runs in [`PostUpdate`] to ensure it reads the player position after all
/// `Update` movement systems have completed.
fn camera_follow_player(
    player: Single<&Transform, With<Player>>,
    mut camera: Single<&mut Transform, (With<GameCamera>, Without<Player>)>,
) {
    let target = player.translation;
    camera.translation.x = target.x;
    camera.translation.y = target.y;
}

/// Adjusts the camera zoom level with the mouse scroll wheel.
///
/// Scroll **up** zooms in (smaller scale); scroll **down** zooms out
/// (larger scale). The scale is clamped to `[MIN_ZOOM, MAX_ZOOM]`.
fn camera_zoom(
    accumulated_scroll: Res<AccumulatedMouseScroll>,
    mut camera: Single<&mut Projection, With<GameCamera>>,
) {
    let delta = accumulated_scroll.delta.y;
    if delta == 0.0 {
        return;
    }

    if let Projection::Orthographic(ref mut ortho) = **camera {
        ortho.scale = (ortho.scale - delta * ZOOM_SPEED).clamp(MIN_ZOOM, MAX_ZOOM);
    }
}
