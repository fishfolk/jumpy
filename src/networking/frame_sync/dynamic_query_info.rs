use bevy::{ecs::component::ComponentId, prelude::World};
use bevy_ecs_dynamic::dynamic_query::{DynamicQuery, FetchKind, FilterKind, QueryError};
use serde::{de::Visitor, ser::SerializeTuple, Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DynamicQueryInfo {
    fetches: Vec<FetchKindInfo>,
    filters: Vec<FilterKindInfo>,
}

impl DynamicQueryInfo {
    pub fn from_query(query: &DynamicQuery) -> Self {
        let fetches = query.fetches().iter().cloned().map(Into::into).collect();
        let filters = query.filters().iter().cloned().map(Into::into).collect();
        Self { fetches, filters }
    }

    pub fn with_all_fetches_mutable(mut self) -> Self {
        for fetch in &mut self.fetches {
            *fetch = FetchKindInfo::RefMut(fetch.component_id());
        }
        self
    }

    pub fn get_query(self: DynamicQueryInfo, world: &World) -> Result<DynamicQuery, QueryError> {
        let fetches = self.fetches.into_iter().map(Into::into).collect();
        let filters = self.filters.into_iter().map(Into::into).collect();
        DynamicQuery::new(world, fetches, filters)
    }
}

#[derive(Clone, Debug)]
enum FetchKindInfo {
    Ref(ComponentId),
    RefMut(ComponentId),
}

impl From<FetchKindInfo> for FetchKind {
    fn from(i: FetchKindInfo) -> Self {
        match i {
            FetchKindInfo::Ref(id) => FetchKind::Ref(id),
            FetchKindInfo::RefMut(id) => FetchKind::RefMut(id),
        }
    }
}

impl From<FetchKind> for FetchKindInfo {
    fn from(f: FetchKind) -> Self {
        match f {
            FetchKind::Ref(id) => Self::Ref(id),
            FetchKind::RefMut(id) => Self::RefMut(id),
        }
    }
}

impl FetchKindInfo {
    fn serde_tag(&self) -> u8 {
        match self {
            FetchKindInfo::Ref(_) => 0,
            FetchKindInfo::RefMut(_) => 1,
        }
    }

    fn component_id(&self) -> ComponentId {
        match self {
            FetchKindInfo::Ref(id) | FetchKindInfo::RefMut(id) => *id,
        }
    }
}

impl Serialize for FetchKindInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let tag = self.serde_tag();
        let component_id = self.component_id().index() as u64;
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&tag)?;
        tup.serialize_element(&component_id)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for FetchKindInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (tag, component_idx) = deserializer.deserialize_tuple(2, U8U64SeqVisitor)?;
        let component_id = ComponentId::new(
            component_idx
                .try_into()
                .expect("Component ID to big to fit into 32-bit target"),
        );
        Ok(match tag {
            0 => Self::Ref(component_id),
            1 => Self::RefMut(component_id),
            _ => return Err(serde::de::Error::custom("Invalid tag")),
        })
    }
}

#[derive(Clone, Debug)]
enum FilterKindInfo {
    With(ComponentId),
    Without(ComponentId),
    Changed(ComponentId),
    Added(ComponentId),
}

impl From<FilterKindInfo> for FilterKind {
    fn from(i: FilterKindInfo) -> Self {
        match i {
            FilterKindInfo::With(id) => FilterKind::With(id),
            FilterKindInfo::Without(id) => FilterKind::Without(id),
            FilterKindInfo::Changed(id) => FilterKind::Changed(id),
            FilterKindInfo::Added(id) => FilterKind::Added(id),
        }
    }
}

impl From<FilterKind> for FilterKindInfo {
    fn from(f: FilterKind) -> Self {
        match f {
            FilterKind::With(id) => Self::With(id),
            FilterKind::Without(id) => Self::Without(id),
            FilterKind::Changed(id) => Self::Changed(id),
            FilterKind::Added(id) => Self::Added(id),
        }
    }
}

impl FilterKindInfo {
    fn serde_tag(&self) -> u8 {
        match self {
            FilterKindInfo::With(_) => 0,
            FilterKindInfo::Without(_) => 1,
            FilterKindInfo::Changed(_) => 2,
            FilterKindInfo::Added(_) => 3,
        }
    }

    fn component_id(&self) -> ComponentId {
        match self {
            FilterKindInfo::With(id)
            | FilterKindInfo::Without(id)
            | FilterKindInfo::Changed(id)
            | FilterKindInfo::Added(id) => *id,
        }
    }
}

impl Serialize for FilterKindInfo {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let tag = self.serde_tag();
        let component_id = self.component_id().index() as u64;
        let mut tup = serializer.serialize_tuple(2)?;
        tup.serialize_element(&tag)?;
        tup.serialize_element(&component_id)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for FilterKindInfo {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let (tag, component_idx) = deserializer.deserialize_tuple(2, U8U64SeqVisitor)?;
        let component_id = ComponentId::new(
            component_idx
                .try_into()
                .expect("Component ID to big to fit into 32-bit target"),
        );
        Ok(match tag {
            0 => Self::With(component_id),
            1 => Self::Without(component_id),
            2 => Self::Changed(component_id),
            3 => Self::Added(component_id),
            _ => return Err(serde::de::Error::custom("Invalid tag")),
        })
    }
}

struct U8U64SeqVisitor;

impl<'de> Visitor<'de> for U8U64SeqVisitor {
    type Value = (u8, u64);

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(formatter, "Sequence of a u8 followed by a u64")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let uint1 = seq
            .next_element::<u8>()?
            .ok_or_else(|| serde::de::Error::custom("Expected u8 in sequence"))?;
        let uint2 = seq
            .next_element::<u64>()?
            .ok_or_else(|| serde::de::Error::custom("Expected 64 in sequence"))?;
        Ok((uint1, uint2))
    }
}
