use core_foundation as cf;
use core_foundation::array as cf_array;
use core_foundation::array::{CFArray, CFArrayRef};

use core_foundation::base::{CFGetTypeID, CFRetain, TCFType};
use core_foundation::dictionary::CFDictionary;
use core_foundation::number::{kCFNumberSInt32Type, CFNumber, CFNumberGetValue};
use core_foundation::runloop::{kCFRunLoopRunHandledSource, CFRunLoopGetCurrent, CFRunLoopRunInMode};
use core_foundation::string::CFString;

use io_kit;
use io_kit::{IOHIDDeviceRef, IOHIDElementCookie, IOHIDElementRef, IOReturn};
use libc::c_void;
use std::cell::RefCell;
use std::ptr;
use std::rc::{Rc, Weak};

// define some constants which IOKit_sys are misings
#[allow(non_snake_case, non_upper_case_globals)]
mod io_kit_const {
    use super::*;
    pub const kIOHIDOptionsTypeNone: io_kit::IOOptionBits = 0x0;
    pub const kHIDPage_GenericDesktop: u32 = 0x1;
    pub const kHIDPage_Button: u32 = 0x09;
    pub const kHIDPage_Consumer: u32 = 0x0C;

    pub const kHIDUsage_GD_Joystick: u32 = 0x04;
    pub const kHIDUsage_GD_GamePad: u32 = 0x05;
    pub const kHIDUsage_GD_MultiAxisController: u32 = 0x08;

    pub const kHIDUsage_GD_X: u32 = 0x30;
    pub const kHIDUsage_GD_Y: u32 = 0x31;
    pub const kHIDUsage_GD_Z: u32 = 0x32;
    pub const kHIDUsage_GD_Rx: u32 = 0x33;
    pub const kHIDUsage_GD_Ry: u32 = 0x34;
    pub const kHIDUsage_GD_Rz: u32 = 0x35;
    pub const kHIDUsage_GD_Slider: u32 = 0x36;
    pub const kHIDUsage_GD_Dial: u32 = 0x37;
    pub const kHIDUsage_GD_Wheel: u32 = 0x38;

    pub const kHIDUsage_GD_Hatswitch: u32 = 0x39;

    pub const kHIDUsage_GD_Start: u32 = 0x3D;
    pub const kHIDUsage_GD_Select: u32 = 0x3E;
    pub const kHIDUsage_GD_SystemMainMenu: u32 = 0x85;

    pub const kHIDUsage_GD_DPadUp: u32 = 0x90;
    pub const kHIDUsage_GD_DPadDown: u32 = 0x91;
    pub const kHIDUsage_GD_DPadRight: u32 = 0x92;
    pub const kHIDUsage_GD_DPadLeft: u32 = 0x93;

    pub fn kIOHIDManufacturerKey() -> &'static str {
        "Manufacturer"
    }

    pub fn kIOHIDDeviceUsageKey() -> &'static str {
        "DeviceUsage"
    }
    pub fn kIOHIDDeviceUsagePageKey() -> &'static str {
        "DeviceUsagePage"
    }
    pub fn kIOHIDPrimaryUsagePageKey() -> &'static str {
        "PrimaryUsagePage"
    }
    pub fn kIOHIDPrimaryUsageKey() -> &'static str {
        "PrimaryUsage"
    }
}
use self::io_kit_const::*;

fn gamepad_rs_runloop_mode() -> CFString {
    "GamepadRS".into()
}

#[derive(Debug)]
pub enum Error {
    Unknown(String),
}

type HIDResult<T> = Result<T, Error>;

pub struct HIDStateContext {
    state: Rc<HIDState>,
}

pub struct HIDState {
    hidman: io_kit::IOHIDManagerRef,
    pub devices: RefCell<Vec<Weak<RefCell<Device>>>>,
}

pub struct HID {
    state: Weak<HIDState>,
}

impl Drop for HID {
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            if state.hidman != ptr::null_mut() {
                {
                    let devices = state.devices.borrow();
                    for d in devices.iter() {
                        if let Some(dd) = d.upgrade() {
                            dd.borrow_mut().remove_from_runloop();
                        }
                    }
                }
                state.devices.borrow_mut().clear();

                unsafe {
                    let current_loop = CFRunLoopGetCurrent();
                    let mode = gamepad_rs_runloop_mode();
                    if current_loop != ptr::null_mut() {
                        io_kit::IOHIDManagerUnscheduleFromRunLoop(
                            state.hidman,
                            current_loop as _,
                            mode.as_CFType().as_CFTypeRef() as _,
                        );
                    }
                    io_kit::IOHIDManagerClose(state.hidman, kIOHIDOptionsTypeNone);
                }
            }
        }
    }
}

