use std::path::Path;

use crate::Result;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sound(usize);

pub async fn load_sound<P: AsRef<Path>>(path: P) -> Result<Sound> {
    unimplemented!("Sound loading is not implemented")
}