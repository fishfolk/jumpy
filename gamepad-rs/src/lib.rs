#[cfg(target_os = "linux")]
extern crate libc;

#[cfg(target_os = "windows")]
extern crate winapi;

#[cfg(target_os = "macos")]
extern crate IOKit_sys as io_kit;
#[cfg(target_os = "macos")]
extern crate core_foundation;
#[cfg(target_os = "macos")]
extern crate libc;

#[macro_use]
extern crate lazy_static;

mod platform;

pub use self::platform::*;

pub const MAX_DEVICES: usize = 8;
pub const MAX_DIGITAL: usize = 16;
pub const MAX_ANALOG: usize = 8;

#[derive(Debug)]
pub struct ControllerInfo {
    pub name: String,
    pub digital_count: usize,
    pub analog_count: usize,
}

impl ControllerInfo {
    pub fn new() -> Self {
        Self {
            name: "null".to_owned(),
            digital_count: 0,
            analog_count: 0,
        }
    }
}

lazy_static! {
    static ref DEFAULT_CONTROLLER_INFO: ControllerInfo = ControllerInfo {
        name: String::from("null"),
        digital_count: 0,
        analog_count: 0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControllerStatus {
    Disconnected,
    Connected,
}

#[derive(Debug)]
pub struct ControllerState {
    pub status: ControllerStatus,
    pub sequence: usize,
    pub digital_state: [bool; MAX_DIGITAL],
    pub analog_state: [f32; MAX_ANALOG],
}

impl ControllerState {
    pub fn new() -> Self {
        Self {
            status: ControllerStatus::Disconnected,
            sequence: 0,
            digital_state: [false; MAX_DIGITAL],
            analog_state: [0.0; MAX_ANALOG],
        }
    }
}

const DEFAULT_CONTROLLER_STATE: ControllerState = ControllerState {
    status: ControllerStatus::Disconnected,
    sequence: 0,
    digital_state: [false; MAX_DIGITAL],
    analog_state: [0.0; MAX_ANALOG],
};
