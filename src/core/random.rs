//! Global, deterministic random resource.

use crate::prelude::*;

use bones_framework::prelude::HasSchema;
pub use turborand::prelude::*;

pub fn plugin(session: &mut Session) {
    session.world.init_resource::<GlobalRng>();
}

/// Resource that can produce deterministic, pseudo-random numbers.
///
/// Access in a system with [`Res<GlobalRng>`].
#[derive(Clone, HasSchema, Deref, DerefMut)]
pub struct GlobalRng(AtomicRng);

impl Default for GlobalRng {
    fn default() -> Self {
        Self(AtomicRng::with_seed(7))
    }
}
