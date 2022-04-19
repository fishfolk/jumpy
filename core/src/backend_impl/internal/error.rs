use glutin::{ContextError, ContextWrapper, CreationError, NotCurrent};
use std::fmt::Debug;

use glutin::event_loop::EventLoopClosed;
use glutin::window::Window;

use crate::error::{Error, ErrorKind};

impl From<glutin::error::OsError> for Error {
    fn from(err: glutin::error::OsError) -> Self {
        Error::new(ErrorKind::Context, err)
    }
}

impl From<glutin::error::ExternalError> for Error {
    fn from(err: glutin::error::ExternalError) -> Self {
        Error::new(ErrorKind::Context, err)
    }
}

impl From<glutin::error::NotSupportedError> for Error {
    fn from(err: glutin::error::NotSupportedError) -> Self {
        Error::new(ErrorKind::Context, err)
    }
}

impl<E: 'static + Debug> From<glutin::event_loop::EventLoopClosed<E>> for Error {
    fn from(err: EventLoopClosed<E>) -> Self {
        Error::new(ErrorKind::Context, err.to_string())
    }
}

impl From<glutin::CreationError> for Error {
    fn from(err: CreationError) -> Self {
        Error::new(ErrorKind::Context, err)
    }
}

impl
    From<(
        glutin::ContextWrapper<glutin::NotCurrent, glutin::window::Window>,
        glutin::ContextError,
    )> for Error
{
    fn from((_, err): (ContextWrapper<NotCurrent, Window>, ContextError)) -> Self {
        err.into()
    }
}

impl From<ContextError> for Error {
    fn from(err: ContextError) -> Self {
        Error::new(ErrorKind::Context, err)
    }
}
