use crate::prelude::*;
pub use turborand::prelude::*;

pub fn install(session: &mut GameSession) {
    session.world.init_resource::<GlobalRng>();
}

#[derive(Clone, TypeUlid, Deref, DerefMut)]
#[ulid = "01GQ0K6DDA9KKQTM3WDK1R91TE"]
pub struct GlobalRng(AtomicRng);

impl Default for GlobalRng {
    fn default() -> Self {
        Self(AtomicRng::with_seed(7))
    }
}
