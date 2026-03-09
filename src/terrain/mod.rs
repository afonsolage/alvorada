//! Terrain module: procedural world-map generation and tile rendering.
//!
//! # Sub-modules
//! | Module | Purpose |
//! |--------|---------|
//! | [`map`]    | Core data types ([`Map`], [`TileType`]) and generation logic. |
//! | [`noise`]  | 2-D Simplex Noise implementation used by the terrain generator. |
//! | [`render`] | Bevy systems that spawn tile entities from the [`Map`] resource. |
//!
//! # Usage
//! Add [`TerrainPlugin`] to your [`bevy::app::App`]:
//! ```no_run
//! # use bevy::prelude::*;
//! # use alvorada::terrain::TerrainPlugin;
//! App::new().add_plugins(TerrainPlugin);
//! ```
//!
//! The plugin:
//! 1. Generates and inserts the [`Map`] resource (seed `12345` by default).
//! 2. Runs [`render::spawn_tiles`] during `Startup` to populate the world.

pub mod map;
pub mod noise;
pub mod render;

pub use map::{Map, TILE_SIZE};

use bevy::prelude::*;

/// Registers terrain resources and rendering systems.
///
/// Inserts the procedurally-generated [`Map`] resource and spawns all tile
/// entities during the [`Startup`] schedule phase.
pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Map::generate(12345))
            .add_systems(Startup, render::spawn_tiles);
    }
}
