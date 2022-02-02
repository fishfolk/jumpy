use crate::RequestStatus;
use std::{error, fmt, result};

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Api,
    Network,
}

impl ErrorKind {
    pub fn as_str(&self) -> &'static str {
        match *self {
            ErrorKind::Api => "Api error",
            ErrorKind::Network => "Network error",
        }
    }
}

enum Repr {
    Simple(ErrorKind),
    Message(ErrorKind, String),
    SimpleMessage(ErrorKind, &'static &'static str),
    Custom(Box<Custom>),
}

#[derive(Debug)]
struct Custom {
    kind: ErrorKind,
    error: Box<dyn error::Error + Send + Sync>,
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
        E: Into<Box<dyn error::Error + Send + Sync>>,
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

impl From<RequestStatus> for Error {
    fn from(status: RequestStatus) -> Self {
        Error::new_message(
            ErrorKind::Api,
            &format!("[{}]: {}", status.as_code(), status.as_str()),
        )
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
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self.repr {
            Repr::Simple(..) => None,
            Repr::Message(..) => None,
            Repr::SimpleMessage(..) => None,
            Repr::Custom(ref c) => c.error.source(),
        }
    }
}
