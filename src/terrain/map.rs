//! Map data structures and procedural generation.
//!
//! The world map is a fixed-size grid of [`TileType`]s stored in the [`Map`]
//! resource. Each tile type carries visual (colour) and gameplay (movement
//! cost, passability) properties. The map is generated once at startup via
//! [`Map::generate`] using [`SimplexNoise`] fractional Brownian motion.
//!
//! # Coordinate system
//! * Tile `(0, 0)` is at the **bottom-left** of the world.
//! * Tile `(MAP_SIZE − 1, MAP_SIZE − 1)` is at the **top-right**.
//! * World origin `(0, 0)` corresponds to the **centre** of the map.

use bevy::prelude::*;

use super::noise::SimplexNoise;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Side length of the square world map in tiles.
pub const MAP_SIZE: u32 = 256;

/// Side length of each rendered tile in world units (pixels).
pub const TILE_SIZE: f32 = 16.0;

// ---------------------------------------------------------------------------
// Tile type
// ---------------------------------------------------------------------------

/// Terrain tile variants, ordered roughly from lowest to highest elevation.
///
/// The discriminant value (e.g. `TileType::Grass as usize`) is used as an
/// index into material and property arrays, so the order must remain stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TileType {
    /// Open ocean — impassable.
    #[default]
    DeepWater = 0,
    /// Coastal shallows — impassable.
    ShallowWater = 1,
    /// Sandy beach or desert — passable, slow movement.
    Sand = 2,
    /// Open grassland — passable, normal movement speed.
    Grass = 3,
    /// Dense forest — passable, reduced movement speed.
    Forest = 4,
    /// Rocky mountain terrain — passable, very slow movement.
    Mountain = 5,
    /// Snow-capped peaks — passable, slow movement.
    Snow = 6,
}

impl TileType {
    /// All variants in discriminant order.
    ///
    /// Use this to build parallel arrays indexed by tile type, e.g. material
    /// handles:
    /// ```
    /// let materials: Vec<_> = TileType::ALL.iter()
    ///     .map(|t| t.color())
    ///     .collect();
    /// ```
    pub const ALL: [TileType; 7] = [
        TileType::DeepWater,
        TileType::ShallowWater,
        TileType::Sand,
        TileType::Grass,
        TileType::Forest,
        TileType::Mountain,
        TileType::Snow,
    ];

    /// The display colour for this tile type.
    pub fn color(self) -> Color {
        match self {
            TileType::DeepWater => Color::srgb(0.05, 0.15, 0.55),
            TileType::ShallowWater => Color::srgb(0.20, 0.45, 0.80),
            TileType::Sand => Color::srgb(0.85, 0.78, 0.50),
            TileType::Grass => Color::srgb(0.30, 0.60, 0.20),
            TileType::Forest => Color::srgb(0.10, 0.38, 0.12),
            TileType::Mountain => Color::srgb(0.50, 0.48, 0.45),
            TileType::Snow => Color::srgb(0.90, 0.92, 0.95),
        }
    }

    /// Movement cost multiplier (1.0 = normal speed; higher = slower).
    ///
    /// Impassable tiles return [`f32::INFINITY`].
    pub fn movement_cost(self) -> f32 {
        match self {
            TileType::DeepWater => f32::INFINITY,
            TileType::ShallowWater => f32::INFINITY,
            TileType::Sand => 1.5,
            TileType::Grass => 1.0,
            TileType::Forest => 1.8,
            TileType::Mountain => 3.0,
            TileType::Snow => 2.5,
        }
    }

    /// Returns `true` if the player can walk on this tile.
    pub fn is_passable(self) -> bool {
        self.movement_cost().is_finite()
    }

    /// Maps this tile type to the [`Biome`] that covers it.
    ///
    /// Water tiles return `None` — they are impassable and carry no monster
    /// pool.
    pub fn biome(self) -> Option<Biome> {
        match self {
            TileType::Grass => Some(Biome::Grasslands),
            TileType::Forest => Some(Biome::Forest),
            TileType::Sand => Some(Biome::Desert),
            TileType::Snow => Some(Biome::Tundra),
            TileType::Mountain => Some(Biome::Volcanic),
            TileType::DeepWater | TileType::ShallowWater => None,
        }
    }

