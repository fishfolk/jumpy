use std::mem;

use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::xinput::{self, XINPUT_CAPABILITIES as XCapabilities, XINPUT_STATE as XState,
                         XINPUT_FLAG_GAMEPAD, XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B,
                         XINPUT_GAMEPAD_BACK, XINPUT_GAMEPAD_DPAD_DOWN, XINPUT_GAMEPAD_DPAD_LEFT,
                         XINPUT_GAMEPAD_DPAD_RIGHT, XINPUT_GAMEPAD_DPAD_UP,
                         XINPUT_GAMEPAD_LEFT_SHOULDER, XINPUT_GAMEPAD_LEFT_THUMB,
                         XINPUT_GAMEPAD_RIGHT_SHOULDER, XINPUT_GAMEPAD_RIGHT_THUMB,
                         XINPUT_GAMEPAD_START, XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y};

use super::super::{ControllerInfo, ControllerState, ControllerStatus, DEFAULT_CONTROLLER_INFO,
                   DEFAULT_CONTROLLER_STATE, MAX_DEVICES, MAX_DIGITAL};

pub struct ControllerContext {
    info: Vec<ControllerInfo>,
    state: Vec<ControllerState>,
    buttons: Vec<[u16; MAX_DIGITAL]>,
}

impl ControllerContext {
    pub fn new() -> Option<Self> {
        let mut info = Vec::new();
        let mut state = Vec::new();
        let mut buttons = Vec::new();
        unsafe { xinput::XInputEnable(1) };
        for _ in 0..MAX_DEVICES {
            info.push(ControllerInfo::new());
            state.push(ControllerState::new());
            buttons.push([0; MAX_DIGITAL]);
        }
        Some(Self {
            info,
            state,
            buttons,
        })
    }
    /// Scan all device and return number of valid controllers
    pub fn scan_controllers(&mut self) -> usize {
        let mut count = 0;
        let mut state = unsafe { mem::zeroed::<XState>() };
        for id in 0..4 {
            let val = unsafe { xinput::XInputGetState(id as u32, &mut state) };
            if val == ERROR_SUCCESS {
                count += 1;
                self.state[id].status = ControllerStatus::Connected;
            } else {
                self.state[id].status = ControllerStatus::Disconnected;
            }
        }
        count
    }
    /// Update controller state by index
    pub fn update(&mut self, index: usize) {
        let mut state = unsafe { mem::zeroed::<XState>() };
        let val = unsafe { xinput::XInputGetState(index as u32, &mut state) };
        if val == ERROR_SUCCESS {
            self.state[index].status = ControllerStatus::Connected;
            self.update_state(index, &state);
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
    fn update_state(&mut self, index: usize, state: &XState) {
        if state.dwPacketNumber as usize == self.state[index].sequence {
            // no change in state
            return;
        }
        self.state[index].sequence = state.dwPacketNumber as usize;
        if self.info[index].digital_count == 0 {
            // we did not yet get the capabilities
            let mut capabilities = unsafe { mem::zeroed::<XCapabilities>() };
            if unsafe {
                xinput::XInputGetCapabilities(index as u32, XINPUT_FLAG_GAMEPAD, &mut capabilities)
            } == ERROR_SUCCESS
            {
                self.update_info(index, &capabilities);
            }
        }
        for i in 0..self.info[index].digital_count {
            self.state[index].digital_state[i] =
                state.Gamepad.wButtons & self.buttons[index][i] != 0;
        }
        self.state[index].analog_state[0] =
            (state.Gamepad.sThumbLX as i32 + 32768) as f32 / 65535.0 * 2.0 - 1.0;
        self.state[index].analog_state[1] =
            (state.Gamepad.sThumbLY as i32 + 32768) as f32 / 65535.0 * 2.0 - 1.0;
        self.state[index].analog_state[2] = state.Gamepad.bLeftTrigger as f32 / 255.0 * 2.0 - 1.0;
        self.state[index].analog_state[3] = state.Gamepad.bRightTrigger as f32 / 255.0 * 2.0 - 1.0;
        self.state[index].analog_state[4] =
            (state.Gamepad.sThumbRX as i32 + 32768) as f32 / 65535.0 * 2.0 - 1.0;
        self.state[index].analog_state[5] =
            (state.Gamepad.sThumbRY as i32 + 32768) as f32 / 65535.0 * 2.0 - 1.0;
    }
    fn update_info(&mut self, index: usize, capabilities: &XCapabilities) {
        let mut name = String::from("XBOX360");
        match capabilities.SubType {
            xinput::XINPUT_DEVSUBTYPE_GAMEPAD => name.push_str(" gamepad"),
            xinput::XINPUT_DEVSUBTYPE_WHEEL => name.push_str(" wheel"),
            xinput::XINPUT_DEVSUBTYPE_ARCADE_STICK => name.push_str(" arcade stick"),
            xinput::XINPUT_DEVSUBTYPE_FLIGHT_SICK => name.push_str(" flight stick"),
            xinput::XINPUT_DEVSUBTYPE_DANCE_PAD => name.push_str(" dance pad"),
            xinput::XINPUT_DEVSUBTYPE_GUITAR => name.push_str(" guitar"),
            xinput::XINPUT_DEVSUBTYPE_DRUM_KIT => name.push_str(" drum"),
            _ => (),
        };
        name.push_str(" controller");
        self.info[index].name = name;
        let mut buttons = 0;
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_A != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_A;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_B != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_B;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_X != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_X;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_Y != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_Y;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_DPAD_UP != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_DPAD_UP;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_DPAD_DOWN != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_DPAD_DOWN;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_DPAD_LEFT != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_DPAD_LEFT;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_DPAD_RIGHT != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_DPAD_RIGHT;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_START != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_START;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_BACK != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_BACK;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_LEFT_THUMB != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_LEFT_THUMB;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_RIGHT_THUMB != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_RIGHT_THUMB;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_LEFT_SHOULDER != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_LEFT_SHOULDER;
            buttons += 1;
        }
        if capabilities.Gamepad.wButtons & XINPUT_GAMEPAD_RIGHT_SHOULDER != 0 {
            self.buttons[index][buttons] = XINPUT_GAMEPAD_RIGHT_SHOULDER;
            buttons += 1;
        }
        self.info[index].digital_count = buttons;
        let mut axis = 0;
        if capabilities.Gamepad.bLeftTrigger != 0 {
            axis += 1;
        }
        if capabilities.Gamepad.bRightTrigger != 0 {
            axis += 1;
        }
        if capabilities.Gamepad.sThumbLX != 0 {
            axis += 1;
        }
        if capabilities.Gamepad.sThumbLY != 0 {
            axis += 1;
        }
        if capabilities.Gamepad.sThumbRX != 0 {
            axis += 1;
        }
        if capabilities.Gamepad.sThumbRY != 0 {
            axis += 1;
        }
        self.info[index].analog_count = axis;
    }
}
