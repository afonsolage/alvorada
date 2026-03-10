//! Monster entities, types, and spawn systems.
//!
//! Each biome on the map hosts a distinct monster species, rendered as a
//! coloured equilateral triangle.  Monsters are spawned at startup at random
//! passable tile positions within their biome, using a deterministic seed so
//! the layout is reproducible for the same map seed.
//!
//! # Monster types
//!
//! | [`MonsterType`] | Biome | Colour |
//! |-----------------|-------|--------|
//! | `Slime`         | Grasslands | Lime green |
//! | `Wolf`          | Forest     | Grey-brown |
//! | `Scorpion`      | Desert     | Sandy yellow |
//! | `IceGolem`      | Tundra     | Pale cyan |
//! | `LavaSpider`    | Volcanic   | Deep orange |
//!
//! # Usage
//! Add [`MonsterPlugin`] to your [`bevy::app::App`].

use bevy::prelude::*;

use crate::combat::Health;
use crate::terrain::{Biome, Map, TILE_SIZE};
use crate::terrain::map::MAP_SIZE;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Circumradius of the triangle mesh used to represent a monster.
///
/// Set to 55 % of [`TILE_SIZE`] (≈ 8.8 px) so a monster fits visibly inside
/// one tile while remaining distinct from the player circle.
const MONSTER_RADIUS: f32 = TILE_SIZE * 0.55;

/// Starting hit-points for every monster.
const MONSTER_HP: i32 = 10;

/// Number of monsters spawned per biome type on startup.
const MONSTERS_PER_BIOME: usize = 10;

/// Width of the floating health bar rendered above each monster (world units).
const HEALTH_BAR_WIDTH: f32 = 28.0;

/// Height of the floating health bar rendered above each monster (world units).
const HEALTH_BAR_HEIGHT: f32 = 4.0;

// ---------------------------------------------------------------------------
// Monster type
// ---------------------------------------------------------------------------

/// The species of a monster, determining its colour and home biome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterType {
    /// Lime-green blob found in grassland biomes.
    Slime,
    /// Grey-brown predator found in forest biomes.
    Wolf,
    /// Sandy-yellow arachnid found in desert biomes.
    Scorpion,
    /// Pale-cyan construct found in tundra biomes.
    IceGolem,
    /// Deep-orange spider found in volcanic biomes.
    LavaSpider,
}

impl MonsterType {
    /// Returns the display name and render colour for this monster type.
    pub fn info(self) -> (&'static str, Color) {
        match self {
            MonsterType::Slime => ("Slime", Color::srgb(0.4, 0.9, 0.2)),
            MonsterType::Wolf => ("Wolf", Color::srgb(0.55, 0.45, 0.30)),
            MonsterType::Scorpion => ("Scorpion", Color::srgb(0.85, 0.72, 0.20)),
            MonsterType::IceGolem => ("Ice Golem", Color::srgb(0.60, 0.88, 0.95)),
            MonsterType::LavaSpider => ("Lava Spider", Color::srgb(0.95, 0.35, 0.05)),
        }
    }
}

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks a monster entity and records its species and display name.
#[derive(Component)]
#[allow(dead_code)] // fields read by label/combat systems (M4, M5)
pub struct Monster {
    /// The species driving this monster's colour and biome association.
    pub monster_type: MonsterType,
    /// Display name shown in floating labels (populated from [`MonsterType::info`]).
    pub name: &'static str,
}

/// Marker component for the floating name text child of a monster entity.
#[derive(Component)]
pub struct MonsterNameLabel;

/// Marker component for the health bar background quad child of a monster entity.
#[derive(Component)]
pub struct MonsterHealthBarBg;

/// Marker component for the health bar fill quad child of a monster entity.
///
/// Queried by [`update_monster_health_bars`] to rescale the fill width each frame.
#[derive(Component)]
pub struct MonsterHealthBarFill;

// ---------------------------------------------------------------------------
// Biome → monster mapping
// ---------------------------------------------------------------------------

impl Biome {
    /// Returns the [`MonsterType`] that inhabits this biome.
    pub fn monster_type(self) -> MonsterType {
        match self {
            Biome::Grasslands => MonsterType::Slime,
            Biome::Forest => MonsterType::Wolf,
            Biome::Desert => MonsterType::Scorpion,
            Biome::Tundra => MonsterType::IceGolem,
            Biome::Volcanic => MonsterType::LavaSpider,
        }
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers monster-related resources and systems.
pub struct MonsterPlugin;

impl Plugin for MonsterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_monsters)
            .add_systems(Update, update_monster_health_bars);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns [`MONSTERS_PER_BIOME`] triangle monsters for each biome at startup.
///
/// Tile positions are sampled deterministically using a simple LCG seeded by
/// the map seed (`12345`) XOR-ed with a per-biome offset, so the layout is
/// always the same for a given map.
///
/// Each monster entity has three child entities attached:
/// - A [`MonsterNameLabel`] `Text2d` floating above the triangle.
/// - A [`MonsterHealthBarBg`] dark-grey rectangle behind the fill.
/// - A [`MonsterHealthBarFill`] red rectangle scaled each frame to current HP.
fn spawn_monsters(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    map: Res<Map>,
) {
    let biomes = [
        Biome::Grasslands,
        Biome::Forest,
        Biome::Desert,
        Biome::Tundra,
        Biome::Volcanic,
    ];

    // Shared meshes for the health bar quads (reused across all monsters).
    let bar_bg_mesh = meshes.add(Rectangle::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT));
    let bar_fill_mesh = meshes.add(Rectangle::new(HEALTH_BAR_WIDTH, HEALTH_BAR_HEIGHT));
    let bar_bg_material = materials.add(ColorMaterial::from_color(Color::srgb(0.2, 0.2, 0.2)));
    let bar_fill_material = materials.add(ColorMaterial::from_color(Color::srgb(0.85, 0.15, 0.15)));