    /// Converts a raw height value (noise output, roughly `[−1, 1]`) to a
    /// tile type using fixed elevation thresholds.
    fn from_height(h: f64) -> Self {
        match h {
            h if h < -0.30 => TileType::DeepWater,
            h if h < -0.05 => TileType::ShallowWater,
            h if h < 0.05 => TileType::Sand,
            h if h < 0.45 => TileType::Grass,
            h if h < 0.65 => TileType::Forest,
            h if h < 0.80 => TileType::Mountain,
            _ => TileType::Snow,
        }
    }
}

// ---------------------------------------------------------------------------
// Biome
// ---------------------------------------------------------------------------

/// Biome regions that determine which monster pool appears on a given tile.
///
/// Each [`TileType`] maps to exactly one biome (or `None` for water tiles)
/// via [`TileType::biome`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Biome {
    /// Open grassland — home to `Slime` monsters.
    Grasslands,
    /// Dense forest — home to `Wolf` monsters.
    Forest,
    /// Sandy desert — home to `Scorpion` monsters.
    Desert,
    /// Snow-covered tundra — home to `IceGolem` monsters.
    Tundra,
    /// Volcanic mountain terrain — home to `LavaSpider` monsters.
    Volcanic,
}

// ---------------------------------------------------------------------------
// Map resource
// ---------------------------------------------------------------------------

/// The world map: a `MAP_SIZE × MAP_SIZE` grid of [`TileType`]s.
///
/// Stored as a flat `Vec` in **row-major order**: the tile at column `x`,
/// row `y` lives at index `y * MAP_SIZE + x`.
///
/// Insert this via [`Map::generate`] and access it as a Bevy [`Resource`].
#[derive(Resource)]
pub struct Map {
    tiles: Vec<TileType>,
}

impl Map {
    // -----------------------------------------------------------------------
    // Construction
    // -----------------------------------------------------------------------

    /// Creates an empty map filled entirely with [`TileType::DeepWater`].
    pub fn new() -> Self {
        Self {
            tiles: vec![TileType::DeepWater; (MAP_SIZE * MAP_SIZE) as usize],
        }
    }

