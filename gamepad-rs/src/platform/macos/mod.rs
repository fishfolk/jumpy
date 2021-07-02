mod hid;

use super::super::{ControllerInfo, ControllerState, ControllerStatus, DEFAULT_CONTROLLER_INFO,
                   DEFAULT_CONTROLLER_STATE, MAX_ANALOG, MAX_DEVICES, MAX_DIGITAL};

pub struct ControllerContext {
    info: Vec<ControllerInfo>,
    state: Vec<ControllerState>,
    hid: hid::HID,
}

// Helper function for convert Vec to array
fn to_digital_state_array(state: &Vec<bool>) -> [bool; MAX_DIGITAL] {
    let mut arr = [false; MAX_DIGITAL];
    for (place, element) in arr.iter_mut().zip(state.iter()) {
        *place = *element;
    }
    arr
}

fn to_analog_state_array(state: &Vec<f32>) -> [f32; MAX_ANALOG] {
    let mut arr = [0.0; MAX_ANALOG];
    for (place, element) in arr.iter_mut().zip(state.iter()) {
        *place = *element;
    }
    arr
}

impl ControllerContext {
    pub fn new() -> Option<Self> {
        let mut info = Vec::new();
        let mut state = Vec::new();
        for _ in 0..MAX_DEVICES {
            info.push(ControllerInfo::new());
            state.push(ControllerState::new());
        }

        let hid = hid::HID::new();

        match hid {
            Ok(hid) => Some(Self { info, state, hid }),
            Err(err) => {
                println!("Error on create HID. reason: {:?}", err);
                None
            }
        }
    }
    pub fn scan_controllers(&mut self) -> usize {
        self.hid.detect_devices();

        let ndev = self.hid.num_devices();
        let state = self.hid.hid_state();
        let devices = state.devices.borrow();

        for (i, dev) in devices.iter().enumerate() {
            if i >= MAX_DEVICES {
                break;
            }

            if let Some(d) = dev.upgrade() {
                let d = d.borrow();

                self.info[i] = ControllerInfo {
                    name: d.product.clone(),
                    analog_count: d.axes.len(),
                    digital_count: d.buttons.len(),
                };
                self.state[i].status = ControllerStatus::Connected;
            } else {
                self.info[i] = ControllerInfo::new();
                self.state[i].status = ControllerStatus::Disconnected;
            }
        }

        ndev
    }
    /// Update controller state by index
    pub fn update(&mut self, index: usize) {
        self.hid.update(index);
        let state = self.hid.hid_state();
        let devices = state.devices.borrow();

        if index >= devices.len() || index >= MAX_DEVICES {
            return;
        }

        let dev = &devices[index];

        if let Some(d) = dev.upgrade() {
            let dev_bor = d.borrow();
            self.state[index] = ControllerState {
                status: ControllerStatus::Connected,
                sequence: dev_bor.state.sequence,
                analog_state: to_analog_state_array(&dev_bor.state.analog_state),
                digital_state: to_digital_state_array(&dev_bor.state.digital_state),
            }
        } else {
            self.state[index].status = ControllerStatus::Disconnected;
        }
    }
    /// Get current information of Controller
    pub fn info(&self, index: usize) -> &ControllerInfo {
        if index < MAX_DEVICES {
            &self.info[index]
        } else {
            &*DEFAULT_CONTROLLER_INFO
        }
    }
    /// Get current state of Controller
    pub fn state(&self, index: usize) -> &ControllerState {
        if index < MAX_DEVICES {
            &self.state[index]
        } else {
            &DEFAULT_CONTROLLER_STATE
        }
    }
}
