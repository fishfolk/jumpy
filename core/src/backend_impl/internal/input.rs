use std::fmt::Debug;
use winit::event::{MouseScrollDelta, VirtualKeyCode};
use winit_input_helper::WinitInputHelper;

use crate::event::Event;
use crate::input::{Button, InputMapping, KeyCode, MouseButton};
use crate::math::{vec2, Vec2};

static mut INPUT_MAPPING: Option<InputMapping> = None;

pub(crate) fn input_mapping() -> &'static InputMapping {
    unsafe { INPUT_MAPPING.get_or_insert_with(InputMapping::default) }
}

pub(crate) fn apply_input_config(config: &InputMapping) {
    unsafe { INPUT_MAPPING = Some(config.clone()) }
}

static mut INPUT: Option<WinitInputHelper> = None;

fn input() -> &'static mut WinitInputHelper {
    unsafe { INPUT.get_or_insert_with(WinitInputHelper::new) }
}

static mut LAST_MOUSE_POSITION: Vec2 = Vec2::ZERO;

fn last_mouse_position() -> Vec2 {
    unsafe { LAST_MOUSE_POSITION }
}

const SCROLL_LINE_HEIGHT: u32 = 8;

static mut MOUSE_WHEEL: Vec2 = Vec2::ZERO;

pub fn input_event_handler<E: 'static + Debug>(event: &winit::event::Event<Event<E>>) -> bool {
    match event {
        winit::event::Event::NewEvents(..) => unsafe { MOUSE_WHEEL = Vec2::ZERO },
        winit::event::Event::WindowEvent { event, .. } => {
            if let winit::event::WindowEvent::MouseWheel { delta, .. } = event {
                match *delta {
                    MouseScrollDelta::LineDelta(x, y) => unsafe {
                        MOUSE_WHEEL.x = x;
                        MOUSE_WHEEL.y = y;
                    },
                    MouseScrollDelta::PixelDelta(position) => unsafe {
                        MOUSE_WHEEL.x = position.x as f32;
                        MOUSE_WHEEL.y = position.y as f32;
                    },
                }
            }
        }
        _ => {}
    }

    let res = input().update(event);

    if let Some((x, y)) = input().mouse() {
        unsafe {
            LAST_MOUSE_POSITION.x = x;
            LAST_MOUSE_POSITION.y = y;
        }
    }

    res
}

pub fn is_key_down(key: KeyCode) -> bool {
    input().key_held(key.into())
}

pub fn is_key_pressed(key: KeyCode) -> bool {
    input().key_pressed(key.into())
}

pub fn is_key_released(key: KeyCode) -> bool {
    input().key_released(key.into())
}

pub fn is_mouse_button_down(button: MouseButton) -> bool {
    input().mouse_pressed(button.into())
}

pub fn is_mouse_button_pressed(button: MouseButton) -> bool {
    input().mouse_held(button.into())
}

pub fn is_mouse_button_released(button: MouseButton) -> bool {
    input().mouse_released(button.into())
}

pub fn mouse_position() -> Vec2 {
    input()
        .mouse()
        .map(|(x, y)| vec2(x, y))
        .unwrap_or_else(|| last_mouse_position())
}

pub fn mouse_movement() -> Vec2 {
    let (x, y) = input().mouse_diff();
    vec2(x, y)
}

pub fn mouse_wheel() -> Vec2 {
    unsafe { MOUSE_WHEEL }
}

