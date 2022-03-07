use hecs::{Entity, World};

pub type UpdateSystemFn = fn(world: &mut World, delta_time: f32);

pub type FixedUpdateSystemFn = fn(world: &mut World, delta_time: f32, integration_factor: f32);

pub type DrawSystemFn = fn(world: &mut World);

/// This is used as a component to signify ownership
pub struct Owner(pub Entity);
