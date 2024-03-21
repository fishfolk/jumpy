//! DynamicBody component allows simultaing phyiscs of a body.

use std::sync::Mutex;

use crate::prelude::*;

pub type SimulationCommand = dyn FnOnce(&mut rapier::RigidBody) + 'static + Send;

/// Bodies with `DynamicBody` will be simulated dynamically
/// if `is_dynamic` is True. Currently `DynamicBody` requires component
/// [`KinematicBody`], as it shares shape.
///
/// `if is_dynamic` is false, body is treated as kinematic.
#[derive(Default, Clone, HasSchema)]
#[repr(C)]
pub struct DynamicBody {
    /// Is body simulating or kinematic mode?
    pub is_dynamic: bool,

    /// This transform is used to determine if position has moved in gameplay code outside of rapier sim.
    /// Is only relevant when `is_dynamic` = true. It is updated whenever transform is synced to or from rapier.
    ///
    /// If this does not match entity `Transform` at beginning of physics update, rapier simulation body
    /// will be teleported to match gameplay.
    ///
    /// See [`Self::simulation_transform_needs_update`] for context.
    last_rapier_synced_transform: Transform,

    /// Simulation command queue. See [`Self::push_simulation_command`] for details.
    #[schema(opaque)]
    command_queue: Arc<Mutex<Vec<Box<SimulationCommand>>>>,
}

impl DynamicBody {
    /// Create new `DynamicBody`. is_dynamic = true simulates physics, false remains kinematic.
    pub fn new(is_dynamic: bool) -> Self {
        Self {
            is_dynamic,
            ..Default::default()
        }
    }
    /// Check if rapier simulation's body's transform is dirty (If it was modified outside of
    /// physics simulation). Always returns true if object is not simulating.
    ///
    /// Updates `last_rapier_synced_transform` if called, should only be called before updating rapier positions from gameplay.
    pub fn simulation_transform_needs_update(&mut self, transform: &Transform) -> bool {
        if !self.is_dynamic {
            // Body not simulating, we should always update transform in sim
            return true;
        }

        // Only use 2D translation + rotation, as this is what rapier sim uses.
        let last_translation = self.last_rapier_synced_transform.translation.xy();
        let current_translation = transform.translation.xy();
        let last_rotation = self
            .last_rapier_synced_transform
            .rotation
            .to_euler(EulerRot::XYZ)
            .2;
        let current_rotation = transform.rotation.to_euler(EulerRot::XYZ).2;

        // transform was modified in gameplay outside of rapier sim
        if last_translation != current_translation || last_rotation != current_rotation {
            self.last_rapier_synced_transform.translation = transform.translation;
            self.last_rapier_synced_transform.rotation = transform.rotation;

            return true;
        }

        // Transform was not modified outside of rapier sim, is not dirty
        false
    }

    /// Update `last_rapier_synced_transform` from simulation output so we can determine when gameplay has modified transform
    /// outside of rapier for [`Self::simulation_transform_needs_update`].
    ///
    /// This does NOT update position of entity.
    pub fn update_last_rapier_synced_transform(&mut self, translation: Vec3, rotation: Quat) {
        self.last_rapier_synced_transform.translation = translation;
        self.last_rapier_synced_transform.rotation = rotation;
    }

    /// Push command that is executed before next physics step, after body is guranteed to be initialized in rapier
    /// and set to [`rapier::RigidBodyType::Dynamic`] (if transitioning from Kinematic to Dynamic).
    ///
    /// Commands are only executed if `is_dynamic` = true (body is simulating) next frame! Otherwise they are discarded.
    /// This restriction is in place to prevent switching back to kinematic and modifying simulation state in an unintentional way.
    ///
    /// Commands are executed in order they are pushed. If you need to immediately apply changes, use [`CollisionWorld::mutate_rigidbody`].
    pub fn push_simulation_command(&mut self, command: Box<SimulationCommand>) {
        self.command_queue.lock().unwrap().push(command);
    }

    /// Retrieve [`SimulationCommand`] queue for processing. Values are moved out of self, flushing queue.
    pub fn simulation_commands(&mut self) -> Vec<Box<SimulationCommand>> {
        let mut dummy: Vec<Box<SimulationCommand>> = vec![];
        let mut queue = self.command_queue.lock().unwrap();
        std::mem::swap(&mut *queue, &mut dummy);
        dummy
    }
}
