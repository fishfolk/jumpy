use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

pub use crate::math::URect;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Vec2Def {
    pub x: f32,
    pub y: f32,
}

impl Vec2Def {
    pub fn new(x: f32, y: f32) -> Self {
        Vec2Def { x, y }
    }
}

impl Default for Vec2Def {
    fn default() -> Self {
        Vec2Def { x: 0.0, y: 0.0 }
    }
}

impl From<Vec2> for Vec2Def {
    fn from(other: Vec2) -> Self {
        Vec2Def {
            x: other.x,
            y: other.y,
        }
    }
}

impl From<Vec2Def> for Vec2 {
    fn from(other: Vec2Def) -> Self {
        vec2(other.x, other.y)
    }
}

#[derive(Debug, Default, Copy, Clone, Serialize, Deserialize)]
pub struct UVec2Def {
    pub x: u32,
    pub y: u32,
}

impl UVec2Def {
    pub fn new(x: u32, y: u32) -> Self {
        UVec2Def { x, y }
    }
}

impl From<UVec2> for UVec2Def {
    fn from(other: UVec2) -> Self {
        UVec2Def {
            x: other.x,
            y: other.y,
        }
    }
}

impl From<UVec2Def> for UVec2 {
    fn from(other: UVec2Def) -> Self {
        uvec2(other.x, other.y)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize)]
#[serde(remote = "Rect")]
pub struct RectDef {
    x: f32,
    y: f32,
    #[serde(rename = "width", alias = "w")]
    w: f32,
    #[serde(rename = "height", alias = "h")]
    h: f32,
}

impl From<Rect> for RectDef {
    fn from(other: Rect) -> Self {
        RectDef {
            x: other.x,
            y: other.y,
            w: other.w,
            h: other.h,
        }
    }
}

impl From<RectDef> for Rect {
    fn from(other: RectDef) -> Self {
        Rect {
            x: other.x,
            y: other.y,
            w: other.w,
            h: other.h,
        }
    }
}

pub mod def_vec2 {
    use super::{Vec2, Vec2Def};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Vec2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let helper = Vec2Def::from(*value);
        Vec2Def::serialize(&helper, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = Vec2Def::deserialize(deserializer)?;
        Ok(Vec2::from(helper))
    }
}

pub mod opt_vec2 {
    use super::Vec2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<Vec2>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "super::def_vec2")] &'a Vec2);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec2>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "super::def_vec2")] Vec2);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

pub mod vec_vec2 {
    use super::Vec2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &[Vec2], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "super::def_vec2")] &'a Vec2);

        let helper: Vec<Helper> = value.iter().map(Helper).collect();
        helper.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec2>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "super::def_vec2")] Vec2);

        let helper = Vec::deserialize(deserializer)?;
        Ok(helper
            .into_iter()
            .map(|Helper(external)| external)
            .collect())
    }
}

pub mod def_uvec2 {
    use super::{UVec2, UVec2Def};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &UVec2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let helper = UVec2Def::from(*value);
        UVec2Def::serialize(&helper, serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<UVec2, D::Error>
    where
        D: Deserializer<'de>,
    {
        let helper = UVec2Def::deserialize(deserializer)?;
        Ok(UVec2::from(helper))
    }
}

pub mod opt_uvec2 {
    use super::UVec2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<UVec2>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(with = "super::def_uvec2")]
            &'a UVec2,
        );

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<UVec2>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(
            #[serde(skip_serializing_if = "Option::is_none")]
            #[serde(with = "super::def_uvec2")]
            UVec2,
        );

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

pub mod opt_rect {
    use super::{Rect, RectDef};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<Rect>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "RectDef")] &'a Rect);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Rect>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "RectDef")] Rect);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}
