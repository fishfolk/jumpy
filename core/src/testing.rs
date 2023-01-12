//! Contains temporary and testing/debugging systems.

use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    bones_lib::install(&mut session.stages);
    session
        .stages
        .add_system_to_stage(CoreStage::Update, handle_input);
}
fn handle_input(inputs: Res<PlayerInputs>, mut _bodies: CompMut<KinematicBody>) {
    let _control = &inputs.players[0].control;
}
