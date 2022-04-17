use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;

use serde::{Deserialize, Serialize};

use quad_snd::{AudioContext as QuadAudioContext, PlaySoundParams, Sound as QuadSound};

use crate::audio::AudioKind::Other;

use crate::file::load_file;
use crate::resources::get_sound;
use crate::{Config, Result};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AudioKind {
    SoundEffect,
    Music,
    Other(String),
}

impl Hash for AudioKind {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::SoundEffect => state.write(b"sound_effect"),
            Self::Music => state.write(b"music"),
            Self::Other(str) => str.hash(state),
        }
    }
}

impl AudioKind {
    const SOUND_EFFECT: &'static str = "sound-effect";
    const MUSIC: &'static str = "music";

    pub fn as_str(&self) -> &str {
        match self {
            Self::SoundEffect => Self::SOUND_EFFECT,
            Self::Music => Self::MUSIC,
            Other(str) => str,
        }
    }

    pub fn is_music(&self) -> bool {
        *self == AudioKind::Music
    }
}

impl Default for AudioKind {
    fn default() -> Self {
        Self::SoundEffect
    }
}

impl From<&str> for AudioKind {
    fn from(str: &str) -> Self {
        match str {
            AudioKind::SOUND_EFFECT => Self::SoundEffect,
            AudioKind::MUSIC => Self::Music,
            _ => Other(str.to_string()),
        }
    }
}

impl From<String> for AudioKind {
    fn from(str: String) -> Self {
        str.as_str().into()
    }
}

