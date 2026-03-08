// Copyright 2024 Afonso Lage
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn a 2D camera
    commands.spawn(Camera2d);

    // Display "Hello, World!" in the center of the screen
    commands.spawn((
        Text2d::new("Hello, World!"),
        TextFont {
            font_size: 72.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}
