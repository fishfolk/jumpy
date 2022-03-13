use core::prelude::*;

static mut CURRENTLY_PLAYING: Option<String> = None;

pub fn start_music(id: &str) {
    stop_music();

    let mut sound = get_music(id);

    play_sound(sound, true);

    unsafe { CURRENTLY_PLAYING = Some(id.to_string()) };
}

pub fn stop_music() {
    if let Some(id) = unsafe { CURRENTLY_PLAYING.as_ref() } {
        let mut sound = get_sound(id);

        stop_sound(sound);
    }
}
