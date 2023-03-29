use super::*;
use serde::{ser::SerializeStruct, Deserialize, Deserializer};

#[derive(Clone, Copy, Debug, Default)]
pub struct ColorMeta(pub Color);

impl bones_bevy_asset::BonesBevyAssetLoad for ColorMeta {}

impl<'de> Deserialize<'de> for ColorMeta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(ColorVisitor)
    }
}

impl Serialize for ColorMeta {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let [r, g, b, a] = self.0.as_rgba_f32();

        let mut color = serializer.serialize_struct("Color", 4)?;
        color.serialize_field("r", &r)?;
        color.serialize_field("g", &g)?;
        color.serialize_field("b", &b)?;
        color.serialize_field("a", &a)?;
        color.end()
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
                    [r as f32, g as f32, b as f32, a as f32].into()
                })
                .map_err(|e| E::custom(e))?,
        ))
    }
}
