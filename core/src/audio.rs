use std::path::Path;

pub use quad_snd::*;

use crate::Result;
use crate::file::load_file;

static mut AUDIO_CONTEXT: Option<AudioContext> = None;

unsafe fn get_audio_context() -> &'static mut AudioContext {
    AUDIO_CONTEXT.get_or_insert_with(AudioContext::new)
}

pub fn play_sound(sound: &mut Sound, should_loop: bool) {
    let ctx = unsafe { get_audio_context() };
    sound.play(ctx, PlaySoundParams {
        volume: 1.0,
        looped: should_loop,
    })
}

pub fn stop_sound(sound: &mut Sound) {
    let ctx = unsafe { get_audio_context() };
    sound.stop(ctx)
}

pub fn load_sound_bytes(bytes: &[u8]) -> Sound {
    let ctx = unsafe { get_audio_context() };
    Sound::load(ctx, bytes)
}

pub async fn load_sound_file<P: AsRef<Path>>(path: P) -> Result<Sound> {
    let bytes = load_file(path).await?;
    let sound = load_sound_bytes(&bytes);
    Ok(sound)
}