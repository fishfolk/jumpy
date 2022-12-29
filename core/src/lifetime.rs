//! Module providing entity lifetime components and systems

use crate::{prelude::*, FPS};

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PostUpdate, lifetime_system);
}

/// The lifetime state of an entity
///
/// > **Note:** The age represents how long the entity has had the [`Lifetime`] component on it, not
/// > necessarily how long since the entity was spawned, if the [`Lifetime`] component was added
/// > later, after it was spawned.
/// >
/// > Also, the age and lifetime are public, subject to other system's modification.
#[derive(Copy, Clone, Default, TypeUlid)]
#[ulid = "01GP9SS1WH06QC352CZ8BKSTZE"]
pub struct Lifetime {
    /// How long the entity should be allowed to live in seconds.
    pub lifetime: f32,
    /// How long the entity has lived in seconds.
    pub age: f32,
}

impl Lifetime {
    pub fn new(lifetime: f32) -> Self {
        Self {
            lifetime,
            ..default()
        }
    }
}

/// Despawns entities that have an expired lifetime
fn lifetime_system(mut entities: ResMut<Entities>, mut lifetimes: CompMut<Lifetime>) {
    let mut to_kill = Vec::new();
    for (entity, mut lifetime) in &mut entities.iter_with(&mut lifetimes) {
        lifetime.age += 1.0 / FPS;
        if lifetime.age > lifetime.lifetime {
            to_kill.push(entity);
        }
    }
    for entity in to_kill {
        entities.kill(entity);
    }
}
