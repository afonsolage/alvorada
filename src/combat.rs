//! Combat components shared by the player and all monster entities.
//!
//! This module provides the [`Health`] component, which is attached to both
//! the player (see `player.rs`) and every monster (see `monster.rs`), as well
//! as the [`CombatPlugin`] which drives the attack system.
//!
//! # Attack system
//! Pressing **Space** spawns a short-lived square hitbox in front of the player,
//! offset one tile in the direction the player last moved ([`Player::facing`]).
//! Any monster within [`ATTACK_RANGE`] takes 1–5 random damage.  Monsters that
//! reach 0 HP are immediately despawned (along with all their child entities).
//!
//! # Usage
//! Add [`CombatPlugin`] to your [`bevy::app::App`].

use bevy::prelude::*;

use crate::monster::Monster;
use crate::player::Player;
use crate::terrain::TILE_SIZE;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Hit-detection radius: half a tile.  A monster must be within this distance
/// from the attack centre to receive damage.
const ATTACK_RANGE: f32 = TILE_SIZE * 0.5;

/// Visual side length of the attack square (one full tile).
const ATTACK_VISUAL_SIZE: f32 = TILE_SIZE;

/// How long (in seconds) the attack square remains visible before despawning.
const ATTACK_LIFETIME: f32 = 0.15;

/// How far in front of the player (in world units) the attack hitbox is centred.
///
/// Set to one full tile so the hitbox spawns just ahead of the player circle
/// without overlapping it.
const ATTACK_OFFSET: f32 = TILE_SIZE;

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Current and maximum hit-points for any entity that can take damage.
///
/// Shared by the player and all monster entities so that combat systems can
/// operate on a single component type regardless of the target.
#[derive(Component)]
pub struct Health {
    /// Hit-points remaining.  Drops to 0 on death.
    pub current: i32,
    /// Maximum hit-points.  Used to calculate percentage fill for health bars.
    pub max: i32,
}

impl Health {
    /// Creates a new [`Health`] component at full health.
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    /// Returns `true` when the entity's hit-points have reached zero or below.
    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

/// Marks a short-lived attack hitbox spawned when the player presses **Space**.
///
/// The entity despawns automatically when [`lifetime`](AttackHitbox::lifetime)
/// reaches zero via the [`tick_attack_hitbox`] system.
#[derive(Component)]
pub struct AttackHitbox {
    /// Seconds remaining before this entity despawns.
    pub lifetime: f32,
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers combat-related systems.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                player_attack,
                apply_attack_damage.after(player_attack),
                tick_attack_hitbox,
            ),
        );
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Spawns an [`AttackHitbox`] in front of the player when **Space** is pressed.
///
/// The hitbox is offset from the player's position by [`ATTACK_OFFSET`] in the
/// direction the player last moved ([`Player::facing`]).
fn player_attack(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    keys: Res<ButtonInput<KeyCode>>,
    player: Single<(&Transform, &Player), With<Player>>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }

    let (transform, player_data) = *player;
    let pos = transform.translation;
    let offset = player_data.facing * ATTACK_OFFSET;
    commands.spawn((
        AttackHitbox {
            lifetime: ATTACK_LIFETIME,
        },
        Mesh2d(meshes.add(Rectangle::new(ATTACK_VISUAL_SIZE, ATTACK_VISUAL_SIZE))),
        MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgba(
            1.0, 1.0, 0.2, 0.5,
        )))),
        Transform::from_xyz(pos.x + offset.x, pos.y + offset.y, 2.0),
    ));
}

/// Applies 1–5 random damage to monsters within [`ATTACK_RANGE`] of any active hitbox.
///
/// Monsters whose HP reaches 0 are immediately despawned (recursively, including
/// their child label entities).
///
/// Randomness uses a lightweight inline LCG seeded by the entity index combined
/// with the current time, avoiding any external `rand` dependency.
fn apply_attack_damage(
    mut commands: Commands,
    time: Res<Time>,
    hitboxes: Query<&Transform, With<AttackHitbox>>,
    mut monsters: Query<(Entity, &Transform, &mut Health), With<Monster>>,
) {
    for hitbox_transform in &hitboxes {
        let attack_pos = hitbox_transform.translation.truncate();
        for (entity, monster_transform, mut health) in &mut monsters {
            let dist = attack_pos.distance(monster_transform.translation.truncate());
            if dist <= ATTACK_RANGE {
                let seed =
                    time.elapsed().as_nanos() as u64 ^ entity.to_bits();
                let damage = pseudo_rand_1_to_5(seed);
                health.current -= damage;
                if health.is_dead() {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}

/// Ticks down each [`AttackHitbox`] lifetime and despawns expired hitboxes.
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Returns a pseudo-random integer in `[1, 5]` from a 64-bit seed.
///
/// Uses a single LCG step (Knuth's multiplicative constants) so it is cheap
/// and dependency-free.
fn pseudo_rand_1_to_5(seed: u64) -> i32 {
    let v = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    ((v >> 33) % 5 + 1) as i32
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_new_starts_at_full() {
        let h = Health::new(10);
        assert_eq!(h.current, 10);
        assert_eq!(h.max, 10);
    }

    #[test]
    fn health_is_dead_when_zero() {
        let h = Health::new(0);
        assert!(h.is_dead());
    }

    #[test]
    fn health_is_dead_when_negative() {
        let mut h = Health::new(5);
        h.current = -1;
        assert!(h.is_dead());
    }

    #[test]
    fn health_is_not_dead_when_positive() {
        let h = Health::new(5);
        assert!(!h.is_dead());
    }

    #[test]
    fn pseudo_rand_1_to_5_is_in_range() {
        // Verify the LCG output is always in [1, 5] for a variety of seeds.
        for seed in [0u64, 1, 42, u64::MAX, 6364136223846793005] {
            let v = pseudo_rand_1_to_5(seed);
            assert!((1..=5).contains(&v), "seed {seed} produced out-of-range value {v}");
        }
    }
}
