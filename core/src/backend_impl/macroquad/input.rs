use crate::config::config;
use crate::input::{InputMapping, KeyCode, MouseButton};
use crate::math::{vec2, Vec2};
use std::borrow::Borrow;

pub(crate) fn input_mapping() -> &'static InputMapping {
    config().input.borrow()
}

pub fn is_key_down(key_code: KeyCode) -> bool {
    macroquad::input::is_key_down(key_code.into())
}

pub fn is_key_pressed(key_code: KeyCode) -> bool {
    macroquad::input::is_key_pressed(key_code.into())
}

pub fn is_key_released(key_code: KeyCode) -> bool {
    macroquad::input::is_key_released(key_code.into())
}

pub fn is_mouse_button_down(button: MouseButton) -> bool {
    macroquad::input::is_mouse_button_down(button.into())
}

pub fn is_mouse_button_pressed(button: MouseButton) -> bool {
    macroquad::input::is_mouse_button_pressed(button.into())
}

pub fn is_mouse_button_released(button: MouseButton) -> bool {
    macroquad::input::is_mouse_button_released(button.into())
}

pub fn mouse_position() -> Vec2 {
    let (x, y) = macroquad::input::mouse_position();
    vec2(x, y)
}

pub fn mouse_wheel() -> Vec2 {
    let (x, y) = macroquad::input::mouse_wheel();
    vec2(x, y)
}

