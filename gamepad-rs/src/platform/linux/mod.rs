// https://github.com/glfw/glfw/blob/master/src/linux_joystick.c

use super::super::{
    ControllerInfo, ControllerState, ControllerStatus, DEFAULT_CONTROLLER_INFO,
    DEFAULT_CONTROLLER_STATE,
};

use std::path::PathBuf;

mod ioctl;
mod linux_input;

use self::ioctl::eviocgabs;
use self::linux_input::*;

fn is_bit_set(bit: usize, arr: &[u8]) -> bool {
    return (arr[bit / 8] & (1 << ((bit as usize) % 8))) != 0;
}

#[repr(C)]
#[derive(Default, Debug)]
struct InputId {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct InputAbsinfo {
    pub value: i32,
    pub minimum: i32,
    pub maximum: i32,
    pub fuzz: i32,
    pub flat: i32,
    pub resolution: i32,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct TimeVal {
    pub tv_sec: libc::time_t,
    pub tv_usec: libc::suseconds_t,
}

#[repr(C)]
#[derive(Default, Debug, Copy, Clone)]
pub struct InputEvent {
    pub time: TimeVal,
    pub type_: u16,
    pub code: u16,
    pub value: i32,
}

struct GamePad {
    fd: libc::c_int,
    info: ControllerInfo,
    state: ControllerState,
    axis_map: [i32; ABS_CNT as usize],
    axis_info: [InputAbsinfo; ABS_CNT as usize],
    buttons_map: [usize; (KEY_CNT - BTN_MISC) as usize],
}

impl GamePad {
    unsafe fn poll_abs_info(&mut self) {
        for code in &self.axis_map {
            if *code != -1 {
                if libc::ioctl(
                    self.fd,
                    eviocgabs(*code as _),
                    &mut self.axis_info[*code as usize],
                ) < 0
                {
                    continue;
                }
            }
        }
    }

