//! Utilities related to system scheduling and, in particular, the netcode rollback schedule.

use bevy_ggrs::GGRSPlugin;

use crate::prelude::*;

pub trait RollbackScheduleAppExt {
    fn extend_rollback_schedule<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Schedule);
    fn extend_rollback_plugin<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(GGRSPlugin<crate::GgrsConfig>) -> GGRSPlugin<crate::GgrsConfig>;
}

impl RollbackScheduleAppExt for App {
    fn extend_rollback_schedule<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Schedule),
    {
        let mut schedule = self.world.resource_mut();
        f(&mut schedule);
        self
    }
    fn extend_rollback_plugin<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(GGRSPlugin<crate::GgrsConfig>) -> GGRSPlugin<crate::GgrsConfig>,
    {
        let plugin = self.world.remove_resource().unwrap();
        self.world.insert_resource(f(plugin));
        self
    }
}
