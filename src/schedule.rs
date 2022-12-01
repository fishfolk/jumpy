//! Utilities related to system scheduling and, in particular, the netcode rollback schedule.

use bevy::{ecs::schedule::IntoSystemDescriptor, utils::HashMap};
use bevy_ggrs::GGRSPlugin;
use bevy_system_graph::{SystemGraph, SystemGraphNode};

use crate::prelude::*;

pub type RollbackSystems = HashMap<RollbackStage, RollbackSystemSet>;

pub struct RollbackSystemSet {
    pub graph: SystemGraph,
    pub last_system: SystemGraphNode,
}

pub trait RollbackScheduleAppExt {
    fn extend_rollback_schedule<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut Schedule);
    fn add_rollback_system<Params>(
        &mut self,
        stage: RollbackStage,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self;
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

    fn add_rollback_system<Params>(
        &mut self,
        stage: RollbackStage,
        system: impl IntoSystemDescriptor<Params>,
    ) -> &mut Self {
        let mut systems = self.world.non_send_resource_mut::<RollbackSystems>();
        match systems.get_mut(&stage) {
            Some(set) => {
                set.last_system = set.last_system.then(system);
            }
            None => {
                let graph = SystemGraph::new();
                systems.insert(
                    stage,
                    RollbackSystemSet {
                        last_system: graph.root(system),
                        graph,
                    },
                );
            }
        }
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
