use std::fmt;
use std::fmt::{Debug, Formatter};
use std::path::Path;

pub use crate::backend_impl::file::*;

pub struct Error {
    pub path: String,
    pub err: Box<dyn std::error::Error + Send + Sync + 'static>,
}

impl Error {
    pub fn new<P: AsRef<Path>, E>(path: P, err: E) -> Self
        where
            E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        let path = path.as_ref();

        Error {
            path: path.to_string_lossy().to_string(),
            err: err.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "File error: {}: {}", &self.path, &self.err)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "File error: {}: {}", &self.path, &self.err)
    }
}

impl std::error::Error for Error {}