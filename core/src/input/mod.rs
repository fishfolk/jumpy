mod gamepad;

pub use gamepad::*;

use serde::{Deserialize, Serialize};

use crate::error::ErrorKind;
use crate::formaterr;
use crate::result::Result;

pub use crate::backend_impl::input::*;

#[derive(Default, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerInput {
    pub left: bool,
    pub right: bool,
    pub fire: bool,
    pub jump: bool,
    pub pickup: bool,
    pub float: bool,
    pub crouch: bool,
    pub slide: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GameInputScheme {
    /// Left side of the keyboard, around WASD
    KeyboardRight,
    /// Left side of the keyboard, around Arrows
    KeyboardLeft,
    /// Gamepad index
    Gamepad(fishsticks::GamepadId),
}

pub fn collect_local_input(input_scheme: GameInputScheme) -> PlayerInput {
    let mut input = PlayerInput::default();

    if let GameInputScheme::Gamepad(gamepad_id) = input_scheme {
        let input_mapping = input_mapping()
            .get_gamepad_mapping(gamepad_id.into())
            .unwrap_or_else(|| gamepad_id.into());

        input.left = is_gamepad_button_down(gamepad_id, Button::DPadLeft)
            || gamepad_axis(gamepad_id, Axis::LeftStickX) < 0.0;

        input.right = is_gamepad_button_down(gamepad_id, Button::DPadRight)
            || gamepad_axis(gamepad_id, Axis::LeftStickX) > 0.0;

        input.fire = is_gamepad_button_down(gamepad_id, input_mapping.fire);

        input.jump = is_gamepad_button_pressed(gamepad_id, input_mapping.jump);

        input.pickup = is_gamepad_button_pressed(gamepad_id, input_mapping.pickup);

        input.crouch = is_gamepad_button_down(gamepad_id, Button::DPadDown)
            || gamepad_axis(gamepad_id, Axis::LeftStickY) > 0.0;

        input.slide = is_gamepad_button_pressed(gamepad_id, input_mapping.slide);
    } else {
        let input_mapping = if matches!(input_scheme, GameInputScheme::KeyboardRight) {
            input_mapping().keyboard_primary.clone()
        } else {
            input_mapping().keyboard_secondary.clone()
        };

        #[allow(clippy::useless_conversion)]
        {
            input.left = is_key_down(input_mapping.left.into());
            input.right = is_key_down(input_mapping.right.into());
            input.fire = is_key_down(input_mapping.fire.into());
            input.jump = is_key_pressed(input_mapping.jump.into());
            input.pickup = is_key_pressed(input_mapping.pickup.into());
            input.float = is_key_down(input_mapping.jump.into());
            input.crouch = is_key_down(input_mapping.crouch.into());
            input.slide = input.crouch && is_key_pressed(input_mapping.slide.into());
        }
    }

    input
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq, Serialize, Deserialize)]
pub enum KeyCode {
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    Kp0,
    Kp1,
    Kp2,
    Kp3,
    Kp4,
    Kp5,
    Kp6,
    Kp7,
    Kp8,
    Kp9,
    KpDecimal,
    KpDivide,
    KpMultiply,
    KpSubtract,
    KpAdd,
    KpEnter,
    KpEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMapping {
    primary: KeyCode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    secondary: Option<KeyCode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardMapping {
    pub left: KeyCode,
    pub right: KeyCode,
    pub fire: KeyCode,
    pub jump: KeyCode,
    pub pickup: KeyCode,
    pub crouch: KeyCode,
    pub slide: KeyCode,
}

impl KeyboardMapping {
    pub fn default_primary() -> KeyboardMapping {
        KeyboardMapping {
            left: KeyCode::Left,
            right: KeyCode::Right,
            fire: KeyCode::L,
            jump: KeyCode::Up,
            pickup: KeyCode::K,
            crouch: KeyCode::Down,
            slide: KeyCode::RightControl,
        }
    }

    pub fn default_secondary() -> KeyboardMapping {
        KeyboardMapping {
            left: KeyCode::A,
            right: KeyCode::D,
            fire: KeyCode::V,
            jump: KeyCode::W,
            pickup: KeyCode::C,
            crouch: KeyCode::S,
            slide: KeyCode::F,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamepadMapping {
    pub id: usize,
    pub fire: Button,
    pub jump: Button,
    pub pickup: Button,
    pub slide: Button,
}

impl From<usize> for GamepadMapping {
    fn from(id: usize) -> Self {
        GamepadMapping {
            id,
            fire: Button::B,
            jump: Button::A,
            pickup: Button::X,
            slide: Button::Y,
        }
    }
}

impl From<GamepadId> for GamepadMapping {
    fn from(id: GamepadId) -> Self {
        let id: usize = id.into();
        id.into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputMapping {
    #[serde(
        default = "KeyboardMapping::default_primary",
        rename = "keyboard-primary"
    )]
    pub keyboard_primary: KeyboardMapping,
    #[serde(
        default = "KeyboardMapping::default_secondary",
        rename = "keyboard-secondary"
    )]
    pub keyboard_secondary: KeyboardMapping,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gamepads: Vec<GamepadMapping>,
}

impl InputMapping {
    pub fn get_gamepad_mapping(&self, id: usize) -> Option<GamepadMapping> {
        self.gamepads.iter().find_map(|gamepad| {
            if gamepad.id == id {
                Some(gamepad.clone())
            } else {
                None
            }
        })
    }

    pub fn verify(&mut self) -> Result<()> {
        {
            let mut used_keys = Vec::new();

            let keyboards = [&self.keyboard_primary, &self.keyboard_secondary];

            for keyboard in keyboards {
                let actions = [
                    keyboard.left,
                    keyboard.right,
                    keyboard.fire,
                    keyboard.jump,
                    keyboard.pickup,
                    keyboard.crouch,
                    keyboard.slide,
                ];

                for keycode in actions {
                    if used_keys.contains(&keycode) {
                        return Err(formaterr!(
                            ErrorKind::Config,
                            "Key '{:?}' is mapped twice!",
                            keycode
                        ));
                    } else {
                        used_keys.push(keycode);
                    }
                }
            }
        }

        {
            let mut used_buttons = Vec::new();

            for gamepad in &self.gamepads {
                let actions = [gamepad.fire, gamepad.jump, gamepad.pickup, gamepad.slide];

                for button in actions {
                    if used_buttons.contains(&button) {
                        return Err(formaterr!(
                            ErrorKind::Config,
                            "Button '{:?}' on gamepad '{}' is mapped twice!",
                            button,
                            gamepad.id,
                        ));
                    } else {
                        used_buttons.push(button);
                    }
                }
            }
        }

        Ok(())
    }
}

impl Default for InputMapping {
    fn default() -> Self {
        InputMapping {
            keyboard_primary: KeyboardMapping::default_primary(),
            keyboard_secondary: KeyboardMapping::default_secondary(),
            gamepads: Vec::new(),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Hash, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    Unknown(usize),
}
