pub use fishsticks::{Axis, GamepadContext, GamepadId};
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::result::Result;

static mut GAMEPAD_CONTEXT: Option<GamepadContext> = None;

pub async fn init_gamepad_context() -> Result<()> {
    let ctx = GamepadContext::init()?;
    unsafe { GAMEPAD_CONTEXT = Some(ctx) }
    Ok(())
}

pub fn gamepad_context() -> &'static GamepadContext {
    unsafe {
        GAMEPAD_CONTEXT.as_ref().unwrap_or_else(|| {
            panic!("Attempted to get gamepad context but it has not been initialized yet!")
        })
    }
}

pub fn gamepad_context_mut() -> &'static mut GamepadContext {
    unsafe {
        GAMEPAD_CONTEXT.as_mut().unwrap_or_else(|| {
            panic!("Attempted to get gamepad context but it has not been initialized yet!")
        })
    }
}

pub fn update_gamepad_context() -> Result<()> {
    gamepad_context_mut().update()?;
    Ok(())
}

/// Check if a gamepad button is pressed on gamepad with id `gamepad_id`, or if it is pressed on
/// any gamepad if `gamepad_id` is `None`
pub fn is_gamepad_button_pressed<G: Into<Option<GamepadId>>>(gamepad_id: G, btn: Button) -> bool {
    let ctx = gamepad_context();

    if let Some(id) = gamepad_id.into() {
        if let Some(gamepad) = ctx.gamepad(id) {
            return gamepad.digital_inputs.just_activated(btn.into());
        } else {
            #[cfg(debug_assertions)]
            println!("WARNING: Gamepad with id '{:?}' was not found!", id);
        }
    } else {
        for (_, gamepad) in ctx.gamepads() {
            if gamepad.digital_inputs.just_activated(btn.into()) {
                return true;
            }
        }
    }

    false
}

/// Check if a gamepad button is pressed on gamepad with id `gamepad_id`, or if it is pressed on
/// any gamepad if `gamepad_id` is `None`
pub fn is_gamepad_button_down<G: Into<Option<GamepadId>>>(gamepad_id: G, btn: Button) -> bool {
    let ctx = gamepad_context();

    if let Some(id) = gamepad_id.into() {
        if let Some(gamepad) = ctx.gamepad(id) {
            return gamepad.digital_inputs.activated(btn.into());
        } else {
            #[cfg(debug_assertions)]
            println!("WARNING: Gamepad with id '{:?}' was not found!", id);
        }
    } else {
        for (_, gamepad) in ctx.gamepads() {
            if gamepad.digital_inputs.activated(btn.into()) {
                return true;
            }
        }
    }

    false
}

/// Get axis value for gamepad with id `gamepad_id`
pub fn gamepad_axis(gamepad_id: GamepadId, axis: Axis) -> f32 {
    let ctx = gamepad_context();

    if let Some(id) = gamepad_id.into() {
        if let Some(gamepad) = ctx.gamepad(id) {
            return gamepad.analog_inputs.value(axis);
        } else {
            #[cfg(debug_assertions)]
            println!("WARNING: Gamepad with id '{:?}' was not found!", id);
        }
    }

    0.0
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
