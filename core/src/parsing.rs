use std::fmt::{self, Debug, Formatter};
use std::path::Path;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::file::{load_file, Error};
use crate::text::ToStringHelper;
use crate::Result;

/// Serialize a value into a string of JSON.
/// Will return a `serde_json::Error` if a parsing error is encountered.
pub fn serialize_json_string<T>(value: &T) -> std::result::Result<String, serde_json::Error>
where
    T: Serialize,
{
    let res = serde_json::to_string_pretty(value)?;
    Ok(res)
}

/// Serialize a value into a slice of JSON.
/// Will return a `serde_json::Error` if a parsing error is encountered.
pub fn serialize_json_bytes<T>(value: &T) -> std::result::Result<Vec<u8>, serde_json::Error>
where
    T: Serialize,
{
    let res = serde_json::to_string_pretty(value)?;
    Ok(res.into_bytes())
}

/// Deserialize a slice of JSON into a value.
/// Will return a `serde_json::Error` if a parsing error is encountered.
pub fn deserialize_json_bytes<'a, T>(value: &'a [u8]) -> std::result::Result<T, serde_json::Error>
where
    T: Deserialize<'a>,
{
    let res = serde_json::from_slice(value)?;
    Ok(res)
}

/// Deserialize a string of JSON into a value.
/// Will return a `serde_json::Error` if a parsing error is encountered.
pub fn deserialize_json_string<'a, T>(value: &'a str) -> std::result::Result<T, serde_json::Error>
where
    T: Deserialize<'a>,
{
    let res = serde_json::from_str(value)?;
    Ok(res)
}

/// Load and deserialize a JSON file into a value
pub async fn load_json_file<T, P: AsRef<Path>>(path: P) -> Result<T>
where
    T: DeserializeOwned,
{
    let bytes = load_file(&path).await?;
    match serde_json::from_slice(&bytes) {
        Err(err) => Err(Error::new(path, err).into()),
        Ok(res) => Ok(res),
    }
}

/// Serialize a value into a string of TOML.
/// Will return a `toml::ser::Error` if a parsing error is encountered.
pub fn serialize_toml_string<T>(value: &T) -> std::result::Result<String, toml::ser::Error>
where
    T: Serialize,
{
    let res = toml::to_string_pretty(value)?;
    Ok(res)
}

/// Serialize a value into a slice of TOML.
/// Will return a `toml::ser::Error` if a parsing error is encountered.
pub fn serialize_toml_bytes<T>(value: &T) -> std::result::Result<Vec<u8>, toml::ser::Error>
where
    T: Serialize,
{
    let res = toml::to_string_pretty(value)?;
    Ok(res.into_bytes())
}

/// Deserialize a slice of TOML into a value.
/// Will return a `toml::de::Error` if a parsing error is encountered.
pub fn deserialize_toml_bytes<'a, T>(value: &'a [u8]) -> std::result::Result<T, toml::de::Error>
where
    T: Deserialize<'a>,
{
    let res = toml::from_slice(value)?;
    Ok(res)
}

/// Deserialize a string of TOML into a value.
/// Will return a `toml::de::Error` if a parsing error is encountered.
pub fn deserialize_toml_string<'a, T>(value: &'a str) -> std::result::Result<T, toml::de::Error>
where
    T: Deserialize<'a>,
{
    let res = toml::from_str(value)?;
    Ok(res)
}

/// Load and deserialize a TOML file into a value
pub async fn load_toml_file<T, P: AsRef<Path>>(path: P) -> Result<T>
where
    T: DeserializeOwned,
{
    let bytes = load_file(&path).await?;
    match toml::from_slice(&bytes) {
        Err(err) => Err(Error::new(path, err).into()),
        Ok(res) => Ok(res),
    }
}
