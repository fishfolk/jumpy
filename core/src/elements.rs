use crate::prelude::*;

pub mod crab;
pub mod decoration;
pub mod grenade;
pub mod kick_bomb;
pub mod musket;
pub mod player_spawner;
pub mod sproinger;
pub mod sword;

/// Marker component added to map elements that have been hydrated.
#[derive(Clone, TypeUlid)]
#[ulid = "01GP42Q5GCY5Y4JC7SQ1YRHYKN"]
pub struct MapElementHydrated;

/// Component containing an element's metadata handle.
#[derive(Clone, TypeUlid, Deref, DerefMut, Default)]
#[ulid = "01GP421CHN323T2614F19PA5E9"]
pub struct ElementHandle(pub Handle<ElementMeta>);

pub fn install(session: &mut GameSession) {
    decoration::install(session);
    player_spawner::install(session);
    sproinger::install(session);
    sword::install(session);
    grenade::install(session);
    crab::install(session);
    kick_bomb::install(session);

    musket::install(session);
}
