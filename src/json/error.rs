use std::fmt;
use std::fmt::{Debug, Formatter};

pub struct Error {
    pub path: String,
    pub error: serde_json::Error,
}

impl Error {
    pub fn new(path: &str, err: serde_json::Error) -> Self {
        Error {
            path: path.to_string(),
            error: err,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", &self.path, &self.error.to_string())
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", &self.path, &self.error.to_string())
    }
}

impl std::error::Error for Error {}