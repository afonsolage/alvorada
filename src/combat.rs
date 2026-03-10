//! Combat components shared by the player and all monster entities.
//!
//! This module provides the [`Health`] component, which is attached to both
//! the player (see `player.rs`) and every monster (see `monster.rs`).
//!
//! # Usage
//! Add [`CombatPlugin`] to your [`bevy::app::App`].

use bevy::prelude::*;

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

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers combat-related systems.
///
/// Currently a placeholder — attack systems will be added in M5.
pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, _app: &mut App) {}
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
}
