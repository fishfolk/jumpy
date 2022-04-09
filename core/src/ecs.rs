use hecs::{Entity, World};

use crate::Result;

pub type UpdateSystemFn = fn(world: &mut World, delta_time: f32) -> Result<()>;

pub type FixedUpdateSystemFn =
    fn(world: &mut World, delta_time: f32, integration_factor: f32) -> Result<()>;

pub type DrawSystemFn = fn(world: &mut World) -> Result<()>;

/// This is used as a component to signify ownership
pub struct Owner(pub Entity);
