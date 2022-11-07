//! Utilities related to system scheduling and, in particular, the netcode rollback schedule.

use crate::prelude::*;

pub trait RollbackScheduleAppExt {
    fn extend_rollback_schedule<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Schedule);
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
}
