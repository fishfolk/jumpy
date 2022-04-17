pub use hecs::*;

use crate::Result;

pub type UpdateFn = fn(world: &mut World, delta_time: f32) -> Result<()>;

pub type FixedUpdateFn =
    fn(world: &mut World, delta_time: f32, integration_factor: f32) -> Result<()>;

pub type DrawFn = fn(world: &mut World, _delta_time: f32) -> Result<()>;

/// This is used as a component to signify ownership
pub struct Owner(pub Entity);
