use std::collections::HashMap;

use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

use crate::error::Result;

pub fn vec2_is_zero(val: &Vec2) -> bool {
    *val == Vec2::ZERO
}

pub fn uvec2_is_zero(val: &UVec2) -> bool {
    *val == UVec2::ZERO
}

pub fn is_false(val: &bool) -> bool {
    !*val
}

/// This can be used to wrap types in order to make serde accept both a value and a vector of
/// values for a field, when deserializing.
#[derive(Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum OneOrMany<T: Clone> {
    One(T),
    Many(Vec<T>),
}

impl<T: Clone> OneOrMany<T> {
    pub fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(value) => vec![value],
            Self::Many(values) => values,
        }
    }
}

impl<T: Clone> From<OneOrMany<T>> for Vec<T> {
    fn from(v: OneOrMany<T>) -> Self {
        match v {
            OneOrMany::One(value) => vec![value],
            OneOrMany::Many(values) => values,
        }
    }
}

// In the future we will more than likely have to support (de)serializing of multiple formats and
// these functions will make the appropriate calls, based on extension.
// For example, the general consensus is that we should replace JSON as the primary data format but
// we will still need JSON support, to load Tiled maps, and so on...

pub fn serialize_string<T>(extension: &str, value: &T) -> Result<String>
where
    T: Serialize,
{
    assert_eq!(
        extension, "json",
        "Serialize: Invalid extension '{}'. Only json is supported for now...",
        extension
    );
    let res = serde_json::to_string_pretty(value)?;
    Ok(res)
}

pub fn serialize_bytes<T>(extension: &str, value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    assert_eq!(
        extension, "json",
        "Serialize: Invalid extension '{}'. Only json is supported for now...",
        extension
    );
    let res = serde_json::to_string_pretty(value)?;
    Ok(res.into_bytes())
}

pub fn deserialize_bytes<'a, T>(extension: &str, value: &'a [u8]) -> Result<T>
where
    T: Deserialize<'a>,
{
    assert_eq!(
        extension, "json",
        "Deserialize: Invalid extension '{}'. Only json is supported for now...",
        extension
    );
    let res: T = serde_json::from_slice(value)?;
    Ok(res)
}

pub fn deserialize_string<'a, T>(extension: &str, value: &'a str) -> Result<T>
where
    T: Deserialize<'a>,
{
    assert_eq!(
        extension, "json",
        "Deserialize: Invalid extension '{}'. Only json is supported for now...",
        extension
    );
    let res: T = serde_json::from_str(value)?;
    Ok(res)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GenericParam {
    Bool(bool),
    Int(i32),
    UInt(u32),
    Float(f32),
    String(String),
    Color(#[serde(with = "super::ColorDef")] Color),
    Vec2(#[serde(with = "super::vec2_def")] Vec2),
    IVec2(#[serde(with = "super::ivec2_def")] IVec2),
    UVec2(#[serde(with = "super::uvec2_def")] UVec2),
    Vec(Vec<Self>),
    HashMap(HashMap<String, Self>),
}

impl GenericParam {
    pub fn get_value<T: GenericParamType>(&self) -> Option<&T> {
        T::from_param(self)
    }
}

pub trait GenericParamType: Clone {
    fn from_param(param: &GenericParam) -> Option<&Self>;
}

impl GenericParamType for bool {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::Bool(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for i32 {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::Int(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for u32 {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::UInt(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for f32 {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::Float(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for String {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::String(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for Color {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::Color(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for Vec2 {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::Vec2(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for IVec2 {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::IVec2(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for UVec2 {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::UVec2(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for Vec<GenericParam> {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::Vec(value) = param {
            Some(value)
        } else {
            None
        }
    }
}

impl GenericParamType for HashMap<String, GenericParam> {
    fn from_param(param: &GenericParam) -> Option<&Self> {
        if let GenericParam::HashMap(value) = param {
            Some(value)
        } else {
            None
        }
    }
}
