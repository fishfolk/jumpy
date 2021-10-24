use serde::{Deserialize, Serialize};

mod helpers;
mod map;
mod math;

mod render;
pub use helpers::*;
pub use map::*;
pub use math::*;
pub use render::*;

use crate::error::Result;

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
