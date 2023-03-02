use crate::prelude::*;

pub mod crab;
pub mod crate_item;
pub mod decoration;
pub mod fish_school;
pub mod grenade;
pub mod kick_bomb;
pub mod mine;
pub mod musket;
pub mod player_spawner;
pub mod slippery_seaweed;
pub mod sproinger;
pub mod stomp_boots;
pub mod sword;
pub mod urchin;

/// Marker component added to map elements that have been hydrated.
#[derive(Clone, TypeUlid)]
#[ulid = "01GP42Q5GCY5Y4JC7SQ1YRHYKN"]
pub struct MapElementHydrated;

/// Component that contains the [`Entity`] to de-hydrate when the entity with this component is out
/// of the [`LoadedMap`] bounds.
///
/// This is useful for map elements that spawn items: when the item falls off the map, it should
/// de-hydrate it's spawner, so that the spawner will re-spawn the item in it's default state.
#[derive(Clone, TypeUlid, Deref, DerefMut)]
#[ulid = "01GP9NY0Y50Y2A8M4A7E9NN8VE"]
pub struct DehydrateOutOfBounds(pub Entity);

/// Component containing an element's metadata handle.
#[derive(Clone, TypeUlid, Deref, DerefMut, Default)]
#[ulid = "01GP421CHN323T2614F19PA5E9"]
pub struct ElementHandle(pub Handle<ElementMeta>);

pub fn install(session: &mut GameSession) {
    session
        .stages
        .add_system_to_stage(CoreStage::First, handle_out_of_bounds_items);

    decoration::install(session);
    urchin::install(session);
    player_spawner::install(session);
    sproinger::install(session);
    sword::install(session);
    grenade::install(session);
    crab::install(session);
    fish_school::install(session);
    kick_bomb::install(session);
    mine::install(session);
    musket::install(session);
    stomp_boots::install(session);
    crate_item::install(session);
    slippery_seaweed::install(session);
}

fn handle_out_of_bounds_items(
    mut commands: Commands,
    mut hydrated: CompMut<MapElementHydrated>,
    entities: ResMut<Entities>,
    transforms: CompMut<Transform>,
    spawners: Comp<DehydrateOutOfBounds>,
    map: Res<LoadedMap>,
) {
    for (item_ent, (transform, spawner)) in entities.iter_with((&transforms, &spawners)) {
        if map.is_out_of_bounds(&transform.translation) {
            hydrated.remove(**spawner);
            commands.add(move |mut entities: ResMut<Entities>| {
                entities.kill(item_ent);
            });
        }
    }
}
