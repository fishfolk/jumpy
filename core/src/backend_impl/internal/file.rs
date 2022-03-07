use std::fs;
use std::path::Path;

use crate::file::Error;

#[cfg(target_arch = "wasm32")]
pub async fn load_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    unimplemented!("WASM file handling is not implemented")
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn load_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    match fs::read(&path) {
        Err(err) => Err(Error::new(path, err).into()),
        Ok(res) => Ok(res),
    }
}