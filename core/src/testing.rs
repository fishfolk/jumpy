//! Contains temporary and testing/debugging systems.

use crate::prelude::*;

pub fn install(session: &mut GameSession) {
    bones_lib::install(&mut session.stages);
    // session
    //     .stages
    //     .add_system_to_stage(CoreStage::Update, testing);
}
// fn testing(entities: Res<Entities>, transforms: Comp<Transform>, players: Comp<PlayerIdx>) {
//     for (_ent, (trans, _idx)) in entities.iter_with((&transforms, &players)) {
//         dbg!(trans.translation.truncate());
//     }
// }
