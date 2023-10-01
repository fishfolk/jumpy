use std::collections::VecDeque;

use bones_framework::prelude::kira::sound::static_sound::StaticSoundSettings;

use crate::prelude::*;

pub fn session_plugin(session: &mut Session) {
    session.add_system_to_stage(Last, play_sounds);
}

/// Resource containing the audio event queue.
#[derive(Default, HasSchema, Clone, Debug)]
pub struct AudioEvents {
    /// List of audio events that haven't been handled by the audio system yet.
    pub queue: VecDeque<AudioEvent>,
}

impl AudioEvents {
    /// Add an event to the audio event queue.
    pub fn send(&mut self, event: AudioEvent) {
        self.queue.push_back(event);
    }

    /// Play a sound.
    ///
    /// Shortcut for sending an [`AudioEvent`] with [`send()`][Self::send].
    pub fn play(&mut self, sound_source: Handle<AudioSource>, volume: f64) {
        self.queue.push_back(AudioEvent::PlaySound {
            sound_source,
            volume,
        })
    }
}

/// An audio event that may be sent to the [`AudioEvents`] resource.
#[derive(Clone, Debug)]
pub enum AudioEvent {
    /// Play a sound.
    PlaySound {
        /// The handle to the sound to play.
        sound_source: Handle<AudioSource>,
        /// The volume to play the sound at.
        volume: f64,
    },
}

fn play_sounds(
    audio: Res<AudioManager>,
    mut audio_events: ResMut<AudioEvents>,
    assets: Res<AssetServer>,
) {
    // Play all the sounds in the queue
    for event in audio_events.queue.drain(..) {
        match event {
            AudioEvent::PlaySound {
                sound_source,
                volume,
            } => {
                if let Err(e) = audio.borrow_mut().play(
                    assets
                        .get(sound_source)
                        .0
                        .with_settings(StaticSoundSettings::default().volume(volume)),
                ) {
                    warn!("Error playing sound: {e}");
                };
            }
        }
    }
}
