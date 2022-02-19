use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

pub mod vec2_def {
    use super::{vec2, Vec2};
    use serde::{
        de::{self, MapAccess, Visitor},
        ser::SerializeStruct,
        Deserialize, Deserializer, Serializer,
    };
    use std::fmt;

    pub fn serialize<S>(value: &Vec2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(stringify!(Vec2), 2)?;
        state.serialize_field("x", &value.x)?;
        state.serialize_field("y", &value.y)?;
        state.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec2, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            X,
            Y,
        }

        struct Vec2Visitor;

        impl<'de> Visitor<'de> for Vec2Visitor {
            type Value = Vec2;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!("struct ", stringify!(Vec2)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut x = None;
                let mut y = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::X => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::Y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                    }
                }

                let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y = y.ok_or_else(|| de::Error::missing_field("y"))?;

                Ok(vec2(x, y))
            }
        }

        deserializer.deserialize_struct(stringify!(Vec2), &["x", "y"], Vec2Visitor)
    }
}

pub mod uvec2_def {
    use super::{uvec2, UVec2};
    use serde::de::MapAccess;
    use serde::{
        de::{self, Visitor},
        ser::SerializeStruct,
        Deserialize, Deserializer, Serializer,
    };

    use std::fmt;

    pub fn serialize<S>(value: &UVec2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(stringify!(UVec2), 2)?;
        state.serialize_field("x", &value.x)?;
        state.serialize_field("y", &value.y)?;
        state.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<UVec2, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            X,
            Y,
        }

        struct UVec2Visitor;

        impl<'de> Visitor<'de> for UVec2Visitor {
            type Value = UVec2;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!("struct ", stringify!(UVec2)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut x = None;
                let mut y = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::X => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::Y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                    }
                }

                let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y = y.ok_or_else(|| de::Error::missing_field("y"))?;

                Ok(uvec2(x, y))
            }
        }

        deserializer.deserialize_struct(stringify!(UVec2), &["x", "y"], UVec2Visitor)
    }
}

pub mod ivec2_def {
    use super::{ivec2, IVec2};
    use serde::de::MapAccess;
    use serde::{
        de::{self, Visitor},
        ser::SerializeStruct,
        Deserialize, Deserializer, Serializer,
    };

    use std::fmt;

    pub fn serialize<S>(value: &IVec2, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct(stringify!(UVec2), 2)?;
        state.serialize_field("x", &value.x)?;
        state.serialize_field("y", &value.y)?;
        state.end()
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<IVec2, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(field_identifier, rename_all = "snake_case")]
        enum Field {
            X,
            Y,
        }

        struct IVec2Visitor;

        impl<'de> Visitor<'de> for IVec2Visitor {
            type Value = IVec2;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(concat!("struct ", stringify!(IVec2)))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut x = None;
                let mut y = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        Field::X => {
                            if x.is_some() {
                                return Err(de::Error::duplicate_field("x"));
                            }
                            x = Some(map.next_value()?);
                        }
                        Field::Y => {
                            if y.is_some() {
                                return Err(de::Error::duplicate_field("y"));
                            }
                            y = Some(map.next_value()?);
                        }
                    }
                }

                let x = x.ok_or_else(|| de::Error::missing_field("x"))?;
                let y = y.ok_or_else(|| de::Error::missing_field("y"))?;

                Ok(ivec2(x, y))
            }
        }

        deserializer.deserialize_struct(stringify!(IVec2), &["x", "y"], IVec2Visitor)
    }
}

pub mod vec2_opt {
    use super::Vec2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<Vec2>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "super::vec2_def")] &'a Vec2);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec2>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "super::vec2_def")] Vec2);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

pub mod vec2_vec {
    use super::Vec2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &[Vec2], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "super::vec2_def")] &'a Vec2);

        let helper: Vec<Helper> = value.iter().map(Helper).collect();
        helper.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec2>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "super::vec2_def")] Vec2);

        let helper = Vec::deserialize(deserializer)?;
        Ok(helper
            .into_iter()
            .map(|Helper(external)| external)
            .collect())
    }
}

pub mod uvec2_opt {
    use super::UVec2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<UVec2>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "super::uvec2_def")] &'a UVec2);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<UVec2>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "super::uvec2_def")] UVec2);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
    }
}

pub mod ivec2_opt {
    use super::IVec2;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<IVec2>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct Helper<'a>(#[serde(with = "super::ivec2_def")] &'a IVec2);

        value.as_ref().map(Helper).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<IVec2>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper(#[serde(with = "super::ivec2_def")] IVec2);

        let helper = Option::deserialize(deserializer)?;
        Ok(helper.map(|Helper(external)| external))
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

pub mod rect_opt {
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
