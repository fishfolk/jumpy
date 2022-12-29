use std::sync::Arc;

use bevy::{prelude::*, time::FixedTimestep};
use bevy_kira_audio::prelude::*;
use bones_bevy_renderer::*;

use jumpy_core::{bevy_prelude::*, metadata::JumpyCoreAssetsPlugin};

#[cfg(not(target_arch = "wasm32"))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Deref, DerefMut, Resource)]
struct Session(GameSession);

impl HasBonesWorld for Session {
    fn world(&mut self) -> &mut bones_lib::prelude::World {
        &mut self.world
    }
}

pub fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    watch_for_changes: true,
                    ..default()
                }),
        )
        .add_plugin(AudioPlugin)
        .add_plugin(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugin(bevy::diagnostic::LogDiagnosticsPlugin::default())
        .add_plugin(BonesRendererPlugin::<Session>::new())
        .add_plugin(JumpyCoreAssetsPlugin)
        .add_startup_system(setup)
        .add_system(load)
        .init_resource::<Snapshot>()
        .add_system(snapshot_restore)
        .add_stage_before(
            CoreStage::Update,
            "update",
            SystemStage::single_threaded()
                .with_system(update_input)
                .with_system(update_game)
                .with_system(play_sounds)
                .with_run_criteria(FixedTimestep::step(1.0 / 60.0)),
        )
        .run();
}

/// Marker component for entities that are rendered for bones.
#[derive(Component)]
struct BevyBones;

#[derive(Resource)]
struct CoreMetaHandle(pub Handle<CoreMeta>);

/// Setup the game, loading the metadata and starting the game session.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let handle = asset_server.load("default.core.yaml");

    commands.insert_resource(CoreMetaHandle(handle));
}

/// Wait for game meta to load, and then start the game session.
fn load(
    mut commands: Commands,
    session: Option<ResMut<Session>>,
    meta_handle: Option<Res<CoreMetaHandle>>,
    mut core_meta_assets: ResMut<Assets<CoreMeta>>,
) {
    if session.is_some() {
        return;
    }
    let Some(meta_handle) = meta_handle else {
        return;
    };
    let Some(meta) = core_meta_assets.remove(&meta_handle.0)  else {
        return;
    };

    let session = GameSession::new(GameSessionInfo {
        map: meta.stable_maps[0].clone(),
        player_info: [
            Some(meta.players[0].clone()),
            Some(meta.players[0].clone()),
            None,
            None,
        ],
        meta: Arc::new(meta),
    });

    commands.insert_resource(Session(session));
}

/// Update the game session input.
fn update_input(
    session: Option<ResMut<Session>>,
    keyboard: Res<Input<KeyCode>>,
    mut windows: ResMut<Windows>,
) {
    let Some(mut session) = session else {
        return;
    };

    if keyboard.just_pressed(KeyCode::F11) {
        if let Some(window) = windows.get_primary_mut() {
            window.set_mode(match window.mode() {
                WindowMode::BorderlessFullscreen => WindowMode::Windowed,
                _ => WindowMode::BorderlessFullscreen,
            });
        }
    }

    session.update_input(|inputs| {
        let control = &mut inputs.players[0].control;

        let jump_pressed = keyboard.pressed(KeyCode::Space);
        control.jump_just_pressed = jump_pressed && !control.jump_pressed;
        control.jump_pressed = jump_pressed;

        let grab_pressed = keyboard.pressed(KeyCode::B);
        control.grab_just_pressed = grab_pressed && !control.grab_pressed;
        control.grab_pressed = grab_pressed;

        let shoot_pressed = keyboard.pressed(KeyCode::V);
        control.shoot_just_pressed = shoot_pressed && !control.shoot_pressed;
        control.shoot_pressed = shoot_pressed;

        let mut move_direction = Vec2::ZERO;
        if keyboard.pressed(KeyCode::D) {
            move_direction += Vec2::X;
        }
        if keyboard.pressed(KeyCode::A) {
            move_direction += Vec2::NEG_X;
        }
        if keyboard.pressed(KeyCode::W) {
            move_direction += Vec2::Y;
        }
        if keyboard.pressed(KeyCode::S) {
            move_direction += Vec2::NEG_Y;
        }
        control.move_direction = move_direction.normalize_or_zero();
    });
}

/// Update the game simulation.
fn update_game(world: &mut World) {
    let Some(mut session) = world.remove_resource::<Session>() else {
        return;
    };

    // Advance the game session
    session.advance(world);

    world.insert_resource(session);
}

#[derive(Resource, Default)]
struct Snapshot(pub Option<bones::World>);

fn snapshot_restore(
    mut snapshot: ResMut<Snapshot>,
    keyboard: Res<Input<KeyCode>>,
    session: Option<ResMut<Session>>,
) {
    let Some(mut session) = session else {
        return;
    };

    if keyboard.just_pressed(KeyCode::F9) {
        snapshot.0 = Some(session.snapshot());
    }

    if keyboard.just_pressed(KeyCode::F10) {
        if let Some(mut snapshot) = snapshot.0.clone() {
            session.restore(&mut snapshot);
        }
    }
}

fn play_sounds(audio: Res<Audio>, session: Option<Res<Session>>) {
    let Some(session) = session else {
        return;
    };
    // Get the sound queue out of the world
    let queue = session
        .world
        .run_initialized_system(move |mut audio_events: bones::ResMut<bones::AudioEvents>| {
            Ok(audio_events.queue.drain(..).collect::<Vec<_>>())
        })
        .unwrap();

    // Play all the sounds in the queue
    for event in queue {
        match event {
            bones::AudioEvent::PlaySound {
                sound_source,
                volume,
            } => {
                audio
                    .play(sound_source.get_bevy_handle_untyped().typed())
                    .with_volume(volume.into());
            }
        }
    }
}
