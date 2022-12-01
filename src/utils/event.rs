use bevy::ecs::event::Event;

use crate::prelude::*;

pub trait FixedUpdateEventAppExt {
    fn add_fixed_update_event<E: Event>(&mut self) -> &mut Self;
}

impl FixedUpdateEventAppExt for bevy::app::App {
    fn add_fixed_update_event<E: Event>(&mut self) -> &mut Self {
        self.init_resource::<Events<E>>()
            .add_rollback_system(RollbackStage::First, Events::<E>::update_system);

        self
    }
}
