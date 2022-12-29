use bevy::render::camera::ScalingMode;

use crate::prelude::*;

#[derive(Reflect, Component, Default)]
#[reflect(Component, Default)]
pub struct MenuCamera;

pub fn spawn_menu_camera(commands: &mut Commands, core: &CoreMeta) -> Entity {
    commands
        .spawn((
            Name::new("Game Camera"),
            MenuCamera,
            Camera2dBundle {
                // Note: This is different than just omitting this transform field because
                // Camera2DBundle's default transform is not the same as Transform::default().
                transform: default(),
                projection: OrthographicProjection {
                    scaling_mode: ScalingMode::FixedVertical(core.camera_height),
                    ..default()
                },
                ..default()
            },
        ))
        .id()
}
