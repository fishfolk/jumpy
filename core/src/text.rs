use macroquad::prelude::*;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_json", serde(rename_all = "snake_case"))]
pub enum HorizontalAlignment {
    Left,
    Right,
    Center,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_json", serde(rename_all = "snake_case"))]
pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
}

pub fn draw_aligned_text(
    text: &str,
    position: Vec2,
    ha: HorizontalAlignment,
    va: VerticalAlignment,
    params: TextParams,
) {
    let measure = measure_text(text, Some(params.font), params.font_size, params.font_scale);

    let x = match ha {
        HorizontalAlignment::Left => position.x,
        _ => {
            if ha == HorizontalAlignment::Center {
                position.x - (measure.width / 2.0)
            } else {
                position.x - measure.width
            }
        }
    };

    let y = match va {
        VerticalAlignment::Top => position.y + measure.height,
        VerticalAlignment::Center => position.y + measure.height / 2.0,
        _ => position.y,
    };

    draw_text_ex(text, x, y, params);
}

/// This is used to implement `ToString` for non-crate types.
/// It is mainly used for types like `Path`, to eliminate the extra steps introduced by the
/// `to_string_lossy` method, as we are not that concerned with correctness in these settings.
pub trait ToStringHelper {
    fn to_string_helper(&self) -> String;
}

impl ToString for dyn ToStringHelper {
    fn to_string(&self) -> String {
        self.to_string_helper()
    }
}

impl ToStringHelper for Path {
    fn to_string_helper(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}

impl ToStringHelper for PathBuf {
    fn to_string_helper(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}

impl ToStringHelper for OsStr {
    fn to_string_helper(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}

impl ToStringHelper for OsString {
    fn to_string_helper(&self) -> String {
        self.to_string_lossy().into_owned()
    }
}
