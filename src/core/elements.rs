//! Map element implementations.
//!
//! A map element is anything that can be placed on the map in the editor.

use std::sync::Mutex;

use crate::{impl_system_param, prelude::*};

pub mod crab;
pub mod crate_item;
pub mod decoration;
pub mod fish_school;
pub mod flappy_jellyfish;
pub mod grenade;
pub mod jellyfish;
pub mod kick_bomb;
pub mod mine;
pub mod musket;
pub mod player_spawner;
pub mod slippery;
pub mod slippery_seaweed;
pub mod snail;
pub mod spike;
pub mod sproinger;
pub mod stomp_boots;
pub mod sword;
pub mod urchin;

pub mod prelude {
    pub use super::{
        crab::*, crate_item::*, decoration::*, fish_school::*, grenade::*, jellyfish::*,
        kick_bomb::*, mine::*, musket::*, player_spawner::*, slippery::*, slippery_seaweed::*,
        snail::*, spike::*, sproinger::*, stomp_boots::*, sword::*, urchin::*, *,
    };
}

#[derive(HasSchema, Default, Clone, Debug)]
#[type_data(metadata_asset("element"))]
#[repr(C)]
pub struct ElementMeta {
    pub name: Ustr,
    pub category: Ustr,
    pub data: Handle<SchemaBox>,
    pub editor: ElementEditorMeta,
    pub plugin: Handle<LuaPlugin>,
}

#[derive(HasSchema, Default, Debug, Clone, Copy)]
#[type_data(metadata_asset("solid"))]
#[repr(C)]
pub struct ElementSolidMeta {
    pub disabled: bool,
    pub offset: Vec2,
    pub size: Vec2,
}

#[derive(HasSchema, Deserialize, Clone, Debug)]
#[repr(C)]
pub struct ElementEditorMeta {
    /// The size of the bounding rect for the element in the editor
    pub grab_size: Vec2,
    /// The offset of the bounding rect for the element in the editor.
    pub grab_offset: Vec2,
    /// Show the element name above the bounding rect in the editor.
    pub show_name: bool,
}

impl Default for ElementEditorMeta {
    fn default() -> Self {
        Self {
            grab_size: Vec2::splat(45.0),
            grab_offset: Vec2::ZERO,
            show_name: true,
        }
    }
}

/// Marker component added to map elements that have been hydrated.
#[derive(Clone, HasSchema, Default)]
#[repr(C)]
pub struct MapElementHydrated;

/// Component that contains the [`Entity`] to de-hydrate when the entity with this component is out
/// of the [`LoadedMap`] bounds.
///
/// This is useful for map elements that spawn items: when the item falls off the map, it should
/// de-hydrate it's spawner, so that the spawner will re-spawn the item in it's default state.
#[derive(Clone, HasSchema, Default, Deref, DerefMut)]
#[repr(C)]
pub struct DehydrateOutOfBounds(pub Entity);

/// Component containing an element's metadata handle.
#[derive(Clone, Copy, HasSchema, Default, Deref, DerefMut)]
#[repr(C)]
pub struct ElementHandle(pub Handle<ElementMeta>);

#[derive(Clone, HasSchema, Default, Deref, DerefMut)]
#[repr(C)]
pub struct ElementSolid(pub Entity);

#[derive(Clone, HasSchema)]
#[schema(no_default)]
pub struct ElementKillCallback {
    pub system: Arc<Mutex<StaticSystem<(), ()>>>,
}

impl ElementKillCallback {
    pub fn new<Args>(system: impl IntoSystem<Args, (), (), Sys = StaticSystem<(), ()>>) -> Self {
        ElementKillCallback {
            system: Arc::new(Mutex::new(system.system())),
        }
    }
}

#[derive(Clone, HasSchema)]
pub struct Spawner {
    /// The group identifier where all of the elements are meant to be shared between them
    pub group_identifier: String,
}

impl Default for Spawner {
    fn default() -> Self {
        Self {
            group_identifier: Ulid::create().to_string(),
        }
    }
}

impl Spawner {
    pub fn new() -> Self {
        Spawner {
            group_identifier: Ulid::create().to_string(),
        }
    }
    pub fn new_grouped(group_identifier: String) -> Self {
        Spawner { group_identifier }
    }
}

#[derive(HasSchema, Default, Clone)]
pub struct SpawnerEntities {
    pub entities_per_spawner_group_identifier: HashMap<String, Vec<Entity>>,
}

impl_system_param! {
    pub struct SpawnerManager<'a> {
        spawners: CompMut<'a, Spawner>,
        spawner_entities: ResMutInit<'a, SpawnerEntities>,
    }
}

impl<'a> SpawnerManager<'a> {
    /// Stores the spawned elements as having been spawned by the provided entity, finding an
    /// existing spawner element, if it exists, to group these spawned elements with.
    pub fn create_grouped_spawner<T: HasSchema>(
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
    pub fn insert_spawned_entity_into_grouped_spawner<T: HasSchema>(
        &mut self,
        spawned_entity: Entity,
        spawner_elements: &Comp<T>,
        entities: &ResMutInit<Entities>,
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
    pub fn remove_spawned_entity_from_grouped_spawner<T: HasSchema>(
        &mut self,
        spawned_entity: Entity,
        spawner_elements: &Comp<T>,
        entities: &ResMutInit<Entities>,
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
        entities: &mut ResMutInit<Entities>,
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
                        commands.add(move |world: &World| (system.lock().unwrap().run)(world, ()));
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

/// Helper macro to install element game and session plugins
macro_rules! install_plugins {
    ($($module:ident),* $(,)?) => {
        pub fn session_plugin(session: &mut Session) {
            ElementHandle::register_schema();
            MapElementHydrated::register_schema();
            DehydrateOutOfBounds::register_schema();

            session
                .stages
                .add_system_to_stage(CoreStage::First, handle_out_of_bounds_items);

            $(
                session.install_plugin($module::session_plugin);
            )*
        }

        pub fn game_plugin(game: &mut Game) {
            ElementMeta::register_schema();
            game.init_shared_resource::<AssetServer>();

            $(
                game.install_plugin($module::game_plugin);
            )*
        }
    };
}

install_plugins!(
    crab,
    crate_item,
    decoration,
    fish_school,
    grenade,
    jellyfish,
    kick_bomb,
    mine,
    musket,
    player_spawner,
    slippery_seaweed,
    slippery,
    snail,
    spike,
    sproinger,
    stomp_boots,
    sword,
    urchin,
);

fn handle_out_of_bounds_items(
    mut commands: Commands,
    mut hydrated: CompMut<MapElementHydrated>,
    entities: ResMutInit<Entities>,
    transforms: CompMut<Transform>,
    spawners: Comp<DehydrateOutOfBounds>,
    map: Res<LoadedMap>,
) {
    for (item_ent, (transform, spawner)) in entities.iter_with((&transforms, &spawners)) {
        if map.is_out_of_bounds(&transform.translation) {
            hydrated.remove(**spawner);
            commands.add(move |mut entities: ResMutInit<Entities>| {
                entities.kill(item_ent);
            });
        }
    }
}