impl From<KeyCode> for VirtualKeyCode {
    fn from(key_code: KeyCode) -> Self {
        match key_code {
            KeyCode::Space => VirtualKeyCode::Space,
            KeyCode::Apostrophe => VirtualKeyCode::Apostrophe,
            KeyCode::Comma => VirtualKeyCode::Comma,
            KeyCode::Minus => VirtualKeyCode::Minus,
            KeyCode::Period => VirtualKeyCode::Period,
            KeyCode::Slash => VirtualKeyCode::Slash,
            KeyCode::Key0 => VirtualKeyCode::Key0,
            KeyCode::Key1 => VirtualKeyCode::Key1,
            KeyCode::Key2 => VirtualKeyCode::Key2,
            KeyCode::Key3 => VirtualKeyCode::Key3,
            KeyCode::Key4 => VirtualKeyCode::Key4,
            KeyCode::Key5 => VirtualKeyCode::Key5,
            KeyCode::Key6 => VirtualKeyCode::Key6,
            KeyCode::Key7 => VirtualKeyCode::Key7,
            KeyCode::Key8 => VirtualKeyCode::Key8,
            KeyCode::Key9 => VirtualKeyCode::Key9,
            KeyCode::Semicolon => VirtualKeyCode::Semicolon,
            KeyCode::Equal => VirtualKeyCode::Equals,
            KeyCode::A => VirtualKeyCode::A,
            KeyCode::B => VirtualKeyCode::B,
            KeyCode::C => VirtualKeyCode::C,
            KeyCode::D => VirtualKeyCode::D,
            KeyCode::E => VirtualKeyCode::E,
            KeyCode::F => VirtualKeyCode::F,
            KeyCode::G => VirtualKeyCode::G,
            KeyCode::H => VirtualKeyCode::H,
            KeyCode::I => VirtualKeyCode::I,
            KeyCode::J => VirtualKeyCode::J,
            KeyCode::K => VirtualKeyCode::K,
            KeyCode::L => VirtualKeyCode::L,
            KeyCode::M => VirtualKeyCode::M,
            KeyCode::N => VirtualKeyCode::N,
            KeyCode::O => VirtualKeyCode::O,
            KeyCode::P => VirtualKeyCode::P,
            KeyCode::Q => VirtualKeyCode::Q,
            KeyCode::R => VirtualKeyCode::R,
            KeyCode::S => VirtualKeyCode::S,
            KeyCode::T => VirtualKeyCode::T,
            KeyCode::U => VirtualKeyCode::U,
            KeyCode::V => VirtualKeyCode::V,
            KeyCode::W => VirtualKeyCode::W,
            KeyCode::X => VirtualKeyCode::X,
            KeyCode::Y => VirtualKeyCode::Y,
            KeyCode::Z => VirtualKeyCode::Z,
            KeyCode::LeftBracket => VirtualKeyCode::LBracket,
            KeyCode::Backslash => VirtualKeyCode::Backslash,
            KeyCode::RightBracket => VirtualKeyCode::RBracket,
            KeyCode::GraveAccent => VirtualKeyCode::Grave,
            KeyCode::Escape => VirtualKeyCode::Escape,
            KeyCode::Enter => VirtualKeyCode::Return,
            KeyCode::Tab => VirtualKeyCode::Tab,
            KeyCode::Backspace => VirtualKeyCode::Back,
            KeyCode::Insert => VirtualKeyCode::Insert,
            KeyCode::Delete => VirtualKeyCode::Delete,
            KeyCode::Right => VirtualKeyCode::Right,
            KeyCode::Left => VirtualKeyCode::Left,
            KeyCode::Down => VirtualKeyCode::Down,
            KeyCode::Up => VirtualKeyCode::Up,
            KeyCode::PageUp => VirtualKeyCode::PageUp,
            KeyCode::PageDown => VirtualKeyCode::PageDown,
            KeyCode::Home => VirtualKeyCode::Home,
            KeyCode::End => VirtualKeyCode::End,
            KeyCode::ScrollLock => VirtualKeyCode::Scroll,
            KeyCode::NumLock => VirtualKeyCode::Numlock,
            KeyCode::PrintScreen => VirtualKeyCode::Snapshot,
            KeyCode::Pause => VirtualKeyCode::Pause,
            KeyCode::F1 => VirtualKeyCode::F1,
            KeyCode::F2 => VirtualKeyCode::F2,
            KeyCode::F3 => VirtualKeyCode::F3,
            KeyCode::F4 => VirtualKeyCode::F4,
            KeyCode::F5 => VirtualKeyCode::F5,
            KeyCode::F6 => VirtualKeyCode::F6,
            KeyCode::F7 => VirtualKeyCode::F7,
            KeyCode::F8 => VirtualKeyCode::F8,
            KeyCode::F9 => VirtualKeyCode::F9,
            KeyCode::F10 => VirtualKeyCode::F10,
            KeyCode::F11 => VirtualKeyCode::F11,
            KeyCode::F12 => VirtualKeyCode::F12,
            KeyCode::F13 => VirtualKeyCode::F13,
            KeyCode::F14 => VirtualKeyCode::F14,
            KeyCode::F15 => VirtualKeyCode::F15,
            KeyCode::F16 => VirtualKeyCode::F16,
            KeyCode::F17 => VirtualKeyCode::F17,
            KeyCode::F18 => VirtualKeyCode::F18,
            KeyCode::F19 => VirtualKeyCode::F19,
            KeyCode::F20 => VirtualKeyCode::F20,
            KeyCode::F21 => VirtualKeyCode::F21,
            KeyCode::F22 => VirtualKeyCode::F22,
            KeyCode::F23 => VirtualKeyCode::F23,
            KeyCode::F24 => VirtualKeyCode::F24,
            KeyCode::Kp0 => VirtualKeyCode::Numpad0,
            KeyCode::Kp1 => VirtualKeyCode::Numpad1,
            KeyCode::Kp2 => VirtualKeyCode::Numpad2,
            KeyCode::Kp3 => VirtualKeyCode::Numpad3,
            KeyCode::Kp4 => VirtualKeyCode::Numpad4,
            KeyCode::Kp5 => VirtualKeyCode::Numpad5,
            KeyCode::Kp6 => VirtualKeyCode::Numpad6,
            KeyCode::Kp7 => VirtualKeyCode::Numpad7,
            KeyCode::Kp8 => VirtualKeyCode::Numpad8,
            KeyCode::Kp9 => VirtualKeyCode::Numpad9,
            KeyCode::KpDecimal => VirtualKeyCode::NumpadDecimal,
            KeyCode::KpDivide => VirtualKeyCode::NumpadDivide,
            KeyCode::KpMultiply => VirtualKeyCode::NumpadMultiply,
            KeyCode::KpSubtract => VirtualKeyCode::NumpadSubtract,
            KeyCode::KpAdd => VirtualKeyCode::NumpadAdd,
            KeyCode::KpEnter => VirtualKeyCode::NumpadEnter,
            KeyCode::KpEqual => VirtualKeyCode::NumpadEquals,
            KeyCode::LeftShift => VirtualKeyCode::LShift,
            KeyCode::LeftControl => VirtualKeyCode::LControl,
            KeyCode::LeftAlt => VirtualKeyCode::LAlt,
            KeyCode::LeftSuper => VirtualKeyCode::LWin,
            KeyCode::RightShift => VirtualKeyCode::RShift,
            KeyCode::RightControl => VirtualKeyCode::RControl,
            KeyCode::RightAlt => VirtualKeyCode::RAlt,
            KeyCode::RightSuper => VirtualKeyCode::RWin,
            _ => VirtualKeyCode::Unlabeled,
        }
    }
}

