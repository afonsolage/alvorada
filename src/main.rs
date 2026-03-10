// Copyright 2024 Afonso Lage
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Alvorada — a top-down 2D game built with Bevy.
//!
//! # Plugins
//! | Plugin | Responsibility |
//! |--------|----------------|
//! | [`terrain::TerrainPlugin`] | Procedurally generated 256×256 tile map. |
//! | [`player::PlayerPlugin`]   | Player circle entity and WASD movement. |
//! | [`camera::CameraPlugin`]   | Camera that follows the player + mouse-wheel zoom. |
//! | [`monster::MonsterPlugin`] | Monster entities and biome-based spawn system. |
//! | [`combat::CombatPlugin`]   | Shared `Health` component and combat systems. |

mod camera;
mod combat;
mod monster;
mod player;
mod terrain;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Alvorada".into(),
                // Bind the canvas element used in the web build
                canvas: Some("#bevy".to_owned()),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins((
            terrain::TerrainPlugin,
            player::PlayerPlugin,
            camera::CameraPlugin,
            monster::MonsterPlugin,
            combat::CombatPlugin,
        ))
        .run();
}