    for (biome_idx, biome) in biomes.iter().enumerate() {
        let monster_type = biome.monster_type();
        let (name, color) = monster_type.info();

        // Collect all passable tiles belonging to this biome.
        let biome_tiles: Vec<(u32, u32)> = (0..MAP_SIZE)
            .flat_map(|y| (0..MAP_SIZE).map(move |x| (x, y)))
            .filter(|&(x, y)| map.get(x, y).biome() == Some(*biome))
            .collect();

        if biome_tiles.is_empty() {
            continue;
        }

        // Create shared mesh and material for this monster type.
        let mesh = meshes.add(RegularPolygon::new(MONSTER_RADIUS, 3));
        let material = materials.add(ColorMaterial::from_color(color));

        // Seed the LCG with the map seed XOR-ed with the biome index so each
        // biome gets a distinct but fully deterministic spawn pattern.
        let mut rng = 12345u64 ^ biome_idx as u64;

        for _ in 0..MONSTERS_PER_BIOME {
            // LCG step (Knuth's multiplicative constants).
            rng = rng
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let idx = ((rng >> 33) as usize) % biome_tiles.len();
            let (tx, ty) = biome_tiles[idx];
            let pos = Map::tile_to_world(tx, ty);

            commands
                .spawn((
                    Monster { monster_type, name },
                    Health::new(MONSTER_HP),
                    Mesh2d(mesh.clone()),
                    MeshMaterial2d(material.clone()),
                    Transform::from_xyz(pos.x, pos.y, 1.0),
                ))
                .with_children(|parent| {
                    // Floating name label above the triangle.
                    parent.spawn((
                        MonsterNameLabel,
                        Text2d::new(name),
                        TextFont {
                            font_size: 10.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        Transform::from_xyz(0.0, MONSTER_RADIUS + 14.0, 0.1),
                    ));

                    // Health bar background (dark grey).
                    parent.spawn((
                        MonsterHealthBarBg,
                        Mesh2d(bar_bg_mesh.clone()),
                        MeshMaterial2d(bar_bg_material.clone()),
                        Transform::from_xyz(0.0, MONSTER_RADIUS + 6.0, 0.1),
                    ));

                    // Health bar fill (red, starts at full width).
                    // Positioned at the same Y as the background but slightly in front.
                    // Scaled on X each frame by `update_monster_health_bars`.
                    parent.spawn((
                        MonsterHealthBarFill,
                        Mesh2d(bar_fill_mesh.clone()),
                        MeshMaterial2d(bar_fill_material.clone()),
                        Transform::from_xyz(0.0, MONSTER_RADIUS + 6.0, 0.11),
                    ));
                });
        }
    }
}

/// Rescales each monster's health bar fill to reflect current HP.
///
/// The fill quad is anchored at its left edge by combining X-axis scaling with
/// a compensating translation, so the bar empties from right to left.
fn update_monster_health_bars(
    monsters: Query<(&Health, &Children), With<Monster>>,
    mut fills: Query<&mut Transform, With<MonsterHealthBarFill>>,
) {
    for (health, children) in &monsters {
        let pct = (health.current as f32 / health.max as f32).clamp(0.0, 1.0);
        for &child in children {
            if let Ok(mut t) = fills.get_mut(child) {
                t.scale.x = pct;
                // Shift left so the left edge stays fixed as the bar shrinks.
                t.translation.x = HEALTH_BAR_WIDTH * 0.5 * (pct - 1.0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monster_type_info_returns_non_empty_name() {
        for mt in [
            MonsterType::Slime,
            MonsterType::Wolf,
            MonsterType::Scorpion,
            MonsterType::IceGolem,
            MonsterType::LavaSpider,
        ] {
            let (name, _color) = mt.info();
            assert!(!name.is_empty(), "MonsterType {:?} has empty name", mt);
        }
    }

    #[test]
    fn biome_monster_type_covers_all_biomes() {
        // Every biome must return a valid monster type (no panic).
        for biome in [
            Biome::Grasslands,
            Biome::Forest,
            Biome::Desert,
            Biome::Tundra,
            Biome::Volcanic,
        ] {
            let _ = biome.monster_type();
        }
    }

    #[test]
    fn tile_biome_mapping_is_consistent() {
        use crate::terrain::map::TileType;
        assert_eq!(TileType::Grass.biome(), Some(Biome::Grasslands));
        assert_eq!(TileType::Forest.biome(), Some(Biome::Forest));
        assert_eq!(TileType::Sand.biome(), Some(Biome::Desert));
        assert_eq!(TileType::Snow.biome(), Some(Biome::Tundra));
        assert_eq!(TileType::Mountain.biome(), Some(Biome::Volcanic));
        assert_eq!(TileType::DeepWater.biome(), None);
        assert_eq!(TileType::ShallowWater.biome(), None);
    }
}
