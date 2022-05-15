use std::path::Path;

use crate::file::Error;

pub async fn read_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<u8>, Error> {
    let path = path.as_ref().to_string_lossy().to_string();

    match macroquad::file::load_file(&path).await {
        Err(err) => Err(Error::new(&path, err)),
        Ok(res) => Ok(res),
    }
}
