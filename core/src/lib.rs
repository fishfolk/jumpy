#[macro_use]
pub mod error;
pub mod data;
pub mod input;
pub mod json;
pub mod math;
pub mod network;
pub mod text;

pub use error::{Error, Result};

pub use async_trait::async_trait;

pub use serde;

use serde::{Deserialize, Serialize};

pub use serde_json;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Id(String);

impl Id {
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl ToString for Id {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl From<&str> for Id {
    fn from(s: &str) -> Self {
        Id(s.to_string())
    }
}

impl From<String> for Id {
    fn from(s: String) -> Self {
        Id(s)
    }
}
