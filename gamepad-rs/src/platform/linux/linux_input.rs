#![allow(dead_code)]

use libc::c_int;

// https://docs.rs/input-linux-sys/0.6.0/src/input_linux_sys/events.rs.html
pub const EV_SYN: c_int = 0x00;
pub const EV_KEY: c_int = 0x01;
pub const EV_REL: c_int = 0x02;
pub const EV_ABS: c_int = 0x03;
pub const EV_MSC: c_int = 0x04;
pub const EV_SW: c_int = 0x05;
pub const EV_LED: c_int = 0x11;
pub const EV_SND: c_int = 0x12;
pub const EV_REP: c_int = 0x14;
pub const EV_FF: c_int = 0x15;
pub const EV_PWR: c_int = 0x16;
pub const EV_FF_STATUS: c_int = 0x17;
pub const EV_MAX: c_int = 0x1f;
pub const EV_CNT: c_int = EV_MAX + 1;

pub const KEY_MAX: c_int = 0x2ff;
pub const KEY_CNT: c_int = KEY_MAX + 1;

pub const ABS_MAX: c_int = 0x3f;
pub const ABS_CNT: c_int = ABS_MAX + 1;

pub const BTN_MISC: c_int = 0x100;
pub const BTN_0: c_int = 0x100;
pub const BTN_1: c_int = 0x101;
pub const BTN_2: c_int = 0x102;
pub const BTN_3: c_int = 0x103;
pub const BTN_4: c_int = 0x104;
pub const BTN_5: c_int = 0x105;
pub const BTN_6: c_int = 0x106;
pub const BTN_7: c_int = 0x107;
pub const BTN_8: c_int = 0x108;
pub const BTN_9: c_int = 0x109;

pub const BTN_MOUSE: c_int = 0x110;
pub const BTN_LEFT: c_int = 0x110;
pub const BTN_RIGHT: c_int = 0x111;
pub const BTN_MIDDLE: c_int = 0x112;
pub const BTN_SIDE: c_int = 0x113;
pub const BTN_EXTRA: c_int = 0x114;
pub const BTN_FORWARD: c_int = 0x115;
pub const BTN_BACK: c_int = 0x116;
pub const BTN_TASK: c_int = 0x117;

pub const BTN_JOYSTICK: c_int = 0x120;
pub const BTN_TRIGGER: c_int = 0x120;
pub const BTN_THUMB: c_int = 0x121;
pub const BTN_THUMB2: c_int = 0x122;
pub const BTN_TOP: c_int = 0x123;
pub const BTN_TOP2: c_int = 0x124;
pub const BTN_PINKIE: c_int = 0x125;
pub const BTN_BASE: c_int = 0x126;
pub const BTN_BASE2: c_int = 0x127;
pub const BTN_BASE3: c_int = 0x128;
pub const BTN_BASE4: c_int = 0x129;
pub const BTN_BASE5: c_int = 0x12a;
pub const BTN_BASE6: c_int = 0x12b;
pub const BTN_DEAD: c_int = 0x12f;

pub const BTN_GAMEPAD: c_int = 0x130;
pub const BTN_SOUTH: c_int = 0x130;
pub const BTN_A: c_int = BTN_SOUTH;
pub const BTN_EAST: c_int = 0x131;
pub const BTN_B: c_int = BTN_EAST;
pub const BTN_C: c_int = 0x132;
pub const BTN_NORTH: c_int = 0x133;
pub const BTN_X: c_int = BTN_NORTH;
pub const BTN_WEST: c_int = 0x134;
pub const BTN_Y: c_int = BTN_WEST;
pub const BTN_Z: c_int = 0x135;
pub const BTN_TL: c_int = 0x136;
pub const BTN_TR: c_int = 0x137;
pub const BTN_TL2: c_int = 0x138;
pub const BTN_TR2: c_int = 0x139;
pub const BTN_SELECT: c_int = 0x13a;
pub const BTN_START: c_int = 0x13b;
pub const BTN_MODE: c_int = 0x13c;
pub const BTN_THUMBL: c_int = 0x13d;
pub const BTN_THUMBR: c_int = 0x13e;

