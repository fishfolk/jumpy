use bevy::render::view::RenderLayers;
use bevy_parallax::ParallaxCameraComponent;

use crate::{player::PlayerIdx, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameCamera>()
            .register_type::<EditorCamera>()
            .extend_rollback_schedule(|schedule| {
                schedule.add_system_to_stage(RollbackStage::UpdateInGame, camera_controller);
            });

        app.add_plugin(bevy_parallax::ParallaxPlugin);
    }
}

/// Named [`RenderLayers`][bevy::render::view::visibility::RenderLayers] bitmasks used throughout
/// the game.
pub struct GameRenderLayers;

impl GameRenderLayers {
    pub const DEFAULT: u8 = 0;
    pub const GAME: u8 = 1;
    pub const EDITOR: u8 = 2;
}

#[derive(Reflect, Component, Default)]
#[reflect(Component, Default)]
pub struct GameCamera;

#[derive(Reflect, Component, Default)]
#[reflect(Component, Default)]
pub struct EditorCamera;

#[derive(Bundle)]
pub struct CameraBundle {
    #[bundle]
    camera_bundle: Camera2dBundle,
    render_layers: RenderLayers,
    parallax_camera_component: ParallaxCameraComponent,
}

pub fn spawn_game_camera(commands: &mut Commands) -> Entity {
    commands
        .spawn()
        .insert(Name::new("Game Camera"))
        .insert(GameCamera)
        .insert_bundle(CameraBundle {
            camera_bundle: Camera2dBundle {
                // This is different than just omitting this transform field because
                // Camera2DBundle's default transform is not the same as Transform::default().
                transform: default(),
                ..default()
            },
            render_layers: RenderLayers::layer(GameRenderLayers::DEFAULT)
                .with(GameRenderLayers::GAME),
            parallax_camera_component: ParallaxCameraComponent,
        })
        .id()
}

pub fn spawn_editor_camera(commands: &mut Commands) -> Entity {
    commands
        .spawn()
        .insert(Name::new("Editor Camera"))
        .insert(EditorCamera)
        .insert_bundle(CameraBundle {
            camera_bundle: Camera2dBundle {
                // This is different than just omitting this transform field because
                // Camera2DBundle's default transform is not the same as Transform::default().
                transform: default(),
                camera: Camera {
                    // Disable editor camera by default
                    is_active: false,
                    ..default()
                },
                ..default()
            },

            render_layers: RenderLayers::layer(GameRenderLayers::DEFAULT)
                .with(GameRenderLayers::EDITOR),
            parallax_camera_component: ParallaxCameraComponent,
        })
        .id()
}

fn camera_controller(
    players: Query<&Transform, With<PlayerIdx>>,
    mut camera: Query<
        (&mut Transform, &mut OrthographicProjection),
        (With<GameCamera>, Without<PlayerIdx>),
    >,
) {
    const LERP_FACTOR: f32 = 0.1;

    let Ok((mut camera_transform, mut projection)) = camera.get_single_mut() else {
        return;
    };

    let mut middle_point = Vec2::ZERO;
    let mut min = Vec2::new(100000.0, 100000.0);
    let mut max = Vec2::new(-100000.0, -100000.0);

    let player_count = players.iter().len();

    for player_transform in &players {
        let pos = player_transform.translation.truncate();
        middle_point += pos;

        min.x = pos.x.min(min.x);
        min.y = pos.y.min(min.y);
        max.x = pos.x.max(max.x);
        max.y = pos.y.max(max.y);
    }

    middle_point /= player_count.max(1) as f32;

    let delta = camera_transform.translation.truncate() - middle_point;
    let dist = delta * LERP_FACTOR;
    camera_transform.translation -= dist.extend(0.0);

    projection.scale = 1.25;
}