    unsafe fn poll(&mut self) {
        let mut e = InputEvent::default();

        if libc::read(
            self.fd,
            &mut e as *mut _ as *mut _,
            std::mem::size_of_val(&e),
        ) < 0
        {
            // handle disconnect
            return;
        }

        if e.type_ == EV_KEY as _ {
            let code = e.code as usize - BTN_MISC as usize;
            self.state.digital_state[self.buttons_map[code]] = e.value != 0;
        }
        if e.type_ == EV_ABS as _ {
            let info = self.axis_info[e.code as usize];
            let value = if e.code >= ABS_HAT0X as _ && e.code <= ABS_HAT3Y as _ {
                e.value as f32
            } else {
                ((e.value as f32 - info.minimum as f32)
                    / (info.maximum as f32 - info.minimum as f32)
                    - 0.5)
                    * 2.
            };
            self.state.analog_state[self.axis_map[e.code as usize] as usize] = value;
        }
    }
}

unsafe fn open_joystick_device(path: PathBuf) -> Option<GamePad> {
    use std::os::unix::ffi::OsStrExt;

    let fd = libc::open(
        path.as_os_str().as_bytes().as_ptr() as _,
        libc::O_RDONLY | libc::O_NONBLOCK,
    );
    if fd == -1 {
        return None;
    }

    let mut ev_bits: [u8; (EV_CNT as usize + 7) / 8] = [0; (EV_CNT as usize + 7) / 8];
    let mut key_bits: [u8; (KEY_CNT as usize + 7) / 8] = [0; (KEY_CNT as usize + 7) / 8];
    let mut abs_bits: [u8; (ABS_CNT as usize + 7) / 8] = [0; (ABS_CNT as usize + 7) / 8];

    // EVIOCGBIT(0, sizeof(evBits))
    let eviocgbit_0: u64 = 2147763488;

    // EVIOCGBIT(EV_KEY, sizeof(keyBits))
    let eviocgbit_ev_key: u64 = 2153792801;

    // ioctls::eviocgbit(EV_ABS, sizeof(absBits))
    let eviocgbit_ev_abs: u64 = 2148025635;

    // EVIOCGID
    let eviocgid: u64 = 2148025602;

    let mut id: InputId = InputId::default();

    if libc::ioctl(fd, eviocgbit_0, ev_bits.as_mut_ptr()) < 0
        || libc::ioctl(fd, eviocgbit_ev_key, key_bits.as_mut_ptr()) < 0
        || libc::ioctl(fd, eviocgbit_ev_abs, abs_bits.as_mut_ptr()) < 0
        || libc::ioctl(fd, eviocgid, &mut id as *mut _) < 0
    {
        libc::close(fd);
        println!("ioctl failed, bad");
        return None;
    }

    // Ensure this device supports the events expected of a joystick
    if !is_bit_set(EV_KEY as _, &ev_bits) || !is_bit_set(EV_ABS as _, &ev_bits) {
        libc::close(fd);
        return None;
    }

    // Retrieve joystick name
    let mut name: [u8; 256] = [0; 256];
    // EVIOCGNAME(256)
    let eviocgname: u64 = 2164278534;
    let name = if libc::ioctl(fd, eviocgname, name.as_mut_ptr()) >= 0 {
        std::ffi::CStr::from_ptr(name.as_ptr() as *const _)
            .to_string_lossy()
            .into_owned()
    } else {
        "Unknwon".to_string()
    };
    println!("Found gamepad {:?}: {:?}", path, name);
    println!("input_id: {:?}", id);

    let mut digital_count = 0;
    let mut buttons_map = [0; (KEY_CNT - BTN_MISC) as usize];
    for code in BTN_MISC..KEY_CNT {
        if !is_bit_set(code as _, &key_bits) {
            continue;
        }

        buttons_map[(code - BTN_MISC) as usize] = digital_count;
        digital_count += 1;
    }

    let mut analog_count = 0;
    let mut axis_map = [-1; ABS_CNT as usize];
    let mut axis_info = [InputAbsinfo::default(); ABS_CNT as usize];

    for code in 0..ABS_CNT {
        if !is_bit_set(code as _, &abs_bits) {
            continue;
        }

        if code >= ABS_HAT0X && code <= ABS_HAT3Y {
            axis_map[code as usize] = analog_count as i32;
            analog_count += 1;
        } else {
            if libc::ioctl(fd, eviocgabs(code as _), &mut axis_info[code as usize]) < 0 {
                continue;
            }
            axis_map[code as usize] = analog_count as i32;
            analog_count += 1;
        }
    }

    let mut gamepad = GamePad {
        fd,
        info: ControllerInfo {
            name,
            digital_count,
            analog_count,
        },
        state: ControllerState::new(),
        axis_info,
        axis_map,
        buttons_map,
    };
    gamepad.state.status = ControllerStatus::Connected;
    gamepad.poll_abs_info();

    Some(gamepad)
}

unsafe fn platform_init_joysticks() -> Vec<GamePad> {
    let dirname = "/dev/input";

    let mut res = vec![];

    for entry in std::fs::read_dir(dirname).unwrap() {
        let path = entry.unwrap().path();
        let file_name = path.file_name().unwrap().to_str().unwrap();

        if file_name.starts_with("event") {
            if let Some(gamepad) = open_joystick_device(path) {
                res.push(gamepad);
            }
        }
    }
    res
}

pub struct ControllerContext {
    gamepads: Vec<GamePad>,
}

impl ControllerContext {
    pub fn new() -> Option<Self> {
        Some(ControllerContext {
            gamepads: unsafe { platform_init_joysticks() },
        })
    }

    /// Update controller state by index
    pub fn update(&mut self, index: usize) {
        if let Some(ref mut gamepad) = self.gamepads.get_mut(index) {
            unsafe {
                gamepad.poll();
            }
        }
    }

    pub fn info(&self, index: usize) -> &ControllerInfo {
        if let Some(ref gamepad) = self.gamepads.get(index) {
            &gamepad.info
        } else {
            &*DEFAULT_CONTROLLER_INFO
        }
    }
    pub fn state(&self, index: usize) -> &ControllerState {
        if let Some(ref gamepad) = self.gamepads.get(index) {
            &gamepad.state
        } else {
            &DEFAULT_CONTROLLER_STATE
        }
    }
}
