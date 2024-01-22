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
        .add_system_to_stage(First, process_audio_events);
}

#[derive(HasSchema)]
#[schema(no_clone)]
pub struct AudioCenter {
    events: VecDeque<AudioEvent>,
    music: Option<StaticSoundHandle>,
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
    pub fn event(&mut self, event: AudioEvent) {
        self.events.push_back(event);
    }

    pub fn music_state(&self) -> Option<PlaybackState> {
        self.music.as_ref().map(StaticSoundHandle::state)
    }

    pub fn play_sound(&mut self, sound_source: Handle<AudioSource>, volume: f64) {
        self.events.push_back(AudioEvent::PlaySound {
            sound_source,
            volume,
        })
    }

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

/// An audio event that may be sent to the [`AudioEvents`] resource.
#[derive(Clone, Debug)]
pub enum AudioEvent {
    MainVolumeChange(f64),
    PlayMusic {
        sound_source: Handle<AudioSource>,
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
    handle: StaticSoundHandle,
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
    for event in audio_center.events.drain(..).collect::<Vec<_>>() {
        match event {
            AudioEvent::MainVolumeChange(main_volume) => {
                let tween = Tween::default();
                if let Some(music) = &mut audio_center.music {
                    if let Err(err) = music.set_volume(main_volume * MUSIC_VOLUME, tween) {
                        warn!("Error setting music volume: {err}");
                    }
                }
                for audio in audios.iter_mut() {
                    if let Err(err) = audio.handle.set_volume(main_volume * audio.volume, tween) {
                        warn!("Error setting audio volume: {err}");
                    }
                }
            }
            AudioEvent::PlayMusic {
                sound_source,
                mut sound_settings,
            } => {
                if let Some(mut music) = audio_center.music.take() {
                    let tween = Tween {
                        start_time: kira::StartTime::Immediate,
                        duration: MUSIC_FADE_DURATION,
                        easing: tween::Easing::Linear,
                    };
                    music.stop(tween).unwrap();
                }
                let settings = storage.get::<Settings>().unwrap();
                let volume = match sound_settings.volume {
                    tween::Value::Fixed(vol) => settings.main_volume * vol.as_amplitude(),
                    _ => settings.main_volume * MUSIC_VOLUME,
                };
                sound_settings.volume = tween::Value::Fixed(Volume::Amplitude(volume));
                let sound_data = assets.get(sound_source).with_settings(*sound_settings);
                match audio_manager.play(sound_data) {
                    Err(err) => warn!("Error playing music: {err}"),
                    Ok(handle) => audio_center.music = Some(handle),
                }
            }
            AudioEvent::PlaySound {
                sound_source,
                volume,
            } => {
                let sound_data = assets
                    .get(sound_source)
                    .with_settings(StaticSoundSettings::default().volume(volume));
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
