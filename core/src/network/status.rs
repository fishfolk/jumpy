#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_json", serde(rename_all = "snake_case"))]
pub enum RequestStatus {
    Ok,
    Unauthorized,
    NotFound,
    RequestTimeout,
    InternalServerError,
    Unknown,
}

impl RequestStatus {
    pub fn as_code(&self) -> u16 {
        match *self {
            RequestStatus::Ok => 200,
            RequestStatus::Unauthorized => 401,
            RequestStatus::NotFound => 404,
            RequestStatus::RequestTimeout => 408,
            RequestStatus::InternalServerError => 500,
            RequestStatus::Unknown => 0,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match *self {
            RequestStatus::Ok => "ok",
            RequestStatus::Unauthorized => "unauthorized",
            RequestStatus::NotFound => "not found",
            RequestStatus::RequestTimeout => "request timeout",
            RequestStatus::InternalServerError => "internal server error",
            RequestStatus::Unknown => "unknown",
        }
    }
}

impl From<u16> for RequestStatus {
    fn from(code: u16) -> Self {
        match code {
            200 => RequestStatus::Ok,
            401 => RequestStatus::Unauthorized,
            404 => RequestStatus::NotFound,
            408 => RequestStatus::RequestTimeout,
            500 => RequestStatus::InternalServerError,
            _ => RequestStatus::Unknown,
        }
    }
}