impl From<macroquad::input::KeyCode> for KeyCode {
    fn from(keycode: macroquad::input::KeyCode) -> Self {
        match keycode {
            macroquad::input::KeyCode::Space => Self::Space,
            macroquad::input::KeyCode::Apostrophe => Self::Apostrophe,
            macroquad::input::KeyCode::Comma => Self::Comma,
            macroquad::input::KeyCode::Minus => Self::Minus,
            macroquad::input::KeyCode::Period => Self::Period,
            macroquad::input::KeyCode::Slash => Self::Slash,
            macroquad::input::KeyCode::Key0 => Self::Key0,
            macroquad::input::KeyCode::Key1 => Self::Key1,
            macroquad::input::KeyCode::Key2 => Self::Key2,
            macroquad::input::KeyCode::Key3 => Self::Key3,
            macroquad::input::KeyCode::Key4 => Self::Key4,
            macroquad::input::KeyCode::Key5 => Self::Key5,
            macroquad::input::KeyCode::Key6 => Self::Key6,
            macroquad::input::KeyCode::Key7 => Self::Key7,
            macroquad::input::KeyCode::Key8 => Self::Key8,
            macroquad::input::KeyCode::Key9 => Self::Key9,
            macroquad::input::KeyCode::Semicolon => Self::Semicolon,
            macroquad::input::KeyCode::Equal => Self::Equal,
            macroquad::input::KeyCode::A => Self::A,
            macroquad::input::KeyCode::B => Self::B,
            macroquad::input::KeyCode::C => Self::C,
            macroquad::input::KeyCode::D => Self::D,
            macroquad::input::KeyCode::E => Self::E,
            macroquad::input::KeyCode::F => Self::F,
            macroquad::input::KeyCode::G => Self::G,
            macroquad::input::KeyCode::H => Self::H,
            macroquad::input::KeyCode::I => Self::I,
            macroquad::input::KeyCode::J => Self::J,
            macroquad::input::KeyCode::K => Self::K,
            macroquad::input::KeyCode::L => Self::L,
            macroquad::input::KeyCode::M => Self::M,
            macroquad::input::KeyCode::N => Self::N,
            macroquad::input::KeyCode::O => Self::O,
            macroquad::input::KeyCode::P => Self::P,
            macroquad::input::KeyCode::Q => Self::Q,
            macroquad::input::KeyCode::R => Self::R,
            macroquad::input::KeyCode::S => Self::S,
            macroquad::input::KeyCode::T => Self::T,
            macroquad::input::KeyCode::U => Self::U,
            macroquad::input::KeyCode::V => Self::V,
            macroquad::input::KeyCode::W => Self::W,
            macroquad::input::KeyCode::X => Self::X,
            macroquad::input::KeyCode::Y => Self::Y,
            macroquad::input::KeyCode::Z => Self::Z,
            macroquad::input::KeyCode::LeftBracket => Self::LeftBracket,
            macroquad::input::KeyCode::Backslash => Self::Backslash,
            macroquad::input::KeyCode::RightBracket => Self::RightBracket,
            macroquad::input::KeyCode::GraveAccent => Self::GraveAccent,
            macroquad::input::KeyCode::World1 => Self::World1,
            macroquad::input::KeyCode::World2 => Self::World2,
            macroquad::input::KeyCode::Escape => Self::Escape,
            macroquad::input::KeyCode::Enter => Self::Enter,
            macroquad::input::KeyCode::Tab => Self::Tab,
            macroquad::input::KeyCode::Backspace => Self::Backspace,
            macroquad::input::KeyCode::Insert => Self::Insert,
            macroquad::input::KeyCode::Delete => Self::Delete,
            macroquad::input::KeyCode::Right => Self::Right,
            macroquad::input::KeyCode::Left => Self::Left,
            macroquad::input::KeyCode::Down => Self::Down,
            macroquad::input::KeyCode::Up => Self::Up,
            macroquad::input::KeyCode::PageUp => Self::PageUp,
            macroquad::input::KeyCode::PageDown => Self::PageDown,
            macroquad::input::KeyCode::Home => Self::Home,
            macroquad::input::KeyCode::End => Self::End,
            macroquad::input::KeyCode::CapsLock => Self::CapsLock,
            macroquad::input::KeyCode::ScrollLock => Self::ScrollLock,
            macroquad::input::KeyCode::NumLock => Self::NumLock,
            macroquad::input::KeyCode::PrintScreen => Self::PrintScreen,
            macroquad::input::KeyCode::Pause => Self::Pause,
            macroquad::input::KeyCode::F1 => Self::F1,
            macroquad::input::KeyCode::F2 => Self::F2,
            macroquad::input::KeyCode::F3 => Self::F3,
            macroquad::input::KeyCode::F4 => Self::F4,
            macroquad::input::KeyCode::F5 => Self::F5,
            macroquad::input::KeyCode::F6 => Self::F6,
            macroquad::input::KeyCode::F7 => Self::F7,
            macroquad::input::KeyCode::F8 => Self::F8,
            macroquad::input::KeyCode::F9 => Self::F9,
            macroquad::input::KeyCode::F10 => Self::F10,
            macroquad::input::KeyCode::F11 => Self::F11,
            macroquad::input::KeyCode::F12 => Self::F12,
            macroquad::input::KeyCode::F13 => Self::F13,
            macroquad::input::KeyCode::F14 => Self::F14,
            macroquad::input::KeyCode::F15 => Self::F15,
            macroquad::input::KeyCode::F16 => Self::F16,
            macroquad::input::KeyCode::F17 => Self::F17,
            macroquad::input::KeyCode::F18 => Self::F18,
            macroquad::input::KeyCode::F19 => Self::F19,
            macroquad::input::KeyCode::F20 => Self::F20,
            macroquad::input::KeyCode::F21 => Self::F21,
            macroquad::input::KeyCode::F22 => Self::F22,
            macroquad::input::KeyCode::F23 => Self::F23,
            macroquad::input::KeyCode::F24 => Self::F24,
            macroquad::input::KeyCode::F25 => Self::F25,
            macroquad::input::KeyCode::Kp0 => Self::Kp0,
            macroquad::input::KeyCode::Kp1 => Self::Kp1,
            macroquad::input::KeyCode::Kp2 => Self::Kp2,
            macroquad::input::KeyCode::Kp3 => Self::Kp3,
            macroquad::input::KeyCode::Kp4 => Self::Kp4,
            macroquad::input::KeyCode::Kp5 => Self::Kp5,
            macroquad::input::KeyCode::Kp6 => Self::Kp6,
            macroquad::input::KeyCode::Kp7 => Self::Kp7,
            macroquad::input::KeyCode::Kp8 => Self::Kp8,
            macroquad::input::KeyCode::Kp9 => Self::Kp9,
            macroquad::input::KeyCode::KpDecimal => Self::KpDecimal,
            macroquad::input::KeyCode::KpDivide => Self::KpDivide,
            macroquad::input::KeyCode::KpMultiply => Self::KpMultiply,
            macroquad::input::KeyCode::KpSubtract => Self::KpSubtract,
            macroquad::input::KeyCode::KpAdd => Self::KpAdd,
            macroquad::input::KeyCode::KpEnter => Self::KpEnter,
            macroquad::input::KeyCode::KpEqual => Self::KpEqual,
            macroquad::input::KeyCode::LeftShift => Self::LeftShift,
            macroquad::input::KeyCode::LeftControl => Self::LeftControl,
            macroquad::input::KeyCode::LeftAlt => Self::LeftAlt,
            macroquad::input::KeyCode::LeftSuper => Self::LeftSuper,
            macroquad::input::KeyCode::RightShift => Self::RightShift,
            macroquad::input::KeyCode::RightControl => Self::RightControl,
            macroquad::input::KeyCode::RightAlt => Self::RightAlt,
            macroquad::input::KeyCode::RightSuper => Self::RightSuper,
            macroquad::input::KeyCode::Menu => Self::Menu,
            macroquad::input::KeyCode::Unknown => Self::Unknown,
        }
    }
}

