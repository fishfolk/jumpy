use fishsticks::GamepadId;

use serde::{Deserialize, Serialize};

use crate::error::ErrorKind;
use crate::Result;

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

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum Button {
    A,
    B,
    X,
    Y,
    Back,
    Guide,
    Start,
    LeftStick,
    RightStick,
    LeftShoulder,
    RightShoulder,
    LeftTrigger,
    RightTrigger,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    #[serde(skip)]
    Unknown,
}

impl From<fishsticks::Button> for Button {
    fn from(button: fishsticks::Button) -> Self {
        match button {
            fishsticks::Button::South => Self::A,
            fishsticks::Button::East => Self::B,
            fishsticks::Button::West => Self::X,
            fishsticks::Button::North => Self::Y,
            fishsticks::Button::Select => Self::Back,
            fishsticks::Button::Mode => Self::Guide,
            fishsticks::Button::Start => Self::Start,
            fishsticks::Button::LeftThumb => Self::LeftStick,
            fishsticks::Button::RightThumb => Self::RightStick,
            fishsticks::Button::LeftTrigger => Self::LeftTrigger,
            fishsticks::Button::RightTrigger => Self::RightTrigger,
            fishsticks::Button::LeftTrigger2 => Self::LeftShoulder,
            fishsticks::Button::RightTrigger2 => Self::RightShoulder,
            fishsticks::Button::DPadUp => Self::DPadUp,
            fishsticks::Button::DPadDown => Self::DPadDown,
            fishsticks::Button::DPadLeft => Self::DPadLeft,
            fishsticks::Button::DPadRight => Self::DPadRight,
            _ => Self::Unknown,
        }
    }
}

impl From<Button> for fishsticks::Button {
    fn from(button: Button) -> Self {
        match button {
            Button::A => Self::South,
            Button::B => Self::East,
            Button::X => Self::West,
            Button::Y => Self::North,
            Button::Back => Self::Select,
            Button::Guide => Self::Mode,
            Button::Start => Self::Start,
            Button::LeftStick => Self::LeftThumb,
            Button::RightStick => Self::RightThumb,
            Button::LeftTrigger => Self::LeftTrigger,
            Button::RightTrigger => Self::RightTrigger,
            Button::LeftShoulder => Self::LeftTrigger2,
            Button::RightShoulder => Self::RightTrigger2,
            Button::DPadUp => Self::DPadUp,
            Button::DPadDown => Self::DPadDown,
            Button::DPadLeft => Self::DPadLeft,
            Button::DPadRight => Self::DPadRight,
            Button::Unknown => Self::Unknown,
        }
    }
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

impl From<&GamepadId> for GamepadMapping {
    fn from(id: &GamepadId) -> Self {
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
