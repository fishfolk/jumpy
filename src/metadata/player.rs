use std::ops::Range;

use bevy::reflect::FromReflect;
use serde::{de::SeqAccess, Deserializer};

use crate::animation::{AnimatedSprite, AnimationBank, AnimationBankSprite};

use super::*;

pub struct PlayerMetadataPlugin;

impl Plugin for PlayerMetadataPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<PlayerMeta>();
    }
}

#[derive(Reflect, TypeUuid, Deserialize, Clone, Debug, Component, Default)]
#[serde(deny_unknown_fields)]
#[uuid = "a939278b-901a-47d4-8ee8-6ac97881cf4d"]
pub struct PlayerMeta {
    pub name: String,
    pub spritesheet: PlayerSpritesheetMeta,
    pub sounds: PlayerSounds,
    pub movement: PlayerMovementMeta,
}

#[derive(Reflect, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerMovementMeta {
    pub slow_fall_speed: f32,
    pub air_move_speed: f32,
}

#[derive(Reflect, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerSpritesheetMeta {
    pub image: String,
    #[serde(skip)]
    pub atlas_handle: AssetHandle<TextureAtlas>,
    #[serde(skip)]
    #[reflect(ignore)]
    pub egui_texture_id: bevy_egui::egui::TextureId,
    pub tile_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub animation_fps: f32,
    pub animations: HashMap<String, AnimationClip>,
    #[serde(default)]
    pub decorations: Vec<String>,
    #[serde(skip)]
    pub decoration_handles: Vec<Handle<TextureAtlas>>,
}

#[derive(Reflect, Deserialize, Clone, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct PlayerSounds {
    pub land: String,
    pub land_volume: f32,
    #[serde(skip)]
    pub land_handle: Handle<AudioSource>,

    pub jump: String,
    pub jump_volume: f32,
    #[serde(skip)]
    pub jump_handle: Handle<AudioSource>,

    pub grab: String,
    pub grab_volume: f32,
    #[serde(skip)]
    pub grab_handle: Handle<AudioSource>,

    pub drop: String,
    pub drop_volume: f32,
    #[serde(skip)]
    pub drop_handle: Handle<AudioSource>,
}

impl PlayerSpritesheetMeta {
    pub fn get_animation_bank_and_sprite(&self) -> (AnimationBank, AnimationBankSprite) {
        let animations = self
            .animations
            .clone()
            .into_iter()
            .map(|(name, clip)| {
                (
                    name,
                    AnimatedSprite {
                        start: clip.frames.start,
                        end: clip.frames.end,
                        atlas: self.atlas_handle.inner.clone_weak(),
                        stacked_atlases: self.decoration_handles.clone(),
                        flip_x: false,
                        flip_y: false,
                        repeat: clip.repeat,
                        fps: self.animation_fps,
                        timer: 0.0,
                        index: 0,
                    },
                )
            })
            .collect();

        (
            AnimationBank {
                animations,
                last_animation: default(),
            },
            AnimationBankSprite {
                current_animation: self.animations.keys().next().cloned().unwrap_or_default(),
                flip_x: false,
                flip_y: false,
            },
        )
    }
}

#[derive(Reflect, FromReflect, serde::Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct AnimationClip {
    #[serde(deserialize_with = "deserialize_range_from_array")]
    pub frames: Range<usize>,
    #[serde(default)]
    pub repeat: bool,
}

fn deserialize_range_from_array<'de, D>(de: D) -> Result<Range<usize>, D::Error>
where
    D: Deserializer<'de>,
{
    de.deserialize_tuple(2, RangeVisitor)
}

struct RangeVisitor;

impl<'de> serde::de::Visitor<'de> for RangeVisitor {
    type Value = Range<usize>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A sequence of 2 integers")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let Some(start) = seq.next_element()? else {
            return Err(serde::de::Error::invalid_length(
                0,
                &"a sequence with a length of 2",
            ));
        };
        let Some(end) = seq.next_element()? else {
            return Err(serde::de::Error::invalid_length(
                1,
                &"a sequence with a length of 2",
            ));
        };

        Ok(start..end)
    }
}
