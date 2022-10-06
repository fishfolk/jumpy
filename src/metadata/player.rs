use std::ops::Range;

use serde::{de::SeqAccess, Deserializer};

use super::*;

#[derive(TypeUuid, Deserialize, Clone, Debug, Component)]
#[serde(deny_unknown_fields)]
#[uuid = "a939278b-901a-47d4-8ee8-6ac97881cf4d"]
pub struct PlayerMeta {
    pub name: String,
    pub spritesheet: PlayerSpritesheetMeta,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct PlayerSpritesheetMeta {
    pub image: String,
    #[serde(skip)]
    pub atlas_handle: Handle<TextureAtlas>,
    #[serde(skip)]
    pub egui_texture_id: bevy_egui::egui::TextureId,
    pub tile_size: UVec2,
    pub columns: usize,
    pub rows: usize,
    pub animation_fps: f32,
    pub animations: HashMap<String, AnimationClip>,
}

#[derive(serde::Deserialize, Clone, Debug)]
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
        let start: usize = if let Some(start) = seq.next_element()? {
            start
        } else {
            return Err(serde::de::Error::invalid_length(
                0,
                &"a sequence with a length of 2",
            ));
        };
        let end: usize = if let Some(end) = seq.next_element()? {
            end
        } else {
            return Err(serde::de::Error::invalid_length(
                1,
                &"a sequence with a length of 2",
            ));
        };

        Ok(start..end)
    }
}
