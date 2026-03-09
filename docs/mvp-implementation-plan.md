# Alvorada — MVP Implementation Plan

> **Version:** 0.1  
> **Status:** Ready for implementation  
> **Audience:** Agentic AI developers  
> **Source issue:** "Create initial implementation plan"  
> This document is the implementation-level companion to [`design.md`](./design.md).
> Every task traces back to the MVP scope described in that issue.

---

## Table of Contents

1. [Scope Summary](#1-scope-summary)
2. [Architecture Overview](#2-architecture-overview)
3. [Milestones](#3-milestones)
   - [M1 – Monster System](#m1--monster-system)
   - [M2 – Biome Definitions](#m2--biome-definitions)
   - [M3 – Player Health & HUD](#m3--player-health--hud)
   - [M4 – Monster Health Bars & Name Display](#m4--monster-health-bars--name-display)
   - [M5 – Attack System](#m5--attack-system)
4. [File & Module Map](#4-file--module-map)
5. [Key Constants](#5-key-constants)
6. [Implementation Notes for AI Agents](#6-implementation-notes-for-ai-agents)

---

## 1. Scope Summary

| Feature | Description |
|---------|-------------|
| **Monster entities** | Triangles, each monster type has a unique colour |
| **Biomes** | Distinct areas (Grasslands, Forest, Desert, Tundra, Volcanic) that each carry a monster pool |
| **Monster labels** | Floating name + health bar rendered above each monster |
| **Player HUD** | Fixed health bar in the bottom-left corner of the screen |
| **Attack system** | Press `Space` → spawn a square hitbox; monsters within ½ tile receive 1–5 random damage |
| **Monster death** | Despawn the entity (+ its label children) when HP reaches 0 |
| **No progression** | No XP, no levelling-up — explicitly out of scope for this MVP |

---

## 2. Architecture Overview

All new gameplay systems follow the existing ECS conventions (see `design.md §2`):

- Each feature lives in a dedicated **plugin** registered in `src/main.rs`.
- New files sit directly under `src/` (flat layout matches existing `player.rs`, `camera.rs`).
- Bevy's `Update` schedule is used for gameplay logic; `PostUpdate` is reserved for the camera.
- UI elements use Bevy's built-in UI nodes (matching the existing zoom-slider pattern in `camera.rs`).

```
src/
├── main.rs           ← register new plugins here
├── camera.rs         (unchanged)
├── player.rs         ← add Health component + player_health_system
├── monster.rs        ← new: MonsterPlugin, all monster logic
├── combat.rs         ← new: CombatPlugin, attack system
├── hud.rs            ← new: HudPlugin, player health bar UI
└── terrain/          (unchanged)
```

---

## 3. Milestones

### M1 – Monster System

**Goal:** Spawn coloured triangle monsters at random passable locations on the map.

#### Task M1-1 — Define monster types and biome pools

Create `src/monster.rs`. Define the following types:

```
MonsterType enum {
    Slime,      // biome: Grasslands  — colour: lime green   (0.4, 0.9, 0.2)
    Wolf,       // biome: Forest      — colour: grey-brown   (0.55, 0.45, 0.30)
    Scorpion,   // biome: Desert      — colour: sandy yellow (0.85, 0.72, 0.20)
    IceGolem,   // biome: Tundra      — colour: pale cyan    (0.60, 0.88, 0.95)
    LavaSpider, // biome: Volcanic    — colour: deep orange  (0.95, 0.35, 0.05)
}
```

Each variant must implement a method returning `(name: &str, color: Color)`.

#### Task M1-2 — Define the `Monster` component

```rust
#[derive(Component)]
pub struct Monster {
    pub monster_type: MonsterType,
    pub name: &'static str,
}
```

#### Task M1-3 — Define the `Health` component (shared by player and monsters)

Create a generic `Health` component in `src/combat.rs` (it is used by both M1 and M3):

```rust
#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self { Self { current: max, max } }
    pub fn is_dead(&self) -> bool { self.current <= 0 }
}
```

Each monster spawns with `Health::new(10)`.

#### Task M1-4 — Spawn monsters on startup

Add a `spawn_monsters` startup system to `MonsterPlugin`.

- Query the `Map` resource.
- For each biome (see M2), sample **8–12 random passable tiles** from the biome's eligible tile types.
- Spawn a triangle entity at each position:
  - Shape: `RegularPolygon::new(MONSTER_RADIUS, 3)` where `MONSTER_RADIUS = TILE_SIZE * 0.55`.
  - Color: from `MonsterType::color()`.
  - `Transform::from_xyz(pos.x, pos.y, 1.0)` (same Z layer as the player).
  - Components: `Monster`, `Health::new(10)`.
- Use a deterministic seed (e.g. the existing seed `12345`) so the layout is reproducible.

#### Task M1-5 — Register `MonsterPlugin`

In `src/main.rs`, add `monster::MonsterPlugin` to the plugin tuple. Add `mod monster;` declaration.

---

### M2 – Biome Definitions

**Goal:** Assign each region of the map a biome label so the monster spawner (M1-4) can populate it correctly.

#### Task M2-1 — Define `Biome` enum

Add to `src/terrain/map.rs` (or a new `src/terrain/biome.rs`):

```
Biome enum {
    Grasslands,   // primary tile: Grass
    Forest,       // primary tile: Forest
    Desert,       // primary tile: Sand
    Tundra,       // primary tile: Snow, Mountain
    Volcanic,     // primary tile: Mountain (secondary: Sand)
}
```

#### Task M2-2 — Map tile types to biomes

Add a method `TileType::biome(self) -> Option<Biome>` that maps:

| TileType | Biome |
|----------|-------|
| Grass | Grasslands |
| Forest | Forest |
| Sand | Desert |
| Snow | Tundra |
| Mountain | Volcanic |
| DeepWater / ShallowWater | `None` (impassable — no monsters) |

#### Task M2-3 — Expose biome-to-monster mapping

Add a method `Biome::monster_type(self) -> MonsterType` returning the monster type associated with each biome (matching the table in M1-1).

#### Task M2-4 — Wire biome lookup into the spawner (M1-4)

In `spawn_monsters`, for each tile, call `map.get(x, y).biome()` to determine which monster pool to draw from, then spawn accordingly.

---

### M3 – Player Health & HUD

**Goal:** Give the player a `Health` component and display it in a fixed bottom-left HUD.

#### Task M3-1 — Add `Health` to the player entity

In `src/player.rs`, attach `Health::new(100)` when spawning the player (alongside the existing `Player` marker, mesh, and material).

#### Task M3-2 — Create `HudPlugin` in `src/hud.rs`

The plugin registers two startup systems and one update system:

1. `spawn_player_hud` — builds the UI node tree.
2. `update_player_health_bar` — syncs the fill bar width to `Health::current / Health::max`.

#### Task M3-3 — Design the health bar node tree

```
Node (position: Absolute, bottom: 16px, left: 16px, flex_direction: Column)
  └─ Node (label row)
       └─ Text "HP" (font_size: 14, colour: WHITE)
  └─ Node (bar background, width: 160px, height: 16px, colour: dark grey)
       └─ Node (bar fill, width: 100%, height: 100%, colour: red (0.85, 0.15, 0.15))
            [marker component: PlayerHealthBarFill]
```

#### Task M3-4 — Health update system

```rust
fn update_player_health_bar(
    player_health: Single<&Health, With<Player>>,
    mut fill: Single<&mut Node, With<PlayerHealthBarFill>>,
) {
    let pct = (player_health.current as f32 / player_health.max as f32).clamp(0.0, 1.0);
    fill.width = Val::Percent(pct * 100.0);
}
```

#### Task M3-5 — Register `HudPlugin` in `main.rs`

Add `hud::HudPlugin` to the plugin tuple and `mod hud;` declaration.

---

### M4 – Monster Health Bars & Name Display

**Goal:** Each monster entity has a floating label (name + health bar) rendered directly above it using Bevy's world-space UI or child entities.

> **Implementation note:** Use **child mesh entities** in world space rather than Bevy UI nodes, because UI nodes are in screen space and cannot follow world-space entities without custom projection math. Each monster spawns two child entities: a name text billboard and a health-bar quad pair.

#### Task M4-1 — Define marker components

```rust
#[derive(Component)] pub struct MonsterNameLabel;
#[derive(Component)] pub struct MonsterHealthBarBg;
#[derive(Component)] pub struct MonsterHealthBarFill;
```

#### Task M4-2 — Spawn label children during monster spawn (extend M1-4)

When spawning each monster, use `commands.entity(monster_id).with_children(|parent| { … })` to attach:

1. **Name text** — Use a `Text2d` component (Bevy 0.18).
   - `Text2d::new(monster.name)`
   - `TextFont { font_size: 10.0, .. }`
   - `TextColor(Color::WHITE)`
   - `Transform::from_xyz(0.0, MONSTER_RADIUS + 14.0, 0.1)` (offset above the triangle)

2. **Health bar background** — A dark grey `Rectangle` quad.
   - Width: `HEALTH_BAR_WIDTH = 28.0`, Height: `HEALTH_BAR_HEIGHT = 4.0`
   - `Transform::from_xyz(0.0, MONSTER_RADIUS + 6.0, 0.1)`
   - Component: `MonsterHealthBarBg`

3. **Health bar fill** — A red `Rectangle` quad (child of the background).
   - Initial width = `HEALTH_BAR_WIDTH` (full, since HP starts at max)
   - `Transform::from_xyz(0.0, 0.0, 0.01)` (slightly in front of background)
   - Component: `MonsterHealthBarFill`

> **Sizing note:** Because the fill bar must shrink from the left edge, it should be anchored at the left: set `Transform::from_xyz(-HEALTH_BAR_WIDTH * 0.5 + (pct * HEALTH_BAR_WIDTH * 0.5), 0.0, 0.01)` and update its `Mesh2d` asset width each frame, or scale via `Transform::scale`. The simpler approach is to **rescale the fill entity on X**: `fill_transform.scale.x = pct;` and shift its X position to keep the left edge fixed.

#### Task M4-3 — Update fill bar each frame

```rust
fn update_monster_health_bars(
    monsters: Query<(&Health, &Children), With<Monster>>,
    mut fills: Query<&mut Transform, With<MonsterHealthBarFill>>,
) {
    for (health, children) in &monsters {
        let pct = (health.current as f32 / health.max as f32).clamp(0.0, 1.0);
        for &child in children {
            if let Ok(mut t) = fills.get_mut(child) {
                t.scale.x = pct;
                // Re-centre: left-anchor offset
                t.translation.x = HEALTH_BAR_WIDTH * 0.5 * (pct - 1.0);
            }
        }
    }
}
```

Register this system on `MonsterPlugin` under `Update`.

---

### M5 – Attack System

**Goal:** Pressing `Space` spawns a short-lived square hitbox centred on the player; any monster within half a tile takes 1–5 random damage; monsters that reach 0 HP are despawned.

#### Task M5-1 — Define the `AttackHitbox` component

Create `src/combat.rs` (if it does not already exist from M1-3):

```rust
#[derive(Component)]
pub struct AttackHitbox {
    /// Seconds remaining before this entity despawns.
    pub lifetime: f32,
}
```

#### Task M5-2 — Define the attack range constant

```rust
/// Half a tile — the radius within which a monster is hit by an attack.
const ATTACK_RANGE: f32 = TILE_SIZE * 0.5;

/// Visual side length of the attack square (one full tile).
const ATTACK_VISUAL_SIZE: f32 = TILE_SIZE;

/// How long the attack square remains visible (seconds).
const ATTACK_LIFETIME: f32 = 0.15;
```

#### Task M5-3 — `player_attack` system

Triggered when `Space` is **just pressed** (not held):

```rust
fn player_attack(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<&Transform, With<Player>>,
) {
    if !keys.just_pressed(KeyCode::Space) { return; }

    let pos = player.translation;
    commands.spawn((
        AttackHitbox { lifetime: ATTACK_LIFETIME },
        Mesh2d(meshes.add(Rectangle::new(ATTACK_VISUAL_SIZE, ATTACK_VISUAL_SIZE))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(
            Color::srgba(1.0, 1.0, 0.2, 0.5),
        ))),
        Transform::from_xyz(pos.x, pos.y, 2.0),
    ));
}
```

#### Task M5-4 — `apply_attack_damage` system

Runs immediately after `player_attack` using `chain()` or `after()` ordering:

```rust
fn apply_attack_damage(
    mut commands: Commands,
    hitboxes: Query<&Transform, With<AttackHitbox>>,
    mut monsters: Query<(Entity, &Transform, &mut Health), With<Monster>>,
) {
    for hitbox_transform in &hitboxes {
        let attack_pos = hitbox_transform.translation.truncate();
        for (entity, monster_transform, mut health) in &mut monsters {
            let dist = attack_pos.distance(monster_transform.translation.truncate());
            if dist <= ATTACK_RANGE {
                // Random damage 1–5 using a deterministic PRNG seeded per frame.
                // Use `rand` crate if available, otherwise use a simple hash of
                // the entity index + current frame count as a lightweight PRNG.
                let damage = pseudo_rand_damage(); // returns i32 in [1, 5]
                health.current -= damage;
                if health.is_dead() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
```

> **Randomness note:** The MVP may use Bevy's `GlobalEntropy` (from `bevy_prng`) or a simple deterministic formula. To avoid an external dependency, a lightweight inline PRNG seeded by `entity.index() ^ frame_count` is sufficient for a demo.

#### Task M5-5 — `tick_attack_hitbox` system

Despawns hitbox entities when their lifetime expires:

```rust
fn tick_attack_hitbox(
    mut commands: Commands,
    time: Res<Time>,
    mut hitboxes: Query<(Entity, &mut AttackHitbox)>,
) {
    for (entity, mut hitbox) in &mut hitboxes {
        hitbox.lifetime -= time.delta_secs();
        if hitbox.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
```

#### Task M5-6 — Create `CombatPlugin` and register systems

```rust
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (
            player_attack,
            apply_attack_damage.after(player_attack),
            tick_attack_hitbox,
        ));
    }
}
```

Register `combat::CombatPlugin` in `main.rs`.

---

## 4. File & Module Map

| File | New / Modified | Responsibility |
|------|---------------|----------------|
| `src/main.rs` | Modified | Register `MonsterPlugin`, `CombatPlugin`, `HudPlugin` |
| `src/player.rs` | Modified | Attach `Health::new(100)` to player on spawn |
| `src/monster.rs` | **New** | `MonsterPlugin`, `MonsterType`, `Monster` component, spawn system, health bar update system |
| `src/combat.rs` | **New** | `Health` component, `CombatPlugin`, attack systems |
| `src/hud.rs` | **New** | `HudPlugin`, player health bar UI |
| `src/terrain/map.rs` | Modified | Add `Biome` enum and `TileType::biome()` method |

---

## 5. Key Constants

| Constant | Value | Location | Purpose |
|----------|-------|----------|---------|
| `MONSTER_RADIUS` | `TILE_SIZE * 0.55` (≈ 8.8 px) | `monster.rs` | Triangle circumradius |
| `MONSTER_HP` | `10` | `monster.rs` | Starting HP for every monster |
| `MONSTERS_PER_BIOME` | `10` | `monster.rs` | Target spawn count per biome type |
| `ATTACK_RANGE` | `TILE_SIZE * 0.5` (8.0 px) | `combat.rs` | Hit detection radius |
| `ATTACK_VISUAL_SIZE` | `TILE_SIZE` (16.0 px) | `combat.rs` | Attack square side length |
| `ATTACK_LIFETIME` | `0.15` seconds | `combat.rs` | Duration before hitbox despawns |
| `HEALTH_BAR_WIDTH` | `28.0` px | `monster.rs` | Monster floating health bar width |
| `HEALTH_BAR_HEIGHT` | `4.0` px | `monster.rs` | Monster floating health bar height |
| `PLAYER_MAX_HP` | `100` | `player.rs` | Player starting HP |
| `HUD_BAR_WIDTH` | `160.0` px | `hud.rs` | Player HUD health bar width |

---

## 6. Implementation Notes for AI Agents

### Ordering constraints

1. Implement M1 **before** M2; the biome lookup is only needed by the spawner.
2. Implement the `Health` component in `combat.rs` (M1-3) **before** M3 and M4, since both milestones depend on it.
3. The `apply_attack_damage` system must run **after** `player_attack` in the same frame; use `.after(player_attack)` system ordering.

### Bevy 0.18 API reminders

- `Text2d` is the component for world-space text in Bevy 0.18 (not `Text`).
- `commands.entity(id).despawn()` recursively despawns all children — no need to manually despawn label children.
- `Mesh2d` / `MeshMaterial2d` are the 2D render components (matching the existing player and tile spawn code).
- `RegularPolygon::new(circumradius, sides)` creates triangles when `sides = 3`.
- System ordering within a plugin: `.add_systems(Update, (system_a, system_b.after(system_a)))`.

### Random damage without external crates

To avoid adding a `rand` dependency for the MVP, use a simple inline LCG seeded by the current time and entity index:

```rust
fn pseudo_rand_1_to_5(seed: u64) -> i32 {
    let v = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    ((v >> 33) % 5 + 1) as i32
}
```

Call it with a seed derived from `time.elapsed().as_nanos() as u64 ^ entity.to_bits()`.

### No progression system

Do **not** implement XP gain, levelling, loot drops, or stat increases. When a monster's HP reaches 0, call `commands.entity(entity).despawn()` and nothing else.
