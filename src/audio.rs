use crate::prelude::*;

pub mod music;
use kira::sound::static_sound::StaticSoundSettings;
pub use music::*;

pub fn game_plugin(game: &mut Game) {
    game.init_shared_resource::<AudioCenter>();

    let modified_session = game.sessions.modify_and_replace_existing_session(
        SessionNames::AUDIO,
        |session: &mut SessionBuilder| {
            session.stages().add_system_to_stage(First, music_system);
        },
    );

    if modified_session.is_none() {
        panic!("Audio plugin failed to find existing bones audio session, make sure jumpy audio plugin is installed after bones default plugins.")
    }
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