impl From<VirtualKeyCode> for KeyCode {
    fn from(key_code: VirtualKeyCode) -> Self {
        match key_code {
            VirtualKeyCode::Space => KeyCode::Space,
            VirtualKeyCode::Apostrophe => KeyCode::Apostrophe,
            VirtualKeyCode::Comma => KeyCode::Comma,
            VirtualKeyCode::Minus => KeyCode::Minus,
            VirtualKeyCode::Period => KeyCode::Period,
            VirtualKeyCode::Slash => KeyCode::Slash,
            VirtualKeyCode::Key0 => KeyCode::Key0,
            VirtualKeyCode::Key1 => KeyCode::Key1,
            VirtualKeyCode::Key2 => KeyCode::Key2,
            VirtualKeyCode::Key3 => KeyCode::Key3,
            VirtualKeyCode::Key4 => KeyCode::Key4,
            VirtualKeyCode::Key5 => KeyCode::Key5,
            VirtualKeyCode::Key6 => KeyCode::Key6,
            VirtualKeyCode::Key7 => KeyCode::Key7,
            VirtualKeyCode::Key8 => KeyCode::Key8,
            VirtualKeyCode::Key9 => KeyCode::Key9,
            VirtualKeyCode::Semicolon => KeyCode::Semicolon,
            VirtualKeyCode::Equals => KeyCode::Equal,
            VirtualKeyCode::A => KeyCode::A,
            VirtualKeyCode::B => KeyCode::B,
            VirtualKeyCode::C => KeyCode::C,
            VirtualKeyCode::D => KeyCode::D,
            VirtualKeyCode::E => KeyCode::E,
            VirtualKeyCode::F => KeyCode::F,
            VirtualKeyCode::G => KeyCode::G,
            VirtualKeyCode::H => KeyCode::H,
            VirtualKeyCode::I => KeyCode::I,
            VirtualKeyCode::J => KeyCode::J,
            VirtualKeyCode::K => KeyCode::K,
            VirtualKeyCode::L => KeyCode::L,
            VirtualKeyCode::M => KeyCode::M,
            VirtualKeyCode::N => KeyCode::N,
            VirtualKeyCode::O => KeyCode::O,
            VirtualKeyCode::P => KeyCode::P,
            VirtualKeyCode::Q => KeyCode::Q,
            VirtualKeyCode::R => KeyCode::R,
            VirtualKeyCode::S => KeyCode::S,
            VirtualKeyCode::T => KeyCode::T,
            VirtualKeyCode::U => KeyCode::U,
            VirtualKeyCode::V => KeyCode::V,
            VirtualKeyCode::W => KeyCode::W,
            VirtualKeyCode::X => KeyCode::X,
            VirtualKeyCode::Y => KeyCode::Y,
            VirtualKeyCode::Z => KeyCode::Z,
            VirtualKeyCode::LBracket => KeyCode::LeftBracket,
            VirtualKeyCode::Backslash => KeyCode::Backslash,
            VirtualKeyCode::RBracket => KeyCode::RightBracket,
            VirtualKeyCode::Grave => KeyCode::GraveAccent,
            VirtualKeyCode::Escape => KeyCode::Escape,
            VirtualKeyCode::Return => KeyCode::Enter,
            VirtualKeyCode::Tab => KeyCode::Tab,
            VirtualKeyCode::Back => KeyCode::Backspace,
            VirtualKeyCode::Insert => KeyCode::Insert,
            VirtualKeyCode::Delete => KeyCode::Delete,
            VirtualKeyCode::Right => KeyCode::Right,
            VirtualKeyCode::Left => KeyCode::Left,
            VirtualKeyCode::Down => KeyCode::Down,
            VirtualKeyCode::Up => KeyCode::Up,
            VirtualKeyCode::PageUp => KeyCode::PageUp,
            VirtualKeyCode::PageDown => KeyCode::PageDown,
            VirtualKeyCode::Home => KeyCode::Home,
            VirtualKeyCode::End => KeyCode::End,
            VirtualKeyCode::Scroll => KeyCode::ScrollLock,
            VirtualKeyCode::Numlock => KeyCode::NumLock,
            VirtualKeyCode::Snapshot => KeyCode::PrintScreen,
            VirtualKeyCode::Pause => KeyCode::Pause,
            VirtualKeyCode::F1 => KeyCode::F1,
            VirtualKeyCode::F2 => KeyCode::F2,
            VirtualKeyCode::F3 => KeyCode::F3,
            VirtualKeyCode::F4 => KeyCode::F4,
            VirtualKeyCode::F5 => KeyCode::F5,
            VirtualKeyCode::F6 => KeyCode::F6,
            VirtualKeyCode::F7 => KeyCode::F7,
            VirtualKeyCode::F8 => KeyCode::F8,
            VirtualKeyCode::F9 => KeyCode::F9,
            VirtualKeyCode::F10 => KeyCode::F10,
            VirtualKeyCode::F11 => KeyCode::F11,
            VirtualKeyCode::F12 => KeyCode::F12,
            VirtualKeyCode::F13 => KeyCode::F13,
            VirtualKeyCode::F14 => KeyCode::F14,
            VirtualKeyCode::F15 => KeyCode::F15,
            VirtualKeyCode::F16 => KeyCode::F16,
            VirtualKeyCode::F17 => KeyCode::F17,
            VirtualKeyCode::F18 => KeyCode::F18,
            VirtualKeyCode::F19 => KeyCode::F19,
            VirtualKeyCode::F20 => KeyCode::F20,
            VirtualKeyCode::F21 => KeyCode::F21,
            VirtualKeyCode::F22 => KeyCode::F22,
            VirtualKeyCode::F23 => KeyCode::F23,
            VirtualKeyCode::F24 => KeyCode::F24,
            VirtualKeyCode::Numpad0 => KeyCode::Kp0,
            VirtualKeyCode::Numpad1 => KeyCode::Kp1,
            VirtualKeyCode::Numpad2 => KeyCode::Kp2,
            VirtualKeyCode::Numpad3 => KeyCode::Kp3,
            VirtualKeyCode::Numpad4 => KeyCode::Kp4,
            VirtualKeyCode::Numpad5 => KeyCode::Kp5,
            VirtualKeyCode::Numpad6 => KeyCode::Kp6,
            VirtualKeyCode::Numpad7 => KeyCode::Kp7,
            VirtualKeyCode::Numpad8 => KeyCode::Kp8,
            VirtualKeyCode::Numpad9 => KeyCode::Kp9,
            VirtualKeyCode::NumpadDecimal => KeyCode::KpDecimal,
            VirtualKeyCode::NumpadDivide => KeyCode::KpDivide,
            VirtualKeyCode::NumpadMultiply => KeyCode::KpMultiply,
            VirtualKeyCode::NumpadSubtract => KeyCode::KpSubtract,
            VirtualKeyCode::NumpadAdd => KeyCode::KpAdd,
            VirtualKeyCode::NumpadEnter => KeyCode::KpEnter,
            VirtualKeyCode::NumpadEquals => KeyCode::KpEqual,
            VirtualKeyCode::LShift => KeyCode::LeftShift,
            VirtualKeyCode::LControl => KeyCode::LeftControl,
            VirtualKeyCode::LAlt => KeyCode::LeftAlt,
            VirtualKeyCode::LWin => KeyCode::LeftSuper,
            VirtualKeyCode::RShift => KeyCode::RightShift,
            VirtualKeyCode::RControl => KeyCode::RightControl,
            VirtualKeyCode::RAlt => KeyCode::RightAlt,
            VirtualKeyCode::RWin => KeyCode::RightSuper,
            _ => KeyCode::Unknown,
        }
    }
}

impl From<MouseButton> for usize {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => 0,
            MouseButton::Right => 1,
            MouseButton::Middle => 2,
            MouseButton::Unknown(i) => i,
        }
    }
}

impl From<usize> for MouseButton {
    fn from(i: usize) -> Self {
        match i {
            0 => MouseButton::Left,
            1 => MouseButton::Right,
            2 => MouseButton::Middle,
            _ => MouseButton::Unknown(i),
        }
    }
}
