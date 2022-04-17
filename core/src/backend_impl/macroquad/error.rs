use crate::error::{Error, ErrorKind};

impl From<macroquad::file::FileError> for Error {
    fn from(err: macroquad::file::FileError) -> Self {
        Error::new(ErrorKind::File, err)
    }
}

impl From<macroquad::text::FontError> for Error {
    fn from(err: macroquad::text::FontError) -> Self {
        Error::new(ErrorKind::Parsing, err)
    }
}
