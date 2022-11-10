use crate::prelude::*;
pub use turborand::prelude::*;

pub struct RandomPlugin;

impl Plugin for RandomPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GlobalRng>();
    }
}

#[derive(Reflect, Component, Serialize, Deserialize, Debug, Deref, DerefMut)]
#[reflect_value(Component, Resource, Default, Serialize, Deserialize)]
pub struct GlobalRng(AtomicRng);

impl Clone for GlobalRng {
    fn clone(&self) -> Self {
        Self(postcard::from_bytes(&postcard::to_allocvec(&self.0).unwrap()).unwrap())
    }
}

impl Default for GlobalRng {
    fn default() -> Self {
        Self(AtomicRng::with_seed(7))
    }
}
