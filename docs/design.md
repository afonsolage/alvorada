# Alvorada — Game Design Document

> **Version:** 0.1 (initial draft)  
> **Status:** Work-in-progress — subsequent issues will add detail to each section.  
> **Audience:** Human designers *and* Agentic AI developers.  
> This document is the authoritative source of truth for all gameplay decisions.
> Every new feature or system **must** trace back to a section here before implementation begins.

---

## Table of Contents

1. [Overview](#1-overview)
2. [Technical Stack](#2-technical-stack)
3. [World & Environment](#3-world--environment)
4. [Game Loop](#4-game-loop)
5. [Character System](#5-character-system)
6. [Progression & Ranks](#6-progression--ranks)
7. [Map Portals](#7-map-portals)
8. [User Interface](#8-user-interface)
9. [Out-of-scope (future iterations)](#9-out-of-scope-future-iterations)

---

## 1. Overview

**Alvorada** is a 2D, top-down, grid-based RPG with procedurally generated maps.

| Attribute       | Value |
|-----------------|-------|
| Perspective     | Top-down 2D |
| Grid type       | Discrete tile grid |
| Map generation  | Procedural (per-instance) |
| Core activities | Exploration · Combat · Resource gathering · Looting |
| Player count    | Single-player (multiplayer not scoped) |

### Design Pillars

1. **Extraction loop** — Every map run is a short, high-stakes session where the player extracts resources.
2. **Meaningful choices** — Build diversity through items, skills, and passive points rather than fixed classes.
3. **Endless progression** — Ranks scale infinitely; there is always a harder map to tackle.
4. **Handcrafted feel from a random world** — Procgen maps must feel intentional thanks to biome coherence rules.

---

## 2. Technical Stack

| Layer        | Technology |
|--------------|------------|
| Language     | Rust (nightly) |
| Game engine  | [Bevy](https://bevyengine.org/) 0.18 |
| Renderer     | Bevy's built-in 2D renderer (sprites/tiles) |
| Target platforms | Native desktop · WebAssembly (wasm32) |

### Coordinate Conventions

- **Tile space**: integer `(col, row)` coordinates, origin `(0, 0)` at the top-left corner of the map.
- **World space**: Bevy's `Transform` in pixels; each tile is `16 × 16` pixels (`TILE_SIZE = 16.0`).
- The map is centred at the world origin — tile `(128, 128)` maps to `(0.0, 0.0)` in world space.
- `MAP_SIZE = 256` tiles per side (see [`src/terrain/map.rs`](../src/terrain/map.rs)).

### ECS Conventions (for AI agents)

- All gameplay systems live in Bevy **plugins** registered in `main.rs`.
- Current plugins: `TerrainPlugin`, `PlayerPlugin`, `CameraPlugin`.
- New features **must** be implemented as a new plugin (or extend an existing one) and registered in `main.rs`.
- Camera follow is scheduled in `PostUpdate` to avoid one-frame lag — do not move it to `Update`.

---

## 3. World & Environment

### 3.1 Map Structure

| Property    | Default value | Notes |
|-------------|---------------|-------|
| Width        | 256 tiles     | `MAP_SIZE` constant |
| Height       | 256 tiles     | `MAP_SIZE` constant |
| Tile size    | 16 × 16 px    | `TILE_SIZE` constant |
| Map origin   | World centre  | Tile `(128,128)` = world `(0,0)` |

Each map is a **unique, self-contained instance**. No two runs produce the same map.

### 3.2 Tile Types

The current tileset (defined in `src/terrain/map.rs`) contains seven tile types, ordered from lowest to highest elevation:

| `TileType` variant | Visual colour | Movement cost | Notes |
|-------------------|---------------|---------------|-------|
| `DeepWater`       | Deep blue     | Impassable    | Open sea / lakes |
| `ShallowWater`    | Light blue    | High          | Coastal / rivers |
| `Sand`            | Tan           | Low           | Beaches, deserts |
| `Grass`           | Green         | Normal        | Plains |
| `Forest`          | Dark green    | Medium        | Dense vegetation |
| `Mountain`        | Grey          | High          | High elevation |
| `Snow`            | White         | High          | Peaks / tundra |

> **AI agent note:** Movement cost values are not yet finalised. Treat the above table as design intent and update it when the pathfinding system is implemented.

### 3.3 Biomes

Each map has exactly **one biome**. The biome determines:

- Allowed tile-type distribution (e.g. a desert biome skews towards `Sand`).
- Weather effects active during the run.
- Monster pool available on the map.
- Resource pool available on the map.

Specific biome definitions (names, tile weights, monster pools, resource pools) will be specified in a future iteration.

### 3.4 Resource & Monster Respawn Policy

**There is no automatic respawn.** Once a resource node is harvested or a monster is killed, it is gone for that map instance permanently.

Exceptions are explicitly triggered by:
- Specific player skills (e.g. a "regrowth" skill).
- Environmental side-effects defined per biome.

These exceptions must be designed explicitly and documented here when introduced.

---

## 4. Game Loop

### 4.1 High-Level Flow

```
Player Base
    │
    ▼
Open Map Portal  ──►  Select / craft portal item
    │
    ▼
Enter Map  ──────────►  New 256×256 procgen instance
    │
    ├──  Explore tiles
    ├──  Gather resources (no respawn)
    ├──  Kill monsters (no respawn)
    └──  Collect loot
    │
    ▼
Leave Map  ◄─────────  Player chooses to exit (portal)
    │                   OR
    │              Player dies  ──►  respawn at Player Base
    │                                  (costs 1 travel charge)
    ▼
Player Base  (store loot, craft, upgrade, open next portal)
```

### 4.2 Travel Charges

- Every map portal item has a **travel charge counter** (exact count TBD, suggested default: **3**).
- Entering a map and leaving it voluntarily consume **0 charges** — the player can exit and re-enter the same portal as many times as they wish until the charges are spent.
- **Dying** inside a map consumes **1 travel charge** on respawn at the player base.
- When all charges are depleted the portal item is destroyed.

> Travel charges only decrease through death. Charges therefore represent tolerance for failure, not just the number of visits, making every death a meaningful loss.

### 4.3 Player Base

The player base is a persistent, safe area that is **never** a procedurally generated map. It serves as:
- Storage for gathered resources and loot.
- Crafting station (details TBD).
- Starting point for opening new portals.

---

## 5. Character System

Alvorada has **no fixed classes**. Character identity emerges entirely from three pillars:

| Pillar          | Description |
|-----------------|-------------|
| **Items**       | Equipped gear that grants stats and bonuses |
| **Skills**      | Active abilities the player can slot and use |
| **Passive points** | Points spent in a passive skill tree to enhance stats/skills |

This means two players can reach similar power levels through completely different build paths.

Detailed design of items, skills, and the passive tree is deferred to future issues.

---

## 6. Progression & Ranks

### 6.1 Map Rank

- Every map has an integer **rank**, starting at **1** with **no upper limit**.
- Rank affects:
  - Monster difficulty (health, damage, abilities).
  - Resource quality (yield, rarity tier).
  - Loot quantity and quality (drop rates, item level).

### 6.2 Rank Scaling

Exact formulae are TBD. The intended feel:
- Rank 1–5: Tutorial / early game — forgiving, introduces basic gameplay mechanics (exploration, harvesting, simple combat).
- Rank 6–20: Mid game — requires intentional builds.
- Rank 21+: Late/end game — high investment required; rewards scale significantly.

> **Note:** Biome-specific mechanics (weather, unique hazards, special resource interactions) are not yet defined. When they are documented (see [Section 9](#9-out-of-scope-future-iterations)), this scaling table should be updated to describe how biome mechanics interact with rank.

### 6.3 How Rank Is Chosen

Rank is a property of the **portal item** used to open the map. Higher-rank portals are obtained through crafting or as loot from high-rank runs (crafting details TBD).

---

## 7. Map Portals

### 7.1 Purpose

A **map portal item** is the mechanism by which the player accesses a map. It encapsulates:

| Property        | Description |
|-----------------|-------------|
| `biome`         | Target biome type |
| `rank`          | Difficulty / reward rank |
| `travel_charges`| Remaining entries before the item is destroyed |
| `modifiers`     | Optional crafted modifiers (e.g. +resource quality, +monster density) |

### 7.2 Crafting Portals

Players can **craft** portal items to target-farm desired biomes or modifiers. The crafting system is TBD, but the expected inputs are:
- A base portal item (found as loot or obtained from the player base).
- Crafting materials specific to the desired biome/modifier.

### 7.3 Portal Modifiers

Portal modifiers allow tuning the map's contents. Examples (not final):

| Modifier | Effect |
|----------|--------|
| `+resource_density` | More resource nodes on the map |
| `+elite_monsters`   | Higher ratio of elite enemies |
| `+loot_quantity`    | Increased drop count |
| `+biome_locked`     | Guarantees a specific biome tile composition |

---

## 8. User Interface

### 8.1 Design Philosophy

- **Simple and clean** — inspired by classic RPG interfaces.
- **Moveable windows** — all panels can be repositioned by the player.
- **HUD is always visible** — critical info is never hidden.

### 8.2 HUD Elements

| Element       | Position (default) | Content |
|---------------|-------------------|---------|
| Health bar    | Bottom-left        | Current / max HP |
| Resource bar  | Bottom-left        | Magic / mana / energy (resource type TBD) |
| Skill bar     | Bottom-centre      | Slotted active skills (hotkeys 1–N) |
| Mini-map      | Top-right          | Explored tiles of current map |
| Travel charges| Top-left           | Remaining charges on active portal |

### 8.3 Camera Controls

| Control            | Action       |
|--------------------|--------------|
| Mouse scroll up    | Zoom in      |
| Mouse scroll down  | Zoom out     |

Zoom is implemented via an orthographic projection scale clamped to `[0.2, 10.0]`.

| Setting | Value | Notes |
|---------|-------|-------|
| Default sensitivity | `0.01` | Scale units per scroll unit |
| Min sensitivity | `0.01` | Lower bound on the in-game slider |
| Max sensitivity | `0.30` | Upper bound on the in-game slider |
| Initial zoom scale | `3.0` | Wide view on first load |

A zoom-sensitivity slider is displayed in the bottom-right corner of the screen so players can tune the speed without restarting.

### 8.4 Windowed Panels

The following panels open on demand and can be repositioned:

| Panel        | Trigger | Content |
|--------------|---------|---------|
| Inventory    | `I`     | All held items and loot |
| Character    | `C`     | Stats, passive point allocations |
| Skills       | `K`     | Skill list, slot assignment |
| Map          | `M`     | Full map view (explored areas) |
| Crafting     | `F`     | Portal and item crafting |

> **AI agent note:** Window keybindings above are *suggested* defaults. Final bindings will be specified when the UI system is implemented. Do not hardcode them — use a configurable input map.

---

## 9. Out-of-scope (future iterations)

The following topics are explicitly deferred and **must not** be designed or implemented until a dedicated issue is opened:

- Specific biome definitions (names, tile weights, monster/resource pools).
- Monster AI and combat mechanics.
- Item stat system and loot tables.
- Skill definitions and the passive skill tree.
- Crafting system details.
- Audio / sound design.
- Multiplayer.
- Map size variants other than 256×256.
- Story / narrative.

---

*This document will be updated iteratively. Each update should increment the version number at the top and note what changed.*
