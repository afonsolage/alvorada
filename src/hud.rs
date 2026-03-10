//! Player HUD — health bar displayed in the bottom-left corner of the screen.
//!
//! The HUD consists of a fixed-position UI panel containing an "HP" label and
//! a red fill bar whose width is updated every frame to reflect the player's
//! current hit-points.
//!
//! # Layout
//! ```text
//! ┌─────────────────────┐
//! │ HP                  │
//! │ ████████░░░░░░░░░░░ │  ← fill (red) inside background (dark grey)
//! └─────────────────────┘
//! ```
//! The panel is anchored to the bottom-left corner via `PositionType::Absolute`.
//!
//! # Usage
//! Add [`HudPlugin`] to your [`bevy::app::App`].

use bevy::prelude::*;

use crate::combat::Health;
use crate::player::Player;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Width of the player health bar in logical pixels.
const HUD_BAR_WIDTH: f32 = 160.0;

/// Height of the player health bar in logical pixels.
const HUD_BAR_HEIGHT: f32 = 16.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marker component for the player health bar fill node.
///
/// Queried by [`update_player_health_bar`] to resize the fill each frame.
#[derive(Component)]
struct PlayerHealthBarFill;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the player HUD startup and update systems.
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player_hud)
            .add_systems(Update, update_player_health_bar);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the bottom-left health bar panel.
///
/// The node tree is:
/// ```text
/// Node (Absolute, bottom: 16px, left: 16px, Column)
///   └─ Text "HP"
///   └─ Node (bar background, 160×16 px, dark grey)
///        └─ Node (bar fill, 100 %, 100 %, red) [PlayerHealthBarFill]
/// ```
fn spawn_player_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(16.0),
                left: Val::Px(16.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Start,
                padding: UiRect::all(Val::Px(8.0)),
                row_gap: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|panel| {
            // "HP" label
            panel.spawn((
                Text::new("HP"),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Bar background (dark grey container)
            panel
                .spawn((
                    Node {
                        width: Val::Px(HUD_BAR_WIDTH),
                        height: Val::Px(HUD_BAR_HEIGHT),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
                ))
                .with_children(|bg| {
                    // Bar fill (red; starts at 100 %)
                    bg.spawn((
                        PlayerHealthBarFill,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.85, 0.15, 0.15)),
                    ));
                });
        });
}

/// Syncs the health bar fill width to the player's current HP percentage.
fn update_player_health_bar(
    player_health: Single<&Health, With<Player>>,
    mut fill: Single<&mut Node, With<PlayerHealthBarFill>>,
) {
    let pct = (player_health.current as f32 / player_health.max as f32).clamp(0.0, 1.0);
    fill.width = Val::Percent(pct * 100.0);
}
