//! Utilities for the menu camera.
//!
//! The in-game camera is not dealt with here because it will created and managed automatically by
//! [`bones_bevy_renderer`].
//!
//! [`bones_bevy_renderer`]: https://docs.rs/bones_bevy_renderer

use bevy::render::camera::ScalingMode;

use crate::prelude::*;

/// Marker component added to the camera that is used to render the main menu.
#[derive(Reflect, Component, Default)]
#[reflect(Component, Default)]
pub struct MenuCamera;

/// Helper function to spawn the menu camera.
///
/// Called in [`loading::GameLoader::load()`].
pub fn spawn_menu_camera(commands: &mut Commands, core: &CoreMeta) -> Entity {
    commands
        .spawn((
            Name::new("Menu Camera"),
            MenuCamera,
            Camera2dBundle {
                // Note: This is different than just omitting this transform field because
                // Camera2DBundle's default transform is not the same as Transform::default().
                transform: default(),
                projection: OrthographicProjection {
                    scaling_mode: ScalingMode::FixedVertical(core.camera.default_height),
                    ..default()
                },
                ..default()
            },
        ))
        .id()
}
