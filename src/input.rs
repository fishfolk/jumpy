use macroquad::experimental::collections::storage;

use core::Result;

use crate::GamepadContext;

pub fn update_gamepad_context(context: Option<&mut GamepadContext>) -> Result<()> {
    if let Some(context) = context {
        context.update()?;
    } else {
        let mut context = storage::get_mut::<GamepadContext>();
        context.update()?;
    }

    Ok(())
}

pub fn is_gamepad_btn_pressed(context: Option<&GamepadContext>, btn: fishsticks::Button) -> bool {
    let check = |context: &GamepadContext| -> bool {
        for (_, gamepad) in context.gamepads() {
            if gamepad.digital_inputs.just_activated(btn) {
                return true;
            }
        }

        false
    };

    if let Some(context) = context {
        check(context)
    } else {
        let context = storage::get::<GamepadContext>();
        check(&context)
    }
}
