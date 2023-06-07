//! Map element implementations.
//!
//! A map element is anything that can be placed on the map in the editor.

use std::sync::Mutex;

use ::bevy::{
    prelude::Resource,
    utils::{HashMap, Uuid},
};

use crate::{impl_system_param, prelude::*};

pub mod crab;
pub mod crate_item;
pub mod decoration;
pub mod fish_school;
pub mod grenade;
pub mod kick_bomb;
pub mod mine;
pub mod musket;
pub mod player_spawner;
pub mod slippery;
pub mod slippery_seaweed;
pub mod snail;
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

#[derive(Clone, TypeUlid)]
#[ulid = "01GP584Z9WN5P0RG2A82MV93P1"]
pub struct ElementKillCallback {
    pub system: Arc<Mutex<System>>,
}

impl ElementKillCallback {
    pub fn new<Args>(system: impl IntoSystem<Args, ()>) -> Self {
        ElementKillCallback {
            system: Arc::new(Mutex::new(system.system())),
        }
    }
}

#[derive(Clone, TypeUlid)]
#[ulid = "01H0AQGTZCVZJXPF2KSR73TQTR"]
pub struct Spawner {
    /// The group identifier where all of the elements are meant to be shared between them
    pub group_identifier: String,
}

impl Default for Spawner {
    fn default() -> Self {
        Self {
            group_identifier: Uuid::new_v4().to_string(),
        }
    }
}

impl Spawner {
    pub fn new() -> Self {
        Spawner {
            group_identifier: Uuid::new_v4().to_string(),
        }
    }
    pub fn new_grouped(group_identifier: String) -> Self {
        Spawner { group_identifier }
    }
}

#[derive(Resource, TypeUlid, Clone, Default)]
#[ulid = "01H11SA5GRF6Y6N20XKB8JR5TE"]
pub struct SpawnerEntities {
    pub entities_per_spawner_group_identifier: HashMap<String, Vec<Entity>>,
}

impl_system_param! {
    pub struct SpawnerManager<'a> {
        spawners: CompMut<'a, Spawner>,
        spawner_entities: ResMut<'a, SpawnerEntities>,
    }
}

impl<'a> SpawnerManager<'a> {
    /// Stores the spawned elements as having been spawned by the provided entity, finding an existing spawner element, if it exists, to group these spawned elements with.
    pub fn create_grouped_spawner<T: EcsData + TypeUlid>(
        &mut self,
        entity: Entity,
        mut spawned_elements: Vec<Entity>,
        spawner_elements: &CompMut<T>,
        entities: &Res<Entities>,
    ) {
        // try to find one other spawner and store the existing player entities
        if let Some((_, (_, first_spawner))) = entities
            .iter_with((spawner_elements, &self.spawners))
            .next()
        {
            // all of the player spawners share the same group identifier
            let spawner = Spawner::new_grouped(first_spawner.group_identifier.clone());

            // add the spawned elements to the existing resource
            self.spawner_entities
                .entities_per_spawner_group_identifier
                .get_mut(&spawner.group_identifier)
                .expect("The spawner group should already exist in the SpawnerEntities resource.")
                .append(&mut spawned_elements);

            self.spawners.insert(entity, spawner);
        } else {
            let spawner = Spawner::new();

            // add the spawned elements to the newly created resource
            self.spawner_entities
                .entities_per_spawner_group_identifier
                .insert(spawner.group_identifier.clone(), spawned_elements);

            self.spawners.insert(entity, spawner);
        }
    }
    /// Stores the spawned elements as having been spawned by the provided entity
    pub fn create_spawner(&mut self, entity: Entity, spawned_elements: Vec<Entity>) {
        let spawner = Spawner::new();

        // add the spawned elements to the newly created resource
        self.spawner_entities
            .entities_per_spawner_group_identifier
            .insert(spawner.group_identifier.clone(), spawned_elements);

        self.spawners.insert(entity, spawner);
    }
    /// Stores the spawned elements as having come from the same group of spawners as the spawner_elements.
    pub fn insert_spawned_entity_into_grouped_spawner<T: EcsData + TypeUlid>(
        &mut self,
        spawned_entity: Entity,
        spawner_elements: &Comp<T>,
        entities: &ResMut<Entities>,
    ) {
        let (_, (_, spawner)) = entities
            .iter_with((spawner_elements, &self.spawners))
            .next()
            .expect("There should already exist at least one spawner of the type provided.");
        self.spawner_entities.entities_per_spawner_group_identifier
            .get_mut(&spawner.group_identifier)
            .expect("There should exist a cooresponding SpawnerEntities for this spawner group identifier.")
            .push(spawned_entity);
    }
    /// Removes that spawned entity from the spawner entities resource
    pub fn remove_spawned_entity_from_grouped_spawner<T: EcsData + TypeUlid>(
        &mut self,
        spawned_entity: Entity,
        spawner_elements: &Comp<T>,
        entities: &ResMut<Entities>,
    ) {
        let (_, (_, spawner)) = entities
            .iter_with((spawner_elements, &self.spawners))
            .next()
            .expect("There should already exist at least one spawner of the type provided.");
        self.spawner_entities.entities_per_spawner_group_identifier
            .get_mut(&spawner.group_identifier)
            .expect("There should exist a cooresponding SpawnerEntities for this spawner group identifier.")
            .retain(|entity| *entity != spawned_entity);
    }
    /// Returns if the entity provided is a spawner
    pub fn is_entity_a_spawner(&self, entity: Entity) -> bool {
        self.spawners.contains(entity)
    }
    /// Kills the provided spawner entity and any spawned entities (if applicable)
    pub fn kill_spawner_entity(
        &mut self,
        spawner_entity: Entity,
        entities: &mut ResMut<Entities>,
        element_kill_callbacks: &Comp<ElementKillCallback>,
        commands: &mut Commands,
    ) {
        let spawner = self
            .spawners
            .get(spawner_entity)
            .expect("The spawner must exist in order to be deleted.");
        let grouped_spawners_count = self
            .spawners
            .iter()
            .filter(|other_spawner| spawner.group_identifier == other_spawner.group_identifier)
            .count();
        if grouped_spawners_count == 1 {
            let entities_per_spawner_group_identifier = self.spawner_entities
                .entities_per_spawner_group_identifier
                .remove(&spawner.group_identifier)
                .expect("The spawner still exists to be deleted, so there should be a cooresponding vector of spawned entities.");

            entities_per_spawner_group_identifier
                .into_iter()
                .for_each(|spawned_entity| {
                    if let Some(element_kill_callback) = element_kill_callbacks.get(spawned_entity)
                    {
                        let system = element_kill_callback.system.clone();
                        commands
                            .add(move |world: &World| (system.lock().unwrap().run)(world).unwrap());
                    } else {
                        entities.kill(spawned_entity);
                    }
                });
        }
        entities.kill(spawner_entity);

        // remove the spawner_entity from spawners so that subsequent calls to this function behave appropriately
        self.spawners.remove(spawner_entity);
    }
}

pub fn install(session: &mut CoreSession) {
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
    snail::install(session);
    fish_school::install(session);
    kick_bomb::install(session);
    mine::install(session);
    musket::install(session);
    stomp_boots::install(session);
    crate_item::install(session);
    slippery_seaweed::install(session);
    slippery::install(session);
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