    /// Generates a terrain map using [`SimplexNoise`] fractional Brownian
    /// motion with 4 octaves.
    ///
    /// The `seed` controls the random layout; the same seed always produces
    /// the same map.
    pub fn generate(seed: u64) -> Self {
        let noise = SimplexNoise::new(seed);
        let mut tiles = Vec::with_capacity((MAP_SIZE * MAP_SIZE) as usize);

        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                // Normalise tile position to [0, 1] then scale to noise domain.
                // A scale factor of 6.0 produces continental-sized features.
                let nx = x as f64 / MAP_SIZE as f64 * 6.0;
                let ny = y as f64 / MAP_SIZE as f64 * 6.0;
                let height = noise.fbm(nx, ny, 4, 0.5, 2.0);
                tiles.push(TileType::from_height(height));
            }
        }

        Self { tiles }
    }

    // -----------------------------------------------------------------------
    // Data access
    // -----------------------------------------------------------------------

    /// Returns the [`TileType`] at tile coordinates `(x, y)`.
    ///
    /// Returns [`TileType::DeepWater`] for out-of-bounds coordinates so that
    /// boundary checks in movement code can simply test passability.
    pub fn get(&self, x: u32, y: u32) -> TileType {
        if x >= MAP_SIZE || y >= MAP_SIZE {
            return TileType::DeepWater;
        }
        self.tiles[(y * MAP_SIZE + x) as usize]
    }

    // -----------------------------------------------------------------------
    // Coordinate conversion helpers
    // -----------------------------------------------------------------------

    /// Converts tile coordinates `(x, y)` to the world-space **centre** of
    /// that tile.
    ///
    /// The full map is centred on the world origin.
    pub fn tile_to_world(x: u32, y: u32) -> Vec2 {
        let half = MAP_SIZE as f32 * TILE_SIZE * 0.5;
        Vec2::new(
            x as f32 * TILE_SIZE - half + TILE_SIZE * 0.5,
            y as f32 * TILE_SIZE - half + TILE_SIZE * 0.5,
        )
    }

    /// Converts a world-space position to tile coordinates.
    ///
    /// Returns `None` if the position lies outside the map boundary.
    pub fn world_to_tile(world: Vec2) -> Option<(u32, u32)> {
        let half = MAP_SIZE as f32 * TILE_SIZE * 0.5;
        let tx = ((world.x + half) / TILE_SIZE).floor();
        let ty = ((world.y + half) / TILE_SIZE).floor();

        if tx >= 0.0 && tx < MAP_SIZE as f32 && ty >= 0.0 && ty < MAP_SIZE as f32 {
            Some((tx as u32, ty as u32))
        } else {
            None
        }
    }

    /// Returns `true` if a circle of the given `radius` centred at `world_pos`
    /// does not overlap any impassable tile and lies fully within the map boundary.
    ///
    /// Uses exact circle-vs-AABB distance testing, so the player's circular
    /// sprite never visually penetrates an impassable tile boundary.  Pass the
    /// player's world-space [`Transform`] translation and the player's radius
    /// constant to get a physically correct collision answer.
    ///
    /// [`Transform`]: bevy::transform::components::Transform
    pub fn circle_passable(&self, world_pos: Vec2, radius: f32) -> bool {
        let half = MAP_SIZE as f32 * TILE_SIZE * 0.5;

        // Reject any position where the circle extends outside the map boundary.
        if world_pos.x - radius < -half
            || world_pos.x + radius > half
            || world_pos.y - radius < -half
            || world_pos.y + radius > half
        {
            return false;
        }

        // Tile-index range covered by the circle's axis-aligned bounding box.
        let tx_min = ((world_pos.x - radius + half) / TILE_SIZE).floor() as u32;
        let tx_max = ((world_pos.x + radius + half) / TILE_SIZE)
            .floor()
            .min(MAP_SIZE as f32 - 1.0) as u32;
        let ty_min = ((world_pos.y - radius + half) / TILE_SIZE).floor() as u32;
        let ty_max = ((world_pos.y + radius + half) / TILE_SIZE)
            .floor()
            .min(MAP_SIZE as f32 - 1.0) as u32;

        for ty in ty_min..=ty_max {
            for tx in tx_min..=tx_max {
                if !self.get(tx, ty).is_passable() {
                    // World-space AABB of this impassable tile.
                    let tile_min = Vec2::new(
                        tx as f32 * TILE_SIZE - half,
                        ty as f32 * TILE_SIZE - half,
                    );
                    let tile_max = tile_min + Vec2::splat(TILE_SIZE);

                    // Closest point on the AABB to the circle centre.
                    let closest = world_pos.clamp(tile_min, tile_max);
                    if closest.distance_squared(world_pos) < radius * radius {
                        return false;
                    }
                }
            }
        }
        true
    }

    /// Finds the nearest passable tile to the centre of the map.
    ///
    /// Performs an expanding-box scan so that inland tiles are preferred.
    /// Returns the world-space centre of the found tile, or [`Vec2::ZERO`]
    /// as a last-resort fallback (should never occur on a normal map).
    pub fn find_spawn_position(&self) -> Vec2 {
        let cx = MAP_SIZE / 2;
        let cy = MAP_SIZE / 2;

        for r in 0..(MAP_SIZE / 2) {
            let x_min = cx.saturating_sub(r);
            let x_max = (cx + r).min(MAP_SIZE - 1);
            let y_min = cy.saturating_sub(r);
            let y_max = (cy + r).min(MAP_SIZE - 1);

            for x in x_min..=x_max {
                for y in y_min..=y_max {
                    if self.get(x, y).is_passable() {
                        return Self::tile_to_world(x, y);
                    }
                }
            }
        }

        Vec2::ZERO // unreachable on any realistic FBM terrain
    }
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tile_to_world_centre_of_map_is_near_origin() {
        let centre = Map::tile_to_world(MAP_SIZE / 2, MAP_SIZE / 2);
        // The centre tile's world position should be close to (0, 0).
        assert!(centre.x.abs() <= TILE_SIZE, "x {}", centre.x);
        assert!(centre.y.abs() <= TILE_SIZE, "y {}", centre.y);
    }

    #[test]
    fn world_to_tile_roundtrip() {
        for (tx, ty) in [(0, 0), (10, 20), (128, 128), (255, 255)] {
            let world = Map::tile_to_world(tx, ty);
            let back = Map::world_to_tile(world).expect("should be inside map");
            assert_eq!(back, (tx, ty), "roundtrip failed for ({tx}, {ty})");
        }
    }

    #[test]
    fn world_to_tile_outside_map_returns_none() {
        let far = Vec2::new(1_000_000.0, 1_000_000.0);
        assert!(Map::world_to_tile(far).is_none());
    }

    #[test]
    fn generate_fills_all_tiles() {
        let map = Map::generate(0);
        assert_eq!(map.tiles.len(), (MAP_SIZE * MAP_SIZE) as usize);
    }

    #[test]
    fn generate_is_reproducible() {
        let a = Map::generate(77);
        let b = Map::generate(77);
        assert_eq!(a.tiles, b.tiles);
    }

    #[test]
    fn generate_differs_by_seed() {
        let a = Map::generate(1);
        let b = Map::generate(2);
        assert_ne!(a.tiles, b.tiles);
    }

    #[test]
    fn find_spawn_position_is_passable() {
        let map = Map::generate(12345);
        let pos = map.find_spawn_position();
        let (tx, ty) = Map::world_to_tile(pos).expect("spawn position inside map");
        assert!(
            map.get(tx, ty).is_passable(),
            "spawn tile ({tx},{ty}) is not passable: {:?}",
            map.get(tx, ty)
        );
    }

    #[test]
    fn get_oob_returns_deep_water() {
        let map = Map::new();
        assert_eq!(map.get(MAP_SIZE, 0), TileType::DeepWater);
        assert_eq!(map.get(0, MAP_SIZE), TileType::DeepWater);
    }

    #[test]
    fn tile_type_passability() {
        assert!(!TileType::DeepWater.is_passable());
        assert!(!TileType::ShallowWater.is_passable());
        assert!(TileType::Grass.is_passable());
        assert!(TileType::Sand.is_passable());
        assert!(TileType::Forest.is_passable());
        assert!(TileType::Mountain.is_passable());
        assert!(TileType::Snow.is_passable());
    }

    #[test]
    fn circle_passable_at_spawn_with_small_radius() {
        let map = Map::generate(12345);
        let spawn = map.find_spawn_position();
        // At the spawn position (centre of a passable tile) with a very small
        // radius the result must be true — the circle is entirely within a
        // passable tile.
        assert!(
            map.circle_passable(spawn, 0.1),
            "spawn position should be passable with a tiny radius"
        );
    }

    #[test]
    fn circle_passable_all_water_map_fails() {
        // Map::new() fills every tile with DeepWater (impassable).
        let map = Map::new();
        // Any position whose circle overlaps the map should fail.
        assert!(
            !map.circle_passable(Vec2::ZERO, 1.0),
            "all-water map: position at origin should not be passable"
        );
    }

    #[test]
    fn circle_passable_outside_map_boundary_fails() {
        let map = Map::generate(12345);
        let half = MAP_SIZE as f32 * TILE_SIZE * 0.5;
        // A circle whose edge extends beyond the map boundary must be rejected.
        assert!(
            !map.circle_passable(Vec2::new(half + 1.0, 0.0), 1.0),
            "position outside map (x > half) should not be passable"
        );
        assert!(
            !map.circle_passable(Vec2::new(0.0, half + 1.0), 1.0),
            "position outside map (y > half) should not be passable"
        );
    }

    #[test]
    fn circle_passable_radius_matters() {
        // Use a generated map and find a passable spawn location.
        let map = Map::generate(12345);
        let spawn = map.find_spawn_position();
        // With a tiny radius the position is passable.
        assert!(map.circle_passable(spawn, 0.1));
        // With a radius equal to the full map size the circle definitely extends
        // into impassable (water) tiles — should fail.
        let huge_radius = MAP_SIZE as f32 * TILE_SIZE;
        assert!(!map.circle_passable(spawn, huge_radius));
    }
}
