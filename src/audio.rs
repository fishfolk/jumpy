use std::collections::VecDeque;

use bones_framework::prelude::kira::{
    sound::{
        static_sound::{StaticSoundHandle, StaticSoundSettings},
        PlaybackState,
    },
    tween::{self, Tween},
    Volume,
};

use crate::prelude::*;

pub mod music;

pub use music::*;

pub fn game_plugin(game: &mut Game) {
    game.init_shared_resource::<AudioCenter>();

    let session = game.sessions.create(SessionNames::AUDIO);

    // Audio doesn't do any rendering
    session.visible = false;
    session
        .stages
        .add_system_to_stage(First, music_system)
        .add_system_to_stage(First, process_audio_events)
        .add_system_to_stage(Last, kill_finished_audios);
}

/// A resource that can be used to control game audios.
#[derive(HasSchema)]
#[schema(no_clone)]
pub struct AudioCenter {
    /// Buffer for audio events that have not yet been processed.
    events: VecDeque<AudioEvent>,
    /// The handle to the current music.
    music: Option<Audio>,
}

impl Default for AudioCenter {
    fn default() -> Self {
        Self {
            events: VecDeque::with_capacity(16),
            music: None,
        }
    }
}

impl AudioCenter {
    /// Push an audio event to the queue for later processing.
    pub fn event(&mut self, event: AudioEvent) {
        self.events.push_back(event);
    }

    /// Get the playback state of the music.
    pub fn music_state(&self) -> Option<PlaybackState> {
        self.music.as_ref().map(|m| m.handle.state())
    }

    /// Play a sound. These are usually short audios that indicate something
    /// happened in game, e.g. a player jump, an explosion, etc.
    pub fn play_sound(&mut self, sound_source: Handle<AudioSource>, volume: f64) {
        self.events.push_back(AudioEvent::PlaySound {
            sound_source,
            volume,
        })
    }

    /// Play some music. These may or may not loop.
    ///
    /// Any current music is stopped.
    pub fn play_music(
        &mut self,
        sound_source: Handle<AudioSource>,
        sound_settings: StaticSoundSettings,
    ) {
        self.events.push_back(AudioEvent::PlayMusic {
            sound_source,
            sound_settings: Box::new(sound_settings),
        });
    }
}

/// An audio event that may be sent to the [`AudioCenter`] resource for
/// processing.
#[derive(Clone, Debug)]
pub enum AudioEvent {
    /// Update the volume of all audios using the new values.
    VolumeChange {
        main_volume: f64,
        music_volume: f64,
        effects_volume: f64,
    },
    /// Play some music.
    ///
    /// Any current music is stopped.
    PlayMusic {
        /// The handle for the music.
        sound_source: Handle<AudioSource>,
        /// The settings for the music.
        sound_settings: Box<StaticSoundSettings>,
    },
    /// Play a sound.
    PlaySound {
        /// The handle to the sound to play.
        sound_source: Handle<AudioSource>,
        /// The volume to play the sound at.
        volume: f64,
    },
}

#[derive(HasSchema)]
#[schema(no_clone, no_default, opaque)]
#[repr(C)]
pub struct Audio {
    /// The handle for the audio.
    handle: StaticSoundHandle,
    /// The original volume requested for the audio.
    volume: f64,
}

fn process_audio_events(
    mut audio_manager: ResMut<AudioManager>,
    mut audio_center: ResMut<AudioCenter>,
    assets: ResInit<AssetServer>,
    mut entities: ResMut<Entities>,
    mut audios: CompMut<Audio>,
    storage: Res<Storage>,
) {
    let settings = storage.get::<Settings>().unwrap();

    for event in audio_center.events.drain(..).collect::<Vec<_>>() {
        match event {
            AudioEvent::VolumeChange {
                main_volume,
                music_volume,
                effects_volume,
            } => {
                let tween = Tween::default();
                // Update music volume
                if let Some(music) = &mut audio_center.music {
                    let volume = main_volume * music_volume * music.volume;
                    if let Err(err) = music.handle.set_volume(volume, tween) {
                        warn!("Error setting music volume: {err}");
                    }
                }
                // Update sound volumes
                for audio in audios.iter_mut() {
                    let volume = main_volume * effects_volume * audio.volume;
                    if let Err(err) = audio.handle.set_volume(volume, tween) {
                        warn!("Error setting audio volume: {err}");
                    }
                }
            }
            AudioEvent::PlayMusic {
                sound_source,
                mut sound_settings,
            } => {
                // Stop the current music
                if let Some(mut music) = audio_center.music.take() {
                    let tween = Tween {
                        start_time: kira::StartTime::Immediate,
                        duration: MUSIC_FADE_DURATION,
                        easing: tween::Easing::Linear,
                    };
                    music.handle.stop(tween).unwrap();
                }
                // Scale the requested volume by the settings value
                let volume = match sound_settings.volume {
                    tween::Value::Fixed(vol) => vol.as_amplitude(),
                    _ => MUSIC_VOLUME,
                };
                let scaled_volume = settings.main_volume * settings.music_volume * volume;
                sound_settings.volume = tween::Value::Fixed(Volume::Amplitude(scaled_volume));
                // Play the new music
                let sound_data = assets.get(sound_source).with_settings(*sound_settings);
                match audio_manager.play(sound_data) {
                    Err(err) => warn!("Error playing music: {err}"),
                    Ok(handle) => audio_center.music = Some(Audio { handle, volume }),
                }
            }
            AudioEvent::PlaySound {
                sound_source,
                volume,
            } => {
                let scaled_volume = settings.main_volume * settings.effects_volume * volume;
                let sound_data = assets
                    .get(sound_source)
                    .with_settings(StaticSoundSettings::default().volume(scaled_volume));
                match audio_manager.play(sound_data) {
                    Err(err) => warn!("Error playing sound: {err}"),
                    Ok(handle) => {
                        let audio_ent = entities.create();
                        audios.insert(audio_ent, Audio { handle, volume });
                    }
                }
            }
        }
    }
}

fn kill_finished_audios(entities: Res<Entities>, audios: Comp<Audio>, mut commands: Commands) {
    for (audio_ent, audio) in entities.iter_with(&audios) {
        if audio.handle.state() == PlaybackState::Stopped {
            commands.add(move |mut entities: ResMut<Entities>| entities.kill(audio_ent));
        }
    }
}
