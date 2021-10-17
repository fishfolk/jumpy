use std::{error, fmt, io, result, string::FromUtf8Error};

use macroquad::prelude::{FileError, FontError};
use nanoserde::DeJsonErr;

// This implements a simple Error and Result type, inspired by `io::Error` and `io::Result`, that
// enables us to work seamlessly with all the different `Error` implementations from our dependencies.
//
// Just implement `From` for `Error`, for any remote implementations of `Error` you encounter, and
// use the `Result` type alias, from this module, as return type when it is required.

pub type Result<T> = result::Result<T, Error>;

enum Repr {
    Simple(ErrorKind),
    Message(ErrorKind, String),
    SimpleMessage(ErrorKind, &'static &'static str),
    Custom(Box<Custom>),
}

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn std::error::Error + Send + Sync>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    General,
    File,
    Parsing,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::General => "General error",
            ErrorKind::File => "File error",
            ErrorKind::Parsing => "Parsing error",
        }
    }
}

pub struct Error {
    repr: Repr,
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.repr, f)
    }
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Error {
            repr: Repr::Custom(Box::new(Custom {
                kind,
                error: error.into(),
            })),
        }
    }

    pub fn new_message(kind: ErrorKind, msg: &str) -> Self {
        Error {
            repr: Repr::Message(kind, msg.to_string()),
        }
    }

    pub const fn new_const(kind: ErrorKind, msg: &'static &'static str) -> Self {
        Error {
            repr: Repr::SimpleMessage(kind, msg),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        match self.repr {
            Repr::Custom(ref c) => c.kind,
            Repr::Simple(kind) => kind,
            Repr::SimpleMessage(kind, _) => kind,
            Repr::Message(kind, _) => kind,
        }
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            repr: Repr::Simple(kind),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.repr {
            Repr::Custom(ref c) => c.error.fmt(f),
            Repr::Simple(kind) => write!(f, "{}", kind.as_str()),
            Repr::SimpleMessage(_, &msg) => msg.fmt(f),
            Repr::Message(_, msg) => msg.fmt(f),
        }
    }
}

impl fmt::Debug for Repr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Repr::Simple(kind) => f.debug_tuple("Kind").field(kind).finish(),
            Repr::SimpleMessage(kind, &message) => f
                .debug_struct("Error")
                .field("kind", kind)
                .field("message", &message)
                .finish(),
            Repr::Message(kind, message) => f
                .debug_struct("Error")
                .field("kind", kind)
                .field("message", &message)
                .finish(),
            Repr::Custom(ref c) => c.error.fmt(f),
        }
    }
}

impl error::Error for Error {
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        match &self.repr {
            Repr::Simple(kind) => kind.as_str(),
            Repr::Message(_, msg) => msg,
            Repr::SimpleMessage(_, &msg) => msg,
            Repr::Custom(ref c) => c.error.description(),
        }
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn error::Error> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Message(..) => None,
            Repr::SimpleMessage(..) => None,
            Repr::Custom(ref c) => c.error.cause(),
        }
    }

    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Message(..) => None,
            Repr::SimpleMessage(..) => None,
            Repr::Custom(ref c) => c.error.source(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::new(ErrorKind::File, error)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(error: FromUtf8Error) -> Self {
        Error::new(ErrorKind::Parsing, error)
    }
}

impl From<FileError> for Error {
    fn from(error: FileError) -> Self {
        Error::new(ErrorKind::File, error)
    }
}

impl From<FontError> for Error {
    fn from(error: FontError) -> Self {
        Error::new(ErrorKind::Parsing, error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::new(ErrorKind::Parsing, error)
    }
}

impl From<ron::Error> for Error {
    fn from(error: ron::Error) -> Self {
        Error::new(ErrorKind::Parsing, error)
    }
}

impl From<nanoserde::DeJsonErr> for Error {
    fn from(error: DeJsonErr) -> Self {
        Error::new(ErrorKind::Parsing, error)
    }
}

// This will create an error based on the parameters you provide.
// It follows the same rules as `format!`, only this takes an optional `ErrorKind`, as its
// first argument (before the format string), which will be the kind of `Error` returned.
// If no `ErrorKind` is specified, the default variant `ErrorKind::General` will be used.
#[macro_export]
macro_rules! formaterr {
    ($kind:path, $($arg:tt)*) => ({
        let res = format!($($arg)*);
        $crate::error::Error::new_message($kind, &res)
    });
    ($($arg:tt)*) => ({
        let res = format!($($arg)*);
        $crate::error::Error::new_message($crate::error::ErrorKind::General, &res)
    });
}
