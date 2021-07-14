use super::super::{
    ControllerInfo, ControllerState, ControllerStatus, DEFAULT_CONTROLLER_INFO,
    DEFAULT_CONTROLLER_STATE,
};

use std::{path::Path, sync::mpsc};

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum JsEventType {
    /// button pressed/released
    Button = 0x01,
    /// button pressed/released
    Axis = 0x02,
    /// initial state of device
    Init = 0x80,
}

#[derive(Debug, Clone, Copy)]
#[repr(C)]
struct JsEvent {
    /// event timestamp in milliseconds
    time: u32,
    /// value
    value: i16,
    /// event type
    type_: u8,
    /// axis/button number
    number: u8,
}

fn joystick_thread(tx: mpsc::Sender<JsEvent>, path: &str) {
    // If a joystick is not connected, the corresponding block device may not exist.
    //
    if !Path::new(path).exists() {
        return;
    }

    use std::fs::File;
    use std::io::Read;
    let mut f = File::open(path).unwrap();
    loop {
        let mut b: [u8; std::mem::size_of::<JsEvent>()] = [0; std::mem::size_of::<JsEvent>()];
        f.read_exact(&mut b).unwrap();
        let event: JsEvent = unsafe { std::mem::transmute(b) };
        tx.send(event).unwrap();
    }
}
pub struct ControllerContext {
    rx: mpsc::Receiver<JsEvent>,
    rx1: mpsc::Receiver<JsEvent>,
    controller_state: [ControllerState; 2],
}

impl ControllerContext {
    pub fn new() -> Option<Self> {
        let (tx, rx) = mpsc::channel();
        let (tx1, rx1) = mpsc::channel();

        std::thread::spawn(move || joystick_thread(tx, "/dev/input/js0"));
        std::thread::spawn(move || joystick_thread(tx1, "/dev/input/js1"));

        Some(Self {
            rx,
            rx1,
            controller_state: [
                ControllerState {
                    status: ControllerStatus::Connected,
                    ..DEFAULT_CONTROLLER_STATE
                },
                ControllerState {
                    status: ControllerStatus::Connected,
                    ..DEFAULT_CONTROLLER_STATE
                },
            ],
        })
    }
    /// Update controller state by index
    pub fn update(&mut self, _index: usize) {
        while let Ok(event) = self.rx.try_recv() {
            if (event.type_ & JsEventType::Axis as u8) != 0 {
                let value = event.value as f32 / std::u16::MAX as f32 * 2.;
                self.controller_state[0].analog_state[event.number as usize] = value;
            }
            if (event.type_ & JsEventType::Button as u8) != 0 {
                self.controller_state[0].digital_state[event.number as usize] = event.value != 0;
            }
            if (event.type_ & JsEventType::Init as u8) != 0 {}
        }

        while let Ok(event) = self.rx1.try_recv() {
            if (event.type_ & JsEventType::Axis as u8) != 0 {
                let value = event.value as f32 / std::u16::MAX as f32 * 2.;
                self.controller_state[1].analog_state[event.number as usize] = value;
            }
            if (event.type_ & JsEventType::Button as u8) != 0 {
                self.controller_state[1].digital_state[event.number as usize] = event.value != 0;
            }
            if (event.type_ & JsEventType::Init as u8) != 0 {}
        }
    }

    pub fn info(&self, _index: usize) -> &ControllerInfo {
        &*DEFAULT_CONTROLLER_INFO
    }
    pub fn state(&self, index: usize) -> &ControllerState {
        &self.controller_state[index]
    }
}
