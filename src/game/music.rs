use macroquad::{
    audio::{play_sound, stop_sound, PlaySoundParams, Sound},
    experimental::collections::storage,
};

use crate::Resources;

static mut CURRENTLY_PLAYING: Option<Sound> = None;

pub fn start_music(id: &str) {
    stop_music();

    let resources = storage::get::<Resources>();
    let sound = resources.music[id];

    play_sound(
        sound,
        PlaySoundParams {
            looped: true,
            volume: 0.6,
        },
    );

    unsafe { CURRENTLY_PLAYING = Some(sound) };
}

pub fn stop_music() {
    if let Some(sound) = unsafe { CURRENTLY_PLAYING } {
        stop_sound(sound);
    }
}
