//! Normalizes paths similarly to canonicalize, but without performing I/O.
//!
//! This is like Python's `os.path.normpath`.
//!
//! Initially adapted from the [`normalize_path`] crate
//!
//! [cargo-paths]: http://lib.rs/normalize_path
//!
//! # Example
//!
//! ```
//! use crate::utils::path::NormalizePath;
//!
//! assert_eq!(
//!     Path::new("/A/foo/../B/./").normalize(),
//!     Path::new("/A/B")
//! );
//! ```

use std::path::{Component, Path, PathBuf};

/// Extension trait to add `normalize_path` to std's [`Path`].
pub trait NormalizePath {
    /// Normalize a path without performing I/O.
    ///
    /// All redundant separator and up-level references are collapsed.
    ///
    /// However, this does not resolve links.
    fn normalize(&self) -> PathBuf;
}

impl NormalizePath for Path {
    fn normalize(&self) -> PathBuf {
        let mut components = self.components().peekable();
        let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek() {
            let buf = PathBuf::from(c.as_os_str());
            components.next();
            buf
        } else {
            PathBuf::new()
        };

        for component in components {
            match component {
                Component::Prefix(..) => unreachable!(),
                Component::RootDir => {
                    ret.push(component.as_os_str());
                }
                Component::CurDir => {}
                Component::ParentDir => {
                    ret.pop();
                }
                Component::Normal(c) => {
                    ret.push(c);
                }
            }
        }
        ret
    }
}
