use bevy::render::view::RenderLayers;
use bevy_parallax::{ParallaxCameraComponent, ParallaxResource};

use crate::{config::ENGINE_CONFIG, prelude::*};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<GameCamera>()
            .register_type::<EditorCamera>();

        if !ENGINE_CONFIG.server_mode {
            app.add_plugin(bevy_parallax::ParallaxPlugin);
        } else {
            app.init_resource::<ParallaxResource>();
        }
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