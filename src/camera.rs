//! Top-down 2D camera that follows the player and supports mouse-wheel zoom.
//!
//! The camera automatically tracks the player's world position each frame by
//! running its follow system in [`PostUpdate`] — after all movement systems
//! have completed — to avoid one-frame lag.
//!
//! # Controls
//! | Input             | Action       |
//! |-------------------|--------------|
//! | Mouse scroll up   | Zoom in      |
//! | Mouse scroll down | Zoom out     |
//!
//! Zoom sensitivity can be adjusted with the on-screen slider (bottom-right).
//!
//! # Usage
//! Add [`CameraPlugin`] to your [`bevy::app::App`].

use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, ui::RelativeCursorPosition};

use crate::player::Player;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default scroll-wheel zoom sensitivity (orthographic scale change per scroll unit).
const DEFAULT_ZOOM_SENSITIVITY: f32 = 0.05;

/// Minimum sensitivity exposed on the slider.
const MIN_SENSITIVITY: f32 = 0.01;

/// Maximum sensitivity exposed on the slider.
const MAX_SENSITIVITY: f32 = 0.3;

/// Minimum orthographic scale value (maximum zoom-in).
const MIN_ZOOM: f32 = 0.2;

/// Maximum orthographic scale value (maximum zoom-out).
const MAX_ZOOM: f32 = 10.0;

/// Initial orthographic scale; starts zoomed out to show a wide map area.
const INITIAL_ZOOM: f32 = 3.0;

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Configurable zoom sensitivity for the camera scroll wheel.
#[derive(Resource)]
pub struct ZoomSettings {
    /// Sensitivity multiplier: scale change per accumulated scroll unit.
    pub sensitivity: f32,
}

impl Default for ZoomSettings {
    fn default() -> Self {
        Self {
            sensitivity: DEFAULT_ZOOM_SENSITIVITY,
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component for the main game camera.
///
/// Query with `With<GameCamera>` to target the camera entity specifically.
#[derive(Component)]
pub struct GameCamera;

/// Marker component for the zoom sensitivity slider track.
#[derive(Component)]
struct ZoomSliderTrack;

/// Marker component for the zoom sensitivity slider fill bar.
#[derive(Component)]
struct ZoomSliderFill;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the game camera entity and its follow/zoom systems.
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ZoomSettings>()
            .add_systems(Startup, (spawn_camera, spawn_zoom_ui))
            // Follow runs in PostUpdate so the camera always reads the final
            // player position for the current frame (no one-frame lag).
            .add_systems(PostUpdate, camera_follow_player)
            .add_systems(Update, (camera_zoom, update_zoom_slider));
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
    zoom_settings: Res<ZoomSettings>,
) {
    let delta = accumulated_scroll.delta.y;
    if delta == 0.0 {
        return;
    }

    if let Projection::Orthographic(ref mut ortho) = **camera {
        ortho.scale =
            (ortho.scale - delta * zoom_settings.sensitivity).clamp(MIN_ZOOM, MAX_ZOOM);
    }
}

/// Spawns the zoom sensitivity slider panel in the bottom-right corner.
fn spawn_zoom_ui(mut commands: Commands) {
    let initial_pct = sensitivity_to_percent(DEFAULT_ZOOM_SENSITIVITY);

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                right: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(6.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|panel| {
            // Label
            panel.spawn((
                Text::new("Zoom Sensitivity"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Slider row: "−"  [=========---------]  "+"
            panel
                .spawn(Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(6.0),
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new("−"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));

                    // Slider track (clickable)
                    row.spawn((
                        ZoomSliderTrack,
                        Node {
                            width: Val::Px(140.0),
                            height: Val::Px(10.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
                        Interaction::default(),
                        RelativeCursorPosition::default(),
                    ))
                    .with_children(|track| {
                        // Colored fill bar showing the current sensitivity level
                        track.spawn((
                            ZoomSliderFill,
                            Node {
                                width: Val::Percent(initial_pct),
                                height: Val::Percent(100.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgb(0.2, 0.6, 1.0)),
                        ));
                    });

                    row.spawn((
                        Text::new("+"),
                        TextFont {
                            font_size: 14.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

/// Updates [`ZoomSettings::sensitivity`] and the slider fill when the user
/// clicks on the slider track.
fn update_zoom_slider(
    mut zoom_settings: ResMut<ZoomSettings>,
    slider_track: Single<(&Interaction, &RelativeCursorPosition), With<ZoomSliderTrack>>,
    mut fill: Single<&mut Node, With<ZoomSliderFill>>,
) {
    let (interaction, rel_cursor) = *slider_track;
    if *interaction != Interaction::Pressed {
        return;
    }

    if let Some(pos) = rel_cursor.normalized {
        let t = pos.x.clamp(0.0, 1.0);
        zoom_settings.sensitivity = MIN_SENSITIVITY + t * (MAX_SENSITIVITY - MIN_SENSITIVITY);
        fill.width = Val::Percent(t * 100.0);
    }
}

/// Maps a sensitivity value to a 0–100 percentage for the slider fill width.
fn sensitivity_to_percent(sensitivity: f32) -> f32 {
    ((sensitivity - MIN_SENSITIVITY) / (MAX_SENSITIVITY - MIN_SENSITIVITY) * 100.0)
        .clamp(0.0, 100.0)
}
