use std::fmt::Debug;

use winit::event_loop::EventLoopClosed;

use crate::error::{Error, ErrorKind};

impl From<winit::error::OsError> for Error {
    fn from(err: winit::error::OsError) -> Self {
        Error::new(ErrorKind::Context, err)
    }
}

impl From<winit::error::ExternalError> for Error {
    fn from(err: winit::error::ExternalError) -> Self {
        Error::new(ErrorKind::Context, err)
    }
}

impl From<winit::error::NotSupportedError> for Error {
    fn from(err: winit::error::NotSupportedError) -> Self {
        Error::new(ErrorKind::Context, err)
    }
}

impl<E: 'static + Debug> From<winit::event_loop::EventLoopClosed<E>> for Error {
    fn from(err: EventLoopClosed<E>) -> Self {
        Error::new(ErrorKind::Context, err.to_string())
    }
}