pub const BTN_DIGI: c_int = 0x140;
pub const BTN_TOOL_PEN: c_int = 0x140;
pub const BTN_TOOL_RUBBER: c_int = 0x141;
pub const BTN_TOOL_BRUSH: c_int = 0x142;
pub const BTN_TOOL_PENCIL: c_int = 0x143;
pub const BTN_TOOL_AIRBRUSH: c_int = 0x144;
pub const BTN_TOOL_FINGER: c_int = 0x145;
pub const BTN_TOOL_MOUSE: c_int = 0x146;
pub const BTN_TOOL_LENS: c_int = 0x147;
pub const BTN_TOOL_QUINTTAP: c_int = 0x148; /* Five fingers on trackpad */
pub const BTN_STYLUS3: c_int = 0x149;
pub const BTN_TOUCH: c_int = 0x14a;
pub const BTN_STYLUS: c_int = 0x14b;
pub const BTN_STYLUS2: c_int = 0x14c;
pub const BTN_TOOL_DOUBLETAP: c_int = 0x14d;
pub const BTN_TOOL_TRIPLETAP: c_int = 0x14e;
pub const BTN_TOOL_QUADTAP: c_int = 0x14f; /* Four fingers on trackpad */

pub const BTN_WHEEL: c_int = 0x150;
pub const BTN_GEAR_DOWN: c_int = 0x150;
pub const BTN_GEAR_UP: c_int = 0x151;
/*
 * Absolute axes
 */

pub const ABS_X: c_int = 0x00;
pub const ABS_Y: c_int = 0x01;
pub const ABS_Z: c_int = 0x02;
pub const ABS_RX: c_int = 0x03;
pub const ABS_RY: c_int = 0x04;
pub const ABS_RZ: c_int = 0x05;
pub const ABS_THROTTLE: c_int = 0x06;
pub const ABS_RUDDER: c_int = 0x07;
pub const ABS_WHEEL: c_int = 0x08;
pub const ABS_GAS: c_int = 0x09;
pub const ABS_BRAKE: c_int = 0x0a;
pub const ABS_HAT0X: c_int = 0x10;
pub const ABS_HAT0Y: c_int = 0x11;
pub const ABS_HAT1X: c_int = 0x12;
pub const ABS_HAT1Y: c_int = 0x13;
pub const ABS_HAT2X: c_int = 0x14;
pub const ABS_HAT2Y: c_int = 0x15;
pub const ABS_HAT3X: c_int = 0x16;
pub const ABS_HAT3Y: c_int = 0x17;
pub const ABS_PRESSURE: c_int = 0x18;
pub const ABS_DISTANCE: c_int = 0x19;
pub const ABS_TILT_X: c_int = 0x1a;
pub const ABS_TILT_Y: c_int = 0x1b;
pub const ABS_TOOL_WIDTH: c_int = 0x1c;

pub const ABS_VOLUME: c_int = 0x20;

pub const ABS_MISC: c_int = 0x28;

/*
 * 0x2e is reserved and should not be used in input drivers.
 * It was used by HID as ABS_MISC+6 and userspace needs to detect if
 * the next ABS_* event is correct or is just ABS_MISC + n.
 * We define here ABS_RESERVED so userspace can rely on it and detect
 * the situation described above.
 */
pub const ABS_RESERVED: c_int = 0x2e;

pub const ABS_MT_SLOT: c_int = 0x2f; /* MT slot being modified */
pub const ABS_MT_TOUCH_MAJOR: c_int = 0x30; /* Major axis of touching ellipse */
pub const ABS_MT_TOUCH_MINOR: c_int = 0x31; /* Minor axis : c_int = omit if circular */
pub const ABS_MT_WIDTH_MAJOR: c_int = 0x32; /* Major axis of approaching ellipse */
pub const ABS_MT_WIDTH_MINOR: c_int = 0x33; /* Minor axis : c_int = omit if circular */
pub const ABS_MT_ORIENTATION: c_int = 0x34; /* Ellipse orientation */
pub const ABS_MT_POSITION_X: c_int = 0x35; /* Center X touch position */
pub const ABS_MT_POSITION_Y: c_int = 0x36; /* Center Y touch position */
pub const ABS_MT_TOOL_TYPE: c_int = 0x37; /* Type of touching device */
pub const ABS_MT_BLOB_ID: c_int = 0x38; /* Group a set of packets as a blob */
pub const ABS_MT_TRACKING_ID: c_int = 0x39; /* Unique ID of initiated contact */
pub const ABS_MT_PRESSURE: c_int = 0x3a; /* Pressure on contact area */
pub const ABS_MT_DISTANCE: c_int = 0x3b; /* Contact hover distance */
pub const ABS_MT_TOOL_X: c_int = 0x3c; /* Center X tool position */
pub const ABS_MT_TOOL_Y: c_int = 0x3d;
