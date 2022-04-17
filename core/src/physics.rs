pub use crate::physics_impl::*;

use std::borrow::{Borrow, BorrowMut};
use std::ops::{Deref, DerefMut};
use std::time::Duration;

static mut PHYSICS_WORLD: Option<PhysicsWorld> = None;

/// Init physics with the specified parameters.
/// This does not need to be called unless you want to use non-default parameters
pub fn init_physics(resolution: u32) -> &'static mut PhysicsWorld {
    unsafe { PHYSICS_WORLD.get_or_insert_with(|| PhysicsWorld::new(resolution)) }
}

/// Get the current physics world or initialize with defaults if this has not been done yet
pub fn physics_world() -> &'static mut PhysicsWorld {
    unsafe { PHYSICS_WORLD.get_or_insert_with(PhysicsWorld::default) }
}

pub fn fixed_delta_time() -> Duration {
    physics_world().fixed_delta_time()
}