fn create_hid_device_mach_dictionary(page: u32, usage: u32) -> CFDictionary {
    let page = CFNumber::from(page as i32);
    let usage = CFNumber::from(usage as i32);

    let page_key = CFString::from(kIOHIDDeviceUsagePageKey());
    let usage_key = CFString::from(kIOHIDDeviceUsageKey());

    CFDictionary::from_CFType_pairs(&[(page_key, page), (usage_key, usage)])
}

#[derive(Debug)]
pub struct HIDElement {
    usage: u32,
    page: u32,
    ref_elem: IOHIDElementRef,
    cookie: IOHIDElementCookie,

    min_report: i32,
    max_report: i32,
}

impl HIDElement {
    fn query_axis(&mut self, device_ref: IOHIDDeviceRef, min: i32, max: i32) -> Option<i32> {
        let device_scale = (max - min) as f32;
        let read_scale = (self.max_report - self.min_report) as f32;

        let state = self.query_state(device_ref);
        if state.is_none() {
            return None;
        }
        let state = state.unwrap();

        Some((((state - self.min_report) as f32) * device_scale / read_scale) as i32 + min)
    }

    fn query_state(&mut self, device_ref: IOHIDDeviceRef) -> Option<i32> {
        use std::mem;

        if device_ref == ptr::null_mut() || self.ref_elem == ptr::null_mut() {
            return None;
        }
        let mut value_ref: io_kit::IOHIDValueRef = ptr::null_mut();

        unsafe {
            if io_kit::IOHIDDeviceGetValue(
                device_ref,
                self.ref_elem,
                mem::transmute(&mut value_ref),
            ) == io_kit::kIOReturnSuccess
            {
                let value = io_kit::IOHIDValueGetIntegerValue(value_ref) as i32;

                // record min and max for auto calibration
                self.min_report = self.min_report.min(value);
                self.max_report = self.max_report.max(value);

                return Some(value);
            }
        }

        None
    }
}

struct DeviceContext {
    device: Rc<RefCell<Device>>,
}

#[derive(Default, Debug)]
pub struct DeviceState {
    pub sequence: usize,
    pub digital_state: Vec<bool>,
    pub analog_state: Vec<f32>,
}

#[derive(Debug)]
pub struct Device {
    usage: i32,
    page: i32,
    device: IOHIDDeviceRef,

    pub product: String,
    pub axes: Vec<HIDElement>,
    pub hats: Vec<HIDElement>,
    pub buttons: Vec<HIDElement>,

    pub state: DeviceState,
}

impl Drop for Device {
    fn drop(&mut self) {
        self.remove_from_runloop();
    }
}

unsafe fn get_property_i32(dev: IOHIDDeviceRef, s: &'static str) -> Option<i32> {
    use std::mem;

    let key = CFString::from(s);
    let ref_cf = io_kit::IOHIDDeviceGetProperty(dev, key.as_CFTypeRef() as _);
    if ref_cf == ptr::null() {
        return None;
    }

    let mut value: i32 = 0;
    let ok = CFNumberGetValue(ref_cf as _, kCFNumberSInt32Type, mem::transmute(&mut value));

    if ok {
        Some(value)
    } else {
        None
    }
}

unsafe fn get_property_str(dev: IOHIDDeviceRef, s: &'static str) -> Option<String> {
    let key = CFString::from(s);
    let ref_cf = io_kit::IOHIDDeviceGetProperty(dev, key.as_CFTypeRef() as _);
    if ref_cf == ptr::null() {
        return None;
    }

    let cf_str = CFString::wrap_under_create_rule(ref_cf as _);
    Some(cf_str.to_string())
}

impl Device {
    fn remove_from_runloop(&mut self) {
        if self.device != ptr::null_mut() {
            use std::mem;
            unsafe {
                io_kit::IOHIDDeviceRegisterRemovalCallback(
                    self.device,
                    mem::transmute::<*const (), _>(ptr::null()),
                    ptr::null_mut(),
                );

                // We only work in main thread, so that it
                // seem to no need to schedule it
                //
                // TODO: Find out how to know do it properly.

                // let current_loop = CFRunLoopGetCurrent();
                // let mode = gamepad_rs_runloop_mode();
                // // io_kit::IOHIDDeviceUnscheduleFromRunLoop(
                //     self.device,
                //     current_loop as _,
                //     mode.as_CFType().as_CFTypeRef() as _,
                // );
            }

            self.device = ptr::null_mut();
        }
    }

