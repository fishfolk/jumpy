use bevy::render::{camera::ScalingMode, view::RenderLayers};
use bevy_parallax::ParallaxCameraComponent;

use crate::{metadata::GameMeta, prelude::*};

pub struct Camera;

impl Plugin for Camera {
    fn build(&self, app: &mut App) {
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

#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
pub struct EditorCamera;

#[derive(Bundle)]
pub struct CameraBundle {
    #[bundle]
    camera_bundle: Camera2dBundle,
    render_layers: RenderLayers,
    parallax_camera_component: ParallaxCameraComponent,
}

pub fn spawn_game_camera(commands: &mut Commands, game: &GameMeta) -> Entity {
    commands
        .spawn()
        .insert(Name::new("Game Camera"))
        .insert(GameCamera)
        .insert_bundle(CameraBundle {
            camera_bundle: {
                let mut camera = Camera2dBundle::default();

                camera.projection.scaling_mode =
                    ScalingMode::FixedVertical(game.camera_height as f32);
                camera.transform = default();

                camera
            },
            render_layers: RenderLayers::layer(GameRenderLayers::DEFAULT)
                .with(GameRenderLayers::GAME),
            parallax_camera_component: ParallaxCameraComponent,
        })
        .id()
}

pub fn spawn_editor_camera(commands: &mut Commands, game: &GameMeta) -> Entity {
    commands
        .spawn()
        .insert(Name::new("Editor Camera"))
        .insert(EditorCamera)
        .insert_bundle(CameraBundle {
            camera_bundle: {
                let mut camera = Camera2dBundle::default();

                camera.projection.scaling_mode =
                    ScalingMode::FixedVertical(game.camera_height as f32);
                camera.transform = default();

                // Disable editor camera by default
                camera.camera.is_active = false;

                camera
            },
            render_layers: RenderLayers::layer(GameRenderLayers::DEFAULT)
                .with(GameRenderLayers::EDITOR),
            parallax_camera_component: ParallaxCameraComponent,
        })
        .id()
}
