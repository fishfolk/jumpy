use bevy::{ecs::component::ComponentId, prelude::World, reflect::TypeRegistryInternal};
use bevy_ecs_dynamic::dynamic_query::{DynamicQuery, FetchKind, FilterKind, QueryError};

use crate::{networking::serialization::TypeNameCache, prelude::*};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DynamicQueryInfo {
    fetches: Vec<FetchKindInfo>,
    filters: Vec<FilterKindInfo>,
}

/// References to the types needed to convert and construct [`DynamicQueryInfo`].
#[derive(Copy, Clone)]
pub struct DynamicQueryInfoResources<'a> {
    pub world: &'a World,
    pub type_names: &'a TypeNameCache,
    pub type_registry: &'a TypeRegistryInternal,
}

impl DynamicQueryInfo {
    pub fn from_query(query: &DynamicQuery, info: DynamicQueryInfoResources) -> Self {
        let fetches = query
            .fetches()
            .iter()
            .cloned()
            .map(|x| FetchKindInfo::from_fetch_kind(x, info))
            .collect();
        let filters = query
            .filters()
            .iter()
            .cloned()
            .map(|x| FilterKindInfo::from_filter_kind(x, info))
            .collect();
        Self { fetches, filters }
    }

    pub fn with_all_fetches_mutable(mut self) -> Self {
        for fetch in &mut self.fetches {
            *fetch = FetchKindInfo::RefMut(match fetch {
                FetchKindInfo::Ref(id) | FetchKindInfo::RefMut(id) => *id,
            });
        }
        self
    }

    pub fn get_query(
        self: DynamicQueryInfo,
        info: DynamicQueryInfoResources,
    ) -> Result<DynamicQuery, QueryError> {
        let fetches = self
            .fetches
            .into_iter()
            .map(|x| x.get_fetch_kind(info))
            .collect();
        let filters = self
            .filters
            .into_iter()
            .map(|x| x.get_filter_kind(info))
            .collect();
        DynamicQuery::new(info.world, fetches, filters)
    }
}

/// The index into the type name cache for this component
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct ComponentTypeNameIdx(pub u32);

impl ComponentTypeNameIdx {
    fn get_component_id(&self, info: DynamicQueryInfoResources) -> ComponentId {
        let type_registry = info.type_registry;

        let type_name = info
            .type_names
            .0
            .get_str(self.0)
            .expect("Missing type name in cache");
        let type_id = type_registry
            .get_with_name(type_name)
            .expect("Type not registered")
            .type_id();

        // FIXME(scripting): This can't support custom components created by scripts, because we
        // assume the component ID can be directly tied to the type ID that we are syncing.
        // We need to fix that and find a way to map component Ids on the server to component ids on the client
        info.world
            .components()
            .get_id(type_id)
            .expect("Component not found")
    }

    fn from_component_id(id: ComponentId, info: DynamicQueryInfoResources) -> Self {
        let world = info.world;
        let type_names = info.type_names;
        let type_registry = info.type_registry;

        let type_id = world
            .components()
            .get_info(id)
            .expect("Component ID not found")
            .type_id()
            .expect("Component without Type ID");
        let type_name = type_registry
            .get(type_id)
            .expect("Type not registered")
            .type_name();

        Self(
            type_names
                .0
                .get_idx(type_name)
                .expect("Missing type name in cache"),
        )
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum FetchKindInfo {
    Ref(ComponentTypeNameIdx),
    RefMut(ComponentTypeNameIdx),
}

impl FetchKindInfo {
    fn get_fetch_kind(&self, info: DynamicQueryInfoResources) -> FetchKind {
        match self {
            FetchKindInfo::Ref(id) => FetchKind::Ref(id.get_component_id(info)),
            FetchKindInfo::RefMut(id) => FetchKind::RefMut(id.get_component_id(info)),
        }
    }

    fn from_fetch_kind(f: FetchKind, info: DynamicQueryInfoResources) -> Self {
        match f {
            FetchKind::Ref(id) => Self::Ref(ComponentTypeNameIdx::from_component_id(id, info)),
            FetchKind::RefMut(id) => {
                Self::RefMut(ComponentTypeNameIdx::from_component_id(id, info))
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum FilterKindInfo {
    With(ComponentTypeNameIdx),
    Without(ComponentTypeNameIdx),
    Changed(ComponentTypeNameIdx),
    Added(ComponentTypeNameIdx),
}

impl FilterKindInfo {
    fn get_filter_kind(&self, info: DynamicQueryInfoResources) -> FilterKind {
        match self {
            FilterKindInfo::With(id) => FilterKind::With(id.get_component_id(info)),
            FilterKindInfo::Without(id) => FilterKind::Without(id.get_component_id(info)),
            FilterKindInfo::Changed(id) => FilterKind::Changed(id.get_component_id(info)),
            FilterKindInfo::Added(id) => FilterKind::Added(id.get_component_id(info)),
        }
    }

    fn from_filter_kind(f: FilterKind, info: DynamicQueryInfoResources) -> Self {
        match f {
            FilterKind::With(id) => Self::With(ComponentTypeNameIdx::from_component_id(id, info)),
            FilterKind::Without(id) => {
                Self::Without(ComponentTypeNameIdx::from_component_id(id, info))
            }
            FilterKind::Changed(id) => {
                Self::Changed(ComponentTypeNameIdx::from_component_id(id, info))
            }
            FilterKind::Added(id) => Self::Added(ComponentTypeNameIdx::from_component_id(id, info)),
        }
    }
}
