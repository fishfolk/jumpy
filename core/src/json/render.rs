use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

pub fn default_scale() -> f32 {
    1.0
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(remote = "Color")]
pub struct ColorDef {
    #[serde(rename = "red", alias = "r")]
    pub r: f32,
    #[serde(rename = "green", alias = "g")]
    pub g: f32,
    #[serde(rename = "blue", alias = "b")]
    pub b: f32,
    #[serde(rename = "alpha", alias = "a")]
    pub a: f32,
}

impl From<Color> for ColorDef {
    fn from(other: Color) -> Self {
        ColorDef {
            r: other.r,
            g: other.g,
            b: other.b,
            a: other.a,
        }
    }
}

impl From<ColorDef> for Color {
    fn from(other: ColorDef) -> Self {
        Color {
            r: other.r,
            g: other.g,
            b: other.b,
            a: other.a,
        }
    }
}

pub mod color_opt {
    use super::Color;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<Color>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "super::ColorDef")] &'a Color);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "super::ColorDef")] Color);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(remote = "FilterMode")]
pub enum FilterModeDef {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "nearest_neighbor")]
    Nearest,
}

pub fn default_filter_mode() -> FilterMode {
    FilterMode::Nearest
}