    fn contain_element(&self, cookie: IOHIDElementCookie) -> bool {
        self.axes.iter().any(|elm| elm.cookie == cookie)
            || self.buttons.iter().any(|elm| elm.cookie == cookie)
            || self.hats.iter().any(|elm| elm.cookie == cookie)
    }

    fn add_element(&mut self, ref_elem: IOHIDElementRef) {
        if ref_elem == ptr::null_mut() {
            return;
        }

        let elem_type_id = unsafe { CFGetTypeID(ref_elem as _) };
        if elem_type_id != unsafe { io_kit::IOHIDElementGetTypeID() } {
            return;
        }

        unsafe {
            let cookie = io_kit::IOHIDElementGetCookie(ref_elem);
            let page = io_kit::IOHIDElementGetUsagePage(ref_elem);
            let usage = io_kit::IOHIDElementGetUsage(ref_elem);

            match io_kit::IOHIDElementGetType(ref_elem) {
                io_kit::kIOHIDElementTypeCollection => {
                    let children = io_kit::IOHIDElementGetChildren(ref_elem);
                    self.add_elements(children as _);
                }

                io_kit::kIOHIDElementTypeInput_Misc
                | io_kit::kIOHIDElementTypeInput_Button
                | io_kit::kIOHIDElementTypeInput_Axis => match page {
                    io_kit_const::kHIDPage_GenericDesktop => match usage {
                        io_kit_const::kHIDUsage_GD_X
                        | io_kit_const::kHIDUsage_GD_Y
                        | io_kit_const::kHIDUsage_GD_Z
                        | io_kit_const::kHIDUsage_GD_Rx
                        | io_kit_const::kHIDUsage_GD_Ry
                        | io_kit_const::kHIDUsage_GD_Rz
                        | io_kit_const::kHIDUsage_GD_Slider
                        | io_kit_const::kHIDUsage_GD_Dial
                        | io_kit_const::kHIDUsage_GD_Wheel => {
                            if !self.contain_element(cookie) {
                                self.axes.push(HIDElement {
                                    usage,
                                    page,
                                    ref_elem,
                                    cookie,
                                    min_report: io_kit::IOHIDElementGetLogicalMin(ref_elem) as i32,
                                    max_report: io_kit::IOHIDElementGetLogicalMax(ref_elem) as i32,
                                });
                            }
                        }
                        io_kit_const::kHIDUsage_GD_Hatswitch => {
                            if !self.contain_element(cookie) {
                                self.hats.push(HIDElement {
                                    usage,
                                    page,
                                    ref_elem,
                                    cookie,
                                    min_report: io_kit::IOHIDElementGetLogicalMin(ref_elem) as i32,
                                    max_report: io_kit::IOHIDElementGetLogicalMax(ref_elem) as i32,

                                });
                            }
                        }

                        io_kit_const::kHIDUsage_GD_DPadUp
                        | io_kit_const::kHIDUsage_GD_DPadDown
                        | io_kit_const::kHIDUsage_GD_DPadRight
                        | io_kit_const::kHIDUsage_GD_DPadLeft
                        | io_kit_const::kHIDUsage_GD_Start
                        | io_kit_const::kHIDUsage_GD_Select
                        | io_kit_const::kHIDUsage_GD_SystemMainMenu => {
                            if !self.contain_element(cookie) {
                                self.buttons.push(HIDElement {
                                    usage,
                                    page,
                                    ref_elem,
                                    cookie,
                                    min_report: io_kit::IOHIDElementGetLogicalMin(ref_elem) as i32,
                                    max_report: io_kit::IOHIDElementGetLogicalMax(ref_elem) as i32,

                                });
                            }
                        }
                        _ => (),
                    },
                    io_kit_const::kHIDPage_Button | 
                    // e.g. 'pause' button on Steelseries MFi gamepads.
                    io_kit_const::kHIDPage_Consumer => {                        
                        if !self.contain_element(cookie) {
                            self.buttons.push(HIDElement {
                                usage,
                                page,
                                ref_elem,
                                cookie,
                                min_report: io_kit::IOHIDElementGetLogicalMin(ref_elem) as i32,
                                max_report: io_kit::IOHIDElementGetLogicalMax(ref_elem) as i32,
                            });
                        }
                    }

                    _ => (),
                },

                _ => {}
            }
        }
    }

