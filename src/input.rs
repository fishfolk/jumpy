use crate::GamepadContext;

pub fn is_gamepad_btn_pressed(gamepad_system: &GamepadContext, btn: fishsticks::Button) -> bool {
    for (_, gamepad) in gamepad_system.gamepads() {
        if gamepad.digital_inputs.just_activated(btn) {
            return true;
        }
    }

    false
}
