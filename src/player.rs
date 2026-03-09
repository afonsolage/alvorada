//! Player entity and movement systems.
//!
//! The player is represented as a red circle that moves freely over the tile
//! grid. Tile type affects movement speed, and water tiles are impassable.
//!
//! # Controls
//! | Key | Action |
//! |-----|--------|
//! | W / ↑ | Move up    |
//! | S / ↓ | Move down  |
//! | A / ← | Move left  |
//! | D / → | Move right |
//!
//! # Usage
//! Add [`PlayerPlugin`] to your [`bevy::app::App`].

use bevy::prelude::*;

use crate::terrain::{Map, TILE_SIZE};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Radius of the player circle in world units.
///
/// Set to 60 % of `TILE_SIZE` (≈ 9.6 px) so the circle fits visibly inside one
/// tile while remaining large enough to see clearly at the default zoom level.
const PLAYER_RADIUS: f32 = TILE_SIZE * 0.6;

/// Base movement speed in world units per second (on [`TileType::Grass`]).
const PLAYER_BASE_SPEED: f32 = 150.0;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks the single player entity.
///
/// The actual world position is stored in the entity's [`Transform`] component.
#[derive(Component)]
pub struct Player;

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the player entity and movement systems.
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, player_movement);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns the player circle at the nearest passable tile to the map centre.
///
/// Queries the [`Map`] resource to find a safe spawn location so the player
/// never starts inside an impassable (water) tile.
fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    map: Res<Map>,
) {
    let spawn_pos = map.find_spawn_position();

    commands.spawn((
        Player,
        Mesh2d(meshes.add(Circle::new(PLAYER_RADIUS))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(0.9, 0.2, 0.2)))),
        // z = 1.0 renders the player on top of the tile layer (z = 0.0).
        Transform::from_xyz(spawn_pos.x, spawn_pos.y, 1.0),
    ));
}

/// Moves the player with **WASD** (or arrow keys), respecting tile properties.
///
/// Movement is blocked when the destination tile is impassable (water).
/// On passable tiles the effective speed is `PLAYER_BASE_SPEED / movement_cost`,
/// so sand and forest tiles feel slower than open grassland.
pub fn player_movement(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    map: Res<Map>,
    mut player: Single<(&Player, &mut Transform)>,
) {
    let (_, ref mut transform) = *player;

    // --- Gather directional input -------------------------------------------
    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    let direction = direction.normalize_or_zero();
    if direction == Vec2::ZERO {
        return;
    }

    // --- Apply movement-cost modifier based on current tile ------------------
    let current_pos = transform.translation.truncate();
    let speed_factor = Map::world_to_tile(current_pos)
        .map(|(tx, ty)| {
            let tile = map.get(tx, ty);
            if tile.is_passable() {
                1.0 / tile.movement_cost()
            } else {
                // Player somehow ended up in water — allow escape at base speed.
                1.0
            }
        })
        .unwrap_or(1.0); // Outside map bounds: allow free movement.

    let delta = direction * PLAYER_BASE_SPEED * speed_factor * time.delta_secs();
    let new_pos = transform.translation + delta.extend(0.0);

    // --- Block movement into impassable tiles --------------------------------
    let can_move = Map::world_to_tile(new_pos.truncate())
        .map(|(tx, ty)| map.get(tx, ty).is_passable())
        .unwrap_or(false); // Outside map boundary is also impassable.

    if can_move {
        transform.translation = new_pos;
    }
}
