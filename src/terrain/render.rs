//! Systems for rendering the tile map as coloured 2-D quads.
//!
//! All tiles of the same [`TileType`] share a single [`ColorMaterial`] handle,
//! which lets Bevy's automatic mesh batching merge them into very few draw
//! calls (one per unique material, i.e. one per tile type).

use bevy::prelude::*;

use super::map::{Map, TileType, MAP_SIZE, TILE_SIZE};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks an entity as a rendered map tile and stores its grid position.
///
/// Attach this component alongside `Mesh2d`, `MeshMaterial2d`, and
/// `Transform` when spawning tile entities (see [`spawn_tiles`]).
/// The `x` and `y` fields are reserved for future systems (e.g. pathfinding
/// or tile highlighting) that need to map a tile entity back to grid
/// coordinates; they are not read by any current system.
#[derive(Component)]
#[allow(dead_code)] // fields reserved for future use (pathfinding, highlighting)
pub struct Tile {
    /// Tile column index (x-axis, increases to the right).
    pub x: u32,
    /// Tile row index (y-axis, increases upward).
    pub y: u32,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns one coloured rectangular mesh per tile in the [`Map`] resource.
///
/// This system is registered as a `Startup` system by [`TerrainPlugin`].
///
/// # Performance
/// * A single `Rectangle` mesh is shared by all 65 536 tile entities.
/// * One [`ColorMaterial`] is created per [`TileType`] (7 total).
/// * Bevy batches draw calls by mesh + material, so the full terrain renders
///   in ≤ 7 GPU draw calls.
pub fn spawn_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    map: Res<Map>,
) {
    // All tiles share a single mesh asset.
    let tile_mesh = meshes.add(Rectangle::new(TILE_SIZE, TILE_SIZE));

    // One material per tile type — same Handle cloned across entities.
    let tile_materials: Vec<Handle<ColorMaterial>> = TileType::ALL
        .iter()
        .map(|&tile_type| materials.add(ColorMaterial::from_color(tile_type.color())))
        .collect();

    for y in 0..MAP_SIZE {
        for x in 0..MAP_SIZE {
            let tile_type = map.get(x, y);
            let material = tile_materials[tile_type as usize].clone();
            let world_pos = Map::tile_to_world(x, y);

            commands.spawn((
                Tile { x, y },
                Mesh2d(tile_mesh.clone()),
                MeshMaterial2d(material),
                Transform::from_xyz(world_pos.x, world_pos.y, 0.0),
            ));
        }
    }
}
