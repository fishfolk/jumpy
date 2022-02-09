use std::collections::HashMap;

use macroquad::prelude::*;

use serde::{Deserialize, Serialize};

pub fn default_true() -> bool {
    true
}

pub fn is_true(val: &bool) -> bool {
    *val
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

impl<T: Clone> Default for OneOrMany<T> {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
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

/// This is used to allow values of different types for the same field in a JSON object.
/// When an enum with only tuple-like variants, all with a single member each, is marked as untagged,
/// serde will return the appropriate enum variant, depending on the type of the JSON value.
/// Furthermore, you can use `GenericParam::Vec` and `GenericParam::HashMap` to allow members of
/// different types in the same collection, in your JSON objects.
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

pub trait BoolHelpers {
    fn is_true(&self) -> bool;
    fn is_false(&self) -> bool;
}

impl BoolHelpers for bool {
    fn is_true(&self) -> bool {
        *self
    }

    fn is_false(&self) -> bool {
        !*self
    }
}
