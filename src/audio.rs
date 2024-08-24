use crate::prelude::*;

pub mod music;
use kira::sound::static_sound::StaticSoundSettings;
pub use music::*;

pub fn game_plugin(game: &mut Game) {
    game.init_shared_resource::<AudioCenter>();

    let session = match game.sessions.get_mut(DEFAULT_BONES_AUDIO_SESSION) {
        Some(session) => session,
        None => panic!("Audio plugin failed to find `DEFAULT_BONES_AUDIO_SESSION`, make sure jumpy audio plugin is installed after bones default plugins.")
    };

    session.stages.add_system_to_stage(First, music_system);
}

/// Extension of bones [`AudioCenter`].
pub trait AudioCenterExt {
    /// Play some music using [`StaticSoundSettings`]. These may or may not loop.
    ///
    /// `force_restart` determines if the same music is played if it should restart or not.
    fn play_music_from_settings(
        &mut self,
        sound_source: Handle<AudioSource>,
        sound_settings: StaticSoundSettings,
        force_restart: bool,
    );
}

impl AudioCenterExt for AudioCenter {
    fn play_music_from_settings(
        &mut self,
        sound_source: Handle<AudioSource>,
        sound_settings: StaticSoundSettings,
        force_restart: bool,
    ) {
        self.push_event(AudioEvent::PlayMusic {
            sound_source,
            sound_settings: Box::new(sound_settings),
            force_restart,
        });
    }
}
