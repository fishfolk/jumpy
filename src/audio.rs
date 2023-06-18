//! Audio & music plugin.
//!
//! Audio playback in Jumpy is powered by [`bevy_kira_audio`]. This module sets up the audio plugin,
//! and installs our [`MusicChannel`] and [`EffectsChannel`] for playing music and sound effects.
//!
//! Also in this module is the [`music_system`] which handles playing the right music in different
//! game states.
//!
//! Game sounds are _not_ handled here. Game sounds events are created in
//! [`jumpy_core`][::jumpy_core] and then processed and sent to the effects channel by the
//! [`play_sounds()`][crate::session::play_sounds] system.
//!
//! [`bevy_kira_audio`]: https://docs.rs/bevy_kira_audio

use std::time::Duration;

use bevy_kira_audio::{
    AudioApp, AudioChannel, AudioControl, AudioInstance, AudioSource, PlaybackState,
};
use rand::{seq::SliceRandom, thread_rng};

use crate::{main_menu::MenuPage, metadata::GameMeta, prelude::*};

/// Audio & Music plugin.
pub struct JumpyAudioPlugin;

impl Plugin for JumpyAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy_kira_audio::AudioPlugin)
            .init_resource::<MusicState>()
            .init_resource::<ShuffledPlaylist>()
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<EffectsChannel>()
            .add_startup_system(setup_audio_defaults)
            .add_system(music_system.run_if(resource_exists::<GameMeta>()));
    }
}

/// Marker struct for the music audio channel.
#[derive(Resource)]
pub struct MusicChannel;

/// Marker struct for the effects audio channel.
#[derive(Resource)]
pub struct EffectsChannel;

/// The music playback state.
#[derive(Resource, Clone, Debug, Default)]
pub enum MusicState {
    /// Music is not playing.
    #[default]
    None,
    /// Playing the main menu music.
    MainMenu(Handle<AudioInstance>),
    /// Playing the character select music.
    CharacterSelect(Handle<AudioInstance>),
    /// Playing the credits music.
    Credits(Handle<AudioInstance>),
    /// Playing the fight music.
    Fight {
        /// The handle to the audio instance.
        instance: Handle<AudioInstance>,
        /// The index of the song in the shuffled playlist.
        idx: usize,
    },
}

impl MusicState {
    /// Get the current audio instance, if one is contained.
    fn current_instance(&self) -> Option<&Handle<AudioInstance>> {
        match self {
            MusicState::None => None,
            MusicState::MainMenu(i) => Some(i),
            MusicState::CharacterSelect(i) => Some(i),
            MusicState::Credits(i) => Some(i),
            MusicState::Fight { instance, .. } => Some(instance),
        }
    }
}

/// Bevy resource containing the in-game music playlist shuffled.
#[derive(Resource, Deref, DerefMut, Clone, Debug, Default)]
pub struct ShuffledPlaylist(pub Vec<AssetHandle<AudioSource>>);

/// Sets the default music and effects volume.
///
/// TODO: make this configurable in the settings menu.
fn setup_audio_defaults(
    music: Res<AudioChannel<MusicChannel>>,
    effects: Res<AudioChannel<EffectsChannel>>,
) {
    music.set_volume(0.22);
    effects.set_volume(0.1);
}

/// The amount of time to spend fading the music in and out.
const MUSIC_FADE_DURATION: Duration = Duration::from_millis(500);

/// System that plays music according to the game mode.
fn music_system(
    game: Res<GameMeta>,
    mut shuffled_fight_music: ResMut<ShuffledPlaylist>,
    mut music_state: ResMut<MusicState>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
    music: Res<AudioChannel<MusicChannel>>,
    engine_state: Res<State<EngineState>>,
    menu_page: Res<MenuPage>,
) {
    if shuffled_fight_music.is_empty() || engine_state.is_changed() {
        let mut songs = game.music.fight.clone();
        songs.shuffle(&mut thread_rng());
        **shuffled_fight_music = songs;
    }

    match engine_state.0 {
        EngineState::LoadingPlatformStorage | EngineState::LoadingGameData => (),
        EngineState::InGame => {
            if let MusicState::Fight { instance, idx } = &mut *music_state {
                let inst = audio_instances.get(instance).unwrap();
                if let PlaybackState::Stopped = inst.state() {
                    *idx += 1;
                    *idx %= shuffled_fight_music.len();

                    *instance = music
                        .play(shuffled_fight_music[*idx].inner.clone_weak())
                        .linear_fade_in(MUSIC_FADE_DURATION)
                        .handle();
                }
            } else {
                if let Some(instance) = music_state.current_instance() {
                    let instance = audio_instances.get_mut(instance).unwrap();
                    instance.stop(AudioTween::linear(MUSIC_FADE_DURATION));
                }

                if let Some(song) = shuffled_fight_music.get(0) {
                    *music_state = MusicState::Fight {
                        instance: music
                            .play(song.inner.clone_weak())
                            .linear_fade_in(MUSIC_FADE_DURATION)
                            .looped()
                            .handle(),
                        idx: 0,
                    };
                }
            }
        }
        EngineState::MainMenu => match &*menu_page {
            MenuPage::PlayerSelect | MenuPage::MapSelect { .. } | MenuPage::NetworkGame => {
                if !matches!(*music_state, MusicState::CharacterSelect(..)) {
                    if let Some(instance) = music_state.current_instance() {
                        let instance = audio_instances.get_mut(instance).unwrap();
                        instance.stop(AudioTween::linear(MUSIC_FADE_DURATION));
                    }
                    *music_state = MusicState::CharacterSelect(
                        music
                            .play(game.music.character_screen.inner.clone_weak())
                            .linear_fade_in(MUSIC_FADE_DURATION)
                            .looped()
                            .handle(),
                    );
                }
            }
            MenuPage::Home | MenuPage::Settings => {
                if !matches!(*music_state, MusicState::MainMenu(..)) {
                    if let Some(instance) = music_state.current_instance() {
                        let instance = audio_instances.get_mut(instance).unwrap();
                        instance.stop(AudioTween::linear(MUSIC_FADE_DURATION));
                    }
                    *music_state = MusicState::MainMenu(
                        music
                            .play(game.music.title_screen.inner.clone_weak())
                            .linear_fade_in(MUSIC_FADE_DURATION)
                            .looped()
                            .handle(),
                    );
                }
            }
            MenuPage::Credits => {
                if !matches!(*music_state, MusicState::Credits(..)) {
                    if let Some(instance) = music_state.current_instance() {
                        let instance = audio_instances.get_mut(instance).unwrap();
                        instance.stop(AudioTween::linear(MUSIC_FADE_DURATION));
                    }
                    *music_state = MusicState::Credits(
                        music
                            .play(game.music.credits.inner.clone_weak())
                            .linear_fade_in(MUSIC_FADE_DURATION)
                            .looped()
                            .handle(),
                    );
                }
            }
        },
    }
}