    fn add_elements(&mut self, array: CFArrayRef) {
        use self::cf_array::*;

        let count = unsafe { CFArrayGetCount(array) };

        for i in 0..count {
            unsafe {
                let ref_elem: IOHIDElementRef = CFArrayGetValueAtIndex(array, i) as _;
                self.add_element(ref_elem);
            }
        }
    }

    fn from_raw_dev(dev: IOHIDDeviceRef) -> Option<Rc<RefCell<Device>>> {
        unsafe {
            let page = {
                if let Some(val) = get_property_i32(dev, kIOHIDPrimaryUsagePageKey()) {
                    val
                } else {
                    return None;
                }
            };
            //  Filter device list to non-keyboard/mouse stuff
            if page != kHIDPage_GenericDesktop as i32 {
                return None;
            }

            let usage = {
                if let Some(val) = get_property_i32(dev, kIOHIDPrimaryUsageKey()) {
                    val
                } else {
                    return None;
                }
            };

            //  Filter device list to non-keyboard/mouse stuff
            if usage != kHIDUsage_GD_Joystick as i32 && usage != kHIDUsage_GD_GamePad as i32
                && usage != kHIDUsage_GD_MultiAxisController as i32
            {
                return None;
            }

            let product = get_property_str(dev, kIOHIDManufacturerKey())
                .unwrap_or("Unidentified joystick".to_owned());

            let mut device = Device {
                usage,
                page,
                product,
                device: dev,
                hats: Vec::new(),
                axes: Vec::new(),
                buttons: Vec::new(),
                state: Default::default(),
            };

            let array_cf =
                io_kit::IOHIDDeviceCopyMatchingElements(dev, ptr::null(), kIOHIDOptionsTypeNone);

            if array_cf != ptr::null() {
                device.add_elements(array_cf as _);

                // sort all elements by usage
                device.buttons.sort_by(|a, b| a.usage.cmp(&b.usage));
                device.axes.sort_by(|a, b| a.usage.cmp(&b.usage));
                device.hats.sort_by(|a, b| a.usage.cmp(&b.usage));
            }

            // TODO:
            // Handle Mapping?
            // https://github.com/spurious/SDL-mirror/blob/93215c489ac11a2b24b2e2665ee729431fdf537c/src/joystick/darwin/SDL_sysjoystick.c#L458

            let device = Rc::new(RefCell::new(device));
            let device_ctx = Box::new(DeviceContext {
                device: device.clone(),
            });

            // Get notified when this device is disconnected.
            io_kit::IOHIDDeviceRegisterRemovalCallback(
                device.borrow().device,
                joystick_device_was_removed_cb,
                Box::into_raw(device_ctx) as _,
            );

            // We only work in main thread, so that it
            // seem to no need to schedule it
            //
            // TODO: Find out how to know do it properly.

            // let run_loop = CFRunLoopGetCurrent();
            // let mode = gamepad_rs_runloop_mode();
            // io_kit::IOHIDDeviceScheduleWithRunLoop(
            //     dev,
            //     run_loop as _,
            //     mode.as_CFType().as_CFTypeRef() as _,
            // );

            Some(device)
        }
    }
}

extern "C" fn joystick_device_was_removed_cb(
    context: *mut c_void,
    _res: IOReturn,
    _sender: *mut c_void,
) {
    let dev: *mut DeviceContext = context as *mut DeviceContext;

    unsafe {
        let b = Box::from_raw(dev);
        b.device.borrow_mut().device = ptr::null_mut();
    }
}

extern "C" fn joystick_device_was_added_cb(
    context: *mut c_void,
    res: IOReturn,
    _sender: *mut c_void,
    device: IOHIDDeviceRef,
) {
    if res != io_kit::kIOReturnSuccess {
        return;
    }

    // cast the context back to HID
    let hid_state_ctx: &mut HIDStateContext = unsafe { &mut *(context as *mut HIDStateContext) };

    if hid_state_ctx.state.already_known(device) {
        // IOKit sent us a duplicate
        return;
    }

    if let Some(dev) = Device::from_raw_dev(device) {
        hid_state_ctx
            .state
            .devices
            .borrow_mut()
            .push(Rc::downgrade(&dev));
    }
}