impl From<usize> for AudioKind {
    fn from(id: usize) -> Self {
        let ctx = audio_context();
        ctx.audio_kind_map
            .iter()
            .find_map(|(key, value)| {
                if *key == id {
                    Some(value.clone())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| panic!("Attempted to get audio kind from unknown id '{}'", &id))
    }
}

impl From<&AudioKind> for usize {
    fn from(kind: &AudioKind) -> Self {
        let ctx = audio_context();
        ctx.audio_kind_map
            .iter()
            .find_map(|(key, value)| if *kind == *value { Some(*key) } else { None })
            .unwrap_or_else(|| panic!("Attempted to get id of unknown audio kind '{:?}'", &kind))
    }
}

impl From<AudioKind> for usize {
    fn from(kind: AudioKind) -> Self {
        (&kind).into()
    }
}

impl ToString for AudioKind {
    fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

#[derive(Clone)]
pub struct Sound {
    id: usize,
    kind: usize,
    volume_modifier: f32,
}

impl Sound {
    pub fn kind(&self) -> &AudioKind {
        let ctx = audio_context();
        ctx.audio_kind_map.get(&self.kind).unwrap()
    }

    pub fn play(&self, should_loop: bool) {
        let ctx = audio_context();
        ctx.play(self, should_loop);
    }

    pub fn stop(&self) {
        let ctx = audio_context();
        ctx.stop(self)
    }

    pub fn set_volume_modifier(&mut self, volume: f32) {
        self.volume_modifier = volume.clamp(0.0, 1.0);
    }
}

impl PartialEq for Sound {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

fn volume_from_byte(byte: u8) -> f32 {
    byte.clamp(0, 100) as f32 / 100.0
}

fn byte_from_volume(volume: f32) -> u8 {
    (volume.clamp(0.0, 1.0) * 255.0) as u8
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    #[serde(rename = "master-volume", default = "AudioConfig::default_volume")]
    pub master_volume: u8,
    #[serde(
        rename = "sound-effect-volume",
        default = "AudioConfig::default_volume"
    )]
    pub sound_effect_volume: u8,
    #[serde(rename = "music-volume", default = "AudioConfig::default_volume")]
    pub music_volume: u8,
    #[serde(flatten)]
    pub other_volumes: HashMap<String, u8>,
}

impl AudioConfig {
    pub fn default_volume() -> u8 {
        255
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        AudioConfig {
            master_volume: 255,
            sound_effect_volume: 255,
            music_volume: 255,
            other_volumes: HashMap::new(),
        }
    }
}

struct AudioContext {
    next_id: usize,
    quad_ctx: QuadAudioContext,
    quad_sounds: HashMap<usize, QuadSound>,
    audio_kind_map: HashMap<usize, AudioKind>,
    current_music: Option<usize>,
    master_volume: f32,
    volumes: HashMap<AudioKind, f32>,
}

impl Default for AudioContext {
    fn default() -> Self {
        Self::new(&AudioConfig::default())
    }
}

impl AudioContext {
    fn new(config: &AudioConfig) -> Self {
        let mut ctx = AudioContext {
            next_id: 0,
            quad_ctx: QuadAudioContext::new(),
            quad_sounds: HashMap::new(),
            current_music: None,
            audio_kind_map: HashMap::new(),
            master_volume: 1.0,
            volumes: HashMap::new(),
        };

        ctx.apply_config(config);

        ctx
    }

    fn volume_for(&self, kind: &AudioKind) -> f32 {
        self.volumes.get(kind).copied().unwrap_or(1.0)
    }

    fn set_master_volume(&mut self, volume: f32) {
        self.master_volume = volume.clamp(0.0, 1.0);
    }

    fn set_volume_for(&mut self, kind: &AudioKind, volume: f32) {
        let current = self.volumes.get_mut(kind).unwrap();
        *current = volume.clamp(0.0, 1.0);
    }

    fn play(&mut self, sound: &Sound, should_loop: bool) {
        if sound.kind().is_music() {
            if let Some(song_id) = self.current_music.take() {
                let sound = self.quad_sounds.get_mut(&song_id).unwrap();
                sound.stop(&mut self.quad_ctx);
            }
        }

        let volume = sound.volume_modifier * self.volume_for(sound.kind()) * self.master_volume;

        let quad_sound = self.quad_sounds.get_mut(&sound.id).unwrap();
        quad_sound.play(
            &mut self.quad_ctx,
            PlaySoundParams {
                volume,
                looped: should_loop,
            },
        )
    }

    fn stop(&mut self, sound: &Sound) {
        if sound.kind().is_music() {
            if let Some(song_id) = self.current_music.take() {
                if song_id != sound.id {
                    self.current_music = Some(song_id);
                }
            }
        }

        let quad_sound = self.quad_sounds.get_mut(&sound.id).unwrap();
        quad_sound.stop(&mut self.quad_ctx);
    }

    fn stop_music(&mut self) {
        if let Some(id) = self.current_music.take() {
            let sound = self.quad_sounds.get_mut(&id).unwrap();
            sound.stop(&mut self.quad_ctx);
        }
    }

    fn apply_config(&mut self, config: &AudioConfig) {
        self.master_volume = volume_from_byte(config.master_volume);

        self.audio_kind_map.clear();
        self.volumes.clear();

        self.audio_kind_map.insert(0, AudioKind::SoundEffect);
        self.volumes.insert(
            AudioKind::SoundEffect,
            volume_from_byte(config.sound_effect_volume),
        );

        self.audio_kind_map.insert(1, AudioKind::Music);
        self.volumes
            .insert(AudioKind::Music, volume_from_byte(config.music_volume));

        let mut next_id = 2;
        for (key, &value) in &config.other_volumes {
            self.audio_kind_map
                .insert(next_id, AudioKind::Other(key.clone()));
            self.volumes
                .insert(AudioKind::Other(key.clone()), volume_from_byte(value));
            next_id += 1;
        }
    }

    fn config(&self) -> AudioConfig {
        AudioConfig {
            master_volume: byte_from_volume(self.master_volume),
            sound_effect_volume: byte_from_volume(self.volume_for(&AudioKind::SoundEffect)),
            music_volume: byte_from_volume(self.volume_for(&AudioKind::Music)),
            other_volumes: self
                .volumes
                .iter()
                .filter_map(|(k, &v)| match k {
                    AudioKind::Other(str) => Some((str.clone(), byte_from_volume(v))),
                    _ => None,
                })
                .collect(),
        }
    }
}

static mut AUDIO_CONTEXT: Option<AudioContext> = None;

fn audio_context() -> &'static mut AudioContext {
    unsafe { AUDIO_CONTEXT.get_or_insert_with(AudioContext::default) }
}

pub(crate) fn apply_audio_config(config: &AudioConfig) {
    audio_context().apply_config(config);
}

pub fn set_master_volume(volume: f32) {
    let ctx = audio_context();
    ctx.set_master_volume(volume);
}

pub fn set_volume_for(kind: &AudioKind, volume: f32) {
    let ctx = audio_context();
    ctx.set_volume_for(kind, volume);
}

pub fn play_sound(id: &str, should_loop: bool) {
    let sound = get_sound(id);
    sound.play(should_loop);
}

pub fn stop_sound(id: &str) {
    let sound = get_sound(id);
    sound.stop();
}

pub fn stop_music() {
    let ctx = audio_context();
    ctx.stop_music();
}

pub fn load_sound_bytes<K: Into<Option<AudioKind>>>(bytes: &[u8], kind: K) -> Sound {
    let ctx = audio_context();
    let quad_sound = QuadSound::load(&mut ctx.quad_ctx, bytes);

    #[cfg(target_arch = "wasm32")]
    while quad_sound.is_loaded() {
        // Wait for sound to load, if on wasm
    }

    let id = ctx.next_id;
    ctx.next_id += 1;

    ctx.quad_sounds.insert(id, quad_sound);

    let kind = kind.into().unwrap_or_default().into();

    Sound {
        id,
        kind,
        volume_modifier: 1.0,
    }
}

pub async fn load_sound_file<P: AsRef<Path>, K: Into<Option<AudioKind>>>(
    path: P,
    kind: K,
) -> Result<Sound> {
    let bytes = load_file(path).await?;
    let sound = load_sound_bytes(&bytes, kind);
    Ok(sound)
}
