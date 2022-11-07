//! Module providing entity lifetime components and systems

use crate::{prelude::*, FPS};

pub struct LifetimePlugin;

impl Plugin for LifetimePlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Lifetime>()
            .extend_rollback_schedule(|schedule| {
                schedule.add_system_to_stage(RollbackStage::PostUpdate, lifetime_system);
            });
    }
}

/// The lifetime state of an entity
///
/// > **Note:** The age represents how long the entity has had the [`Lifetime`] component on it, not
/// > necessarily how long since the entity was spawned, if the [`Lifetime`] component was added
/// > later, after it was spawned.
/// >
/// > Also, the age and lifetime are public, subject to other system's modification.
#[derive(Reflect, Debug, Clone, Component, Default)]
#[reflect(Component, Default)]
pub struct Lifetime {
    /// How long the entity should be allowed to live in seconds.
    pub lifetime: f32,
    /// How long the entity has lived in seconds.
    pub age: f32,
    /// Set this to `true` to despawn the entity non-recursively when the lifetime is expired.
    ///
    /// By default this is set to false and will despawn the entity recursively.
    pub non_recursive_despawn: bool,
}

/// Despawns entities that have an expired lifetime
fn lifetime_system(mut commands: Commands, mut entities: Query<(Entity, &mut Lifetime)>) {
    for (entity, mut lifetime) in &mut entities {
        lifetime.age += FPS as f32;
        if lifetime.age >= lifetime.lifetime {
            if lifetime.non_recursive_despawn {
                commands.entity(entity).despawn();
            } else {
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}
