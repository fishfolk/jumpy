use macroquad::{audio::play_sound, prelude::collections::storage};

use crate::Resources;

/// This is a stand-in until we have volume settings
pub const SOUND_EFFECT_VOLUME: f32 = 0.4;

pub fn play_sound_effect(sound_id: &str, volume_multiplier: f32) {
    let resources = storage::get::<Resources>();
    let sound = resources.sounds[sound_id];
    play_sound(
        sound,
        macroquad::audio::PlaySoundParams {
            looped: false,
            volume: SOUND_EFFECT_VOLUME * volume_multiplier,
        },
    );
}
