use core::prelude::*;

use crate::Resources;

static mut CURRENTLY_PLAYING: Option<String> = None;

pub fn start_music(id: &str) {
    stop_music();

    let mut resources = storage::get_mut::<Resources>();
    let mut sound = resources.music.get_mut(id).unwrap();

    play_sound(sound, true);

    unsafe { CURRENTLY_PLAYING = Some(id.to_string()) };
}

pub fn stop_music() {
    if let Some(id) = unsafe { CURRENTLY_PLAYING.as_ref() } {
        let mut resources = storage::get_mut::<Resources>();
        let mut sound = resources.music.get_mut(id).unwrap();

        stop_sound(sound);
    }
}
