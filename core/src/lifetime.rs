//! Entity lifetimes for deleting an entity after a period of time.

use std::time::Duration;

use crate::{prelude::*, FPS};

pub fn install(session: &mut CoreSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::PostUpdate, lifetime_system)
        .add_system_to_stage(CoreStage::PostUpdate, invincibility);
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
    for (entity, lifetime) in &mut entities.iter_with(&mut lifetimes) {
        lifetime.age += 1.0 / FPS;
        if lifetime.age > lifetime.lifetime {
            to_kill.push(entity);
        }
    }
    for entity in to_kill {
        entities.kill(entity);
    }
}

/// A timer that can be used to make an entity invincible for a certain amount of time
///
/// This is a general purpose invinvibility timer, but will serve as our spawn protection timer.
#[derive(Clone, Default, TypeUlid, Debug)]
#[ulid = "01GV3P99HFCZSC2MMXHA9394EJ"]
pub struct Invincibility(Timer);

impl Invincibility {
    pub fn new(duration: Duration) -> Self {
        Self(Timer::new(duration, TimerMode::Once))
    }
}

fn invincibility(
    time: Res<Time>,
    mut commands: Commands,
    entities: ResMut<Entities>,
    mut invincibles: CompMut<Invincibility>,
) {
    for (player_ent, invincible) in &mut entities.iter_with(&mut invincibles) {
        invincible.0.tick(time.delta());

        if invincible.0.finished() {
            commands.add(move |mut invincibles: CompMut<Invincibility>| {
                invincibles.remove(player_ent);
            });
        }
    }
}
