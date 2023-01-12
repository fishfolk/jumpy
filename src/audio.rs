use std::time::Duration;

use bevy_kira_audio::{
    AudioApp, AudioChannel, AudioControl, AudioInstance, AudioSource, PlaybackState,
};
use rand::{seq::SliceRandom, thread_rng};

use crate::{metadata::GameMeta, prelude::*};

pub struct JumpyAudioPlugin;

impl Plugin for JumpyAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_kira_audio::AudioPlugin)
            .init_resource::<CurrentMusic>()
            .init_resource::<ShuffledPlaylist>()
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<EffectsChannel>()
            .add_startup_system(setup_audio_defaults)
            .add_system(music_system.run_if_resource_exists::<GameMeta>());
    }
}

#[derive(Resource)]
pub struct MusicChannel;
#[derive(Resource)]
pub struct EffectsChannel;

#[derive(Resource, Clone, Debug, Default)]
pub struct CurrentMusic {
    pub instance: Handle<AudioInstance>,
    pub idx: usize,
}

#[derive(Resource, Deref, DerefMut, Clone, Debug, Default)]
pub struct ShuffledPlaylist(pub Vec<AssetHandle<AudioSource>>);

fn setup_audio_defaults(
    music: Res<AudioChannel<MusicChannel>>,
    effects: Res<AudioChannel<EffectsChannel>>,
) {
    music.set_volume(0.12);
    effects.set_volume(0.1);
}

/// Loops through all the game music as the game is on.
fn music_system(
    game: Res<GameMeta>,
    mut playlist: ResMut<ShuffledPlaylist>,
    mut current_music: ResMut<CurrentMusic>,
    audio_instances: Res<Assets<AudioInstance>>,
    music: Res<AudioChannel<MusicChannel>>,
) {
    if playlist.is_empty() {
        let mut songs = game.playlist.clone();
        songs.shuffle(&mut thread_rng());
        **playlist = songs;
    }

    if let Some(instance) = audio_instances.get(&current_music.instance) {
        if let PlaybackState::Stopped = instance.state() {
            current_music.idx += 1;
            current_music.idx %= playlist.len();

            current_music.instance = music
                .play(playlist[current_music.idx].inner.clone_weak())
                .linear_fade_in(Duration::from_secs_f32(0.5))
                .handle();
        }
    } else if let Some(song) = playlist.get(0) {
        if current_music.instance == default() {
            current_music.instance = music
                .play(song.inner.clone_weak())
                .linear_fade_in(Duration::from_secs_f32(0.5))
                .handle();
            current_music.idx = 0;
        }
    }
}