impl HIDState {
    pub fn already_known(&self, device: IOHIDDeviceRef) -> bool {
        self.devices.borrow().iter().any(|dev| {
            if let Some(p) = dev.upgrade() {
                p.borrow().device == device
            } else {
                false
            }
        })
    }
}

impl HID {
    // Return number of devices
    pub fn num_devices(&self) -> usize {
        let mut n = 0;

        let state = self.hid_state();
        let devices = state.devices.borrow();

        for dev in devices.iter() {
            if let Some(_) = dev.upgrade() {
                n += 1;
            }
        }

        n
    }

    // Query the new devices is inserted or not
    pub fn detect_devices(&mut self) {
        // Remove all empty weak pointer
        {
            let state = self.hid_state();
            let mut devices = state.devices.borrow_mut();
            devices.retain(|p| p.upgrade().is_some());
        }

        unsafe {
            let mode = gamepad_rs_runloop_mode();
            while CFRunLoopRunInMode(mode.as_CFTypeRef() as _, 0.0, 1) == kCFRunLoopRunHandledSource
            {
                /* no-op. Pending callbacks will fire in CFRunLoopRunInMode(). */
            }
        }
    }

    pub fn update(&mut self, dev_index: usize) {
        let state = self.hid_state();
        let devices = state.devices.borrow();
        if dev_index >= devices.len() {
            return;
        }

        let dev = &devices[dev_index].upgrade();
        if dev.is_none() {
            return;
        }

        let dev = dev.as_ref().unwrap();
        let mut dev_bor = dev.borrow_mut();
        let device_ref = dev_bor.device;

        let mut new_state = DeviceState::default();

        for btn in dev_bor.buttons.iter_mut() {
            if let Some(state) = btn.query_state(device_ref) {
                new_state.digital_state.push(state != 0);
            }
        }

        for axis in dev_bor.axes.iter_mut() {
            if let Some(state) = axis.query_axis(device_ref, -32768, 32767) {
                new_state.analog_state.push(state as f32 / 32768.0);
            }
        }

        dev_bor.state = new_state;
    }

    pub fn new() -> HIDResult<HID> {
        let hidman = unsafe {
            let hidman = io_kit::IOHIDManagerCreate(
                cf::base::kCFAllocatorDefault as _,
                io_kit::kIOHIDManagerOptionNone,
            );

            if io_kit::kIOReturnSuccess != io_kit::IOHIDManagerOpen(hidman, kIOHIDOptionsTypeNone) {
                return Err(Error::Unknown("Fail to open HID Manager".to_owned()));
            }

            CFRetain(hidman as _);

            hidman
        };

        let mut hid = HID { state: Weak::new() };

        hid.config_manager(Rc::new(HIDState {
            hidman,
            devices: RefCell::new(Vec::new()),
        }));

        Ok(hid)
    }

    pub fn hid_state(&self) -> Rc<HIDState> {
        self.state.upgrade().unwrap()
    }

    fn config_manager(&mut self, state: Rc<HIDState>) {
        self.state = Rc::downgrade(&state);

        unsafe {
            let array = CFArray::from_CFTypes(&[
                create_hid_device_mach_dictionary(kHIDPage_GenericDesktop, kHIDUsage_GD_Joystick),
                create_hid_device_mach_dictionary(kHIDPage_GenericDesktop, kHIDUsage_GD_GamePad),
                create_hid_device_mach_dictionary(
                    kHIDPage_GenericDesktop,
                    kHIDUsage_GD_MultiAxisController,
                ),
            ]);

            let runloop = CFRunLoopGetCurrent();

            let state_ctx = Box::new(HIDStateContext {
                state: state.clone(),
            });

            io_kit::IOHIDManagerSetDeviceMatchingMultiple(state.hidman, array.as_CFTypeRef() as _);
            io_kit::IOHIDManagerRegisterDeviceMatchingCallback(
                state.hidman,
                joystick_device_was_added_cb,
                Box::into_raw(state_ctx) as _,
            );

            let mode = gamepad_rs_runloop_mode();
            io_kit::IOHIDManagerScheduleWithRunLoop(
                state.hidman,
                runloop as _,
                mode.as_CFTypeRef() as _,
            );

            // joystick_device_was_added_cb will be called if there are any devices
            while CFRunLoopRunInMode(mode.as_CFTypeRef() as _, 0.0, 1) == kCFRunLoopRunHandledSource
            {
                /* no-op. Callback fires once per existing device. */
            }
        }
    }
}
