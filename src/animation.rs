use std::ops::Range;

use serde::{de::SeqAccess, Deserializer};

use crate::prelude::*;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, _app: &mut App) {}
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct Clip {
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
