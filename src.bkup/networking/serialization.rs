use crate::prelude::*;

use bevy::{reflect::TypeRegistry, utils::HashMap};

pub mod de;
pub mod ser;

pub struct SerializationPlugin;

impl Plugin for SerializationPlugin {
    fn build(&self, app: &mut App) {
        // This type isn't registered
        app.register_type::<bevy::math::Affine3A>();
    }
}

pub fn serialize_to_bytes<T>(value: &T) -> anyhow::Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    Ok(postcard::to_allocvec(value)?)
}

pub fn deserialize_from_bytes<'a, T>(bytes: &'a [u8]) -> anyhow::Result<T>
where
    T: Deserialize<'a>,
{
    Ok(postcard::from_bytes(bytes)?)
}

pub fn deserializer_from_bytes(
    bytes: &[u8],
) -> postcard::Deserializer<postcard::de_flavors::Slice> {
    postcard::Deserializer::from_bytes(bytes)
}

#[derive(Default, Serialize, Deserialize)]
pub struct TypeNameCache(pub StringCache);

/// String cache that can be used to map strings to indices in order to save space.
#[derive(Debug, Clone, Reflect, Default, Serialize, Deserialize)]
#[reflect(Default)]
pub struct StringCache {
    strings: Vec<String>,
    indexes: HashMap<String, u32>,
}

impl StringCache {
    pub fn insert(&mut self, s: &str) {
        let idx = self.strings.len();
        self.strings.push(s.into());
        self.indexes
            .insert(s.into(), idx.try_into().expect("Too many strings in cache"));
    }

    pub fn get_idx(&self, str: &str) -> Option<u32> {
        self.indexes.get(str).copied()
    }

    pub fn get_str(&self, idx: u32) -> Option<&str> {
        self.strings.get(idx as usize).map(|x| x.as_str())
    }
}

pub fn get_type_name_cache(type_registry: &TypeRegistry) -> TypeNameCache {
    let mut cache = StringCache::default();

    let type_registry = type_registry.read();
    for registration in type_registry.iter() {
        cache.insert(registration.type_name());
    }

    TypeNameCache(cache)
}