impl From<KeyCode> for macroquad::input::KeyCode {
    fn from(keycode: KeyCode) -> Self {
        match keycode {
            KeyCode::Space => Self::Space,
            KeyCode::Apostrophe => Self::Apostrophe,
            KeyCode::Comma => Self::Comma,
            KeyCode::Minus => Self::Minus,
            KeyCode::Period => Self::Period,
            KeyCode::Slash => Self::Slash,
            KeyCode::Key0 => Self::Key0,
            KeyCode::Key1 => Self::Key1,
            KeyCode::Key2 => Self::Key2,
            KeyCode::Key3 => Self::Key3,
            KeyCode::Key4 => Self::Key4,
            KeyCode::Key5 => Self::Key5,
            KeyCode::Key6 => Self::Key6,
            KeyCode::Key7 => Self::Key7,
            KeyCode::Key8 => Self::Key8,
            KeyCode::Key9 => Self::Key9,
            KeyCode::Semicolon => Self::Semicolon,
            KeyCode::Equal => Self::Equal,
            KeyCode::A => Self::A,
            KeyCode::B => Self::B,
            KeyCode::C => Self::C,
            KeyCode::D => Self::D,
            KeyCode::E => Self::E,
            KeyCode::F => Self::F,
            KeyCode::G => Self::G,
            KeyCode::H => Self::H,
            KeyCode::I => Self::I,
            KeyCode::J => Self::J,
            KeyCode::K => Self::K,
            KeyCode::L => Self::L,
            KeyCode::M => Self::M,
            KeyCode::N => Self::N,
            KeyCode::O => Self::O,
            KeyCode::P => Self::P,
            KeyCode::Q => Self::Q,
            KeyCode::R => Self::R,
            KeyCode::S => Self::S,
            KeyCode::T => Self::T,
            KeyCode::U => Self::U,
            KeyCode::V => Self::V,
            KeyCode::W => Self::W,
            KeyCode::X => Self::X,
            KeyCode::Y => Self::Y,
            KeyCode::Z => Self::Z,
            KeyCode::LeftBracket => Self::LeftBracket,
            KeyCode::Backslash => Self::Backslash,
            KeyCode::RightBracket => Self::RightBracket,
            KeyCode::GraveAccent => Self::GraveAccent,
            KeyCode::World1 => Self::World1,
            KeyCode::World2 => Self::World2,
            KeyCode::Escape => Self::Escape,
            KeyCode::Enter => Self::Enter,
            KeyCode::Tab => Self::Tab,
            KeyCode::Backspace => Self::Backspace,
            KeyCode::Insert => Self::Insert,
            KeyCode::Delete => Self::Delete,
            KeyCode::Right => Self::Right,
            KeyCode::Left => Self::Left,
            KeyCode::Down => Self::Down,
            KeyCode::Up => Self::Up,
            KeyCode::PageUp => Self::PageUp,
            KeyCode::PageDown => Self::PageDown,
            KeyCode::Home => Self::Home,
            KeyCode::End => Self::End,
            KeyCode::CapsLock => Self::CapsLock,
            KeyCode::ScrollLock => Self::ScrollLock,
            KeyCode::NumLock => Self::NumLock,
            KeyCode::PrintScreen => Self::PrintScreen,
            KeyCode::Pause => Self::Pause,
            KeyCode::F1 => Self::F1,
            KeyCode::F2 => Self::F2,
            KeyCode::F3 => Self::F3,
            KeyCode::F4 => Self::F4,
            KeyCode::F5 => Self::F5,
            KeyCode::F6 => Self::F6,
            KeyCode::F7 => Self::F7,
            KeyCode::F8 => Self::F8,
            KeyCode::F9 => Self::F9,
            KeyCode::F10 => Self::F10,
            KeyCode::F11 => Self::F11,
            KeyCode::F12 => Self::F12,
            KeyCode::F13 => Self::F13,
            KeyCode::F14 => Self::F14,
            KeyCode::F15 => Self::F15,
            KeyCode::F16 => Self::F16,
            KeyCode::F17 => Self::F17,
            KeyCode::F18 => Self::F18,
            KeyCode::F19 => Self::F19,
            KeyCode::F20 => Self::F20,
            KeyCode::F21 => Self::F21,
            KeyCode::F22 => Self::F22,
            KeyCode::F23 => Self::F23,
            KeyCode::F24 => Self::F24,
            KeyCode::F25 => Self::F25,
            KeyCode::Kp0 => Self::Kp0,
            KeyCode::Kp1 => Self::Kp1,
            KeyCode::Kp2 => Self::Kp2,
            KeyCode::Kp3 => Self::Kp3,
            KeyCode::Kp4 => Self::Kp4,
            KeyCode::Kp5 => Self::Kp5,
            KeyCode::Kp6 => Self::Kp6,
            KeyCode::Kp7 => Self::Kp7,
            KeyCode::Kp8 => Self::Kp8,
            KeyCode::Kp9 => Self::Kp9,
            KeyCode::KpDecimal => Self::KpDecimal,
            KeyCode::KpDivide => Self::KpDivide,
            KeyCode::KpMultiply => Self::KpMultiply,
            KeyCode::KpSubtract => Self::KpSubtract,
            KeyCode::KpAdd => Self::KpAdd,
            KeyCode::KpEnter => Self::KpEnter,
            KeyCode::KpEqual => Self::KpEqual,
            KeyCode::LeftShift => Self::LeftShift,
            KeyCode::LeftControl => Self::LeftControl,
            KeyCode::LeftAlt => Self::LeftAlt,
            KeyCode::LeftSuper => Self::LeftSuper,
            KeyCode::RightShift => Self::RightShift,
            KeyCode::RightControl => Self::RightControl,
            KeyCode::RightAlt => Self::RightAlt,
            KeyCode::RightSuper => Self::RightSuper,
            KeyCode::Menu => Self::Menu,
            KeyCode::Unknown => Self::Unknown,
        }
    }
}

impl From<MouseButton> for macroquad::input::MouseButton {
    fn from(button: MouseButton) -> Self {
        match button {
            MouseButton::Left => macroquad::input::MouseButton::Left,
            MouseButton::Middle => macroquad::input::MouseButton::Middle,
            MouseButton::Right => macroquad::input::MouseButton::Right,
            MouseButton::Unknown(..) => macroquad::input::MouseButton::Unknown,
        }
    }
}

impl From<macroquad::input::MouseButton> for MouseButton {
    fn from(button: macroquad::input::MouseButton) -> Self {
        match button {
            macroquad::input::MouseButton::Left => MouseButton::Left,
            macroquad::input::MouseButton::Middle => MouseButton::Middle,
            macroquad::input::MouseButton::Right => MouseButton::Right,
            macroquad::input::MouseButton::Unknown => MouseButton::Unknown(255),
        }
    }
}
