use super::*;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Copy, Debug)]
pub struct ColorMeta(pub [f32; 4]);
impl bones_bevy_asset::BonesBevyAssetLoad for ColorMeta {}

impl Default for ColorMeta {
    fn default() -> Self {
        Self([0.0, 0.0, 0.0, 1.0])
    }
}

impl<'de> Deserialize<'de> for ColorMeta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }
}

struct ColorVisitor;
impl<'de> serde::de::Visitor<'de> for ColorVisitor {
    type Value = ColorMeta;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("A hex-encoded RGB or RGBA color")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(ColorMeta(
            csscolorparser::parse(v)
                .map(|x| {
                    let [r, g, b, a] = x.to_array();
                    [r as f32, g as f32, b as f32, a as f32]
                })
                .map_err(|e| E::custom(e))?,
        ))
    }
}
