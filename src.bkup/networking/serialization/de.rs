use std::any::TypeId;

use bevy::reflect::{
    ArrayInfo, DynamicList, DynamicMap, DynamicStruct, DynamicTuple, DynamicTupleStruct, ListInfo,
    Map, MapInfo, StructInfo, TupleInfo, TupleStructInfo, TypeInfo, TypeRegistration,
    TypeRegistryInternal,
};
use serde::de::{DeserializeSeed, Error, Visitor};

use super::StringCache;
use crate::prelude::*;

macro_rules! custom_err {
    ($($arg:expr),* $(,)?) => {
        Error::custom(format_args!($($arg),*))
    };
}

pub struct CompactReflectDeserializer<'a>(DeserializerInner<'a>);

impl<'a> CompactReflectDeserializer<'a> {
    pub fn new(registry: &'a TypeRegistryInternal, type_names: &'a StringCache) -> Self {
        Self(DeserializerInner {
            registry,
            type_names,
        })
    }
}

#[derive(Clone, Copy)]
struct DeserializerInner<'a> {
    registry: &'a TypeRegistryInternal,
    type_names: &'a StringCache,
}

impl<'a> DeserializerInner<'a> {
    fn get_registration_from_idx<E: Error>(&self, type_idx: u32) -> Result<&TypeRegistration, E> {
        let type_name = self
            .type_names
            .get_str(type_idx)
            .ok_or_else(|| custom_err!("Type name not in name cache"))?;

        self.registry
            .get_with_name(type_name)
            .ok_or_else(|| custom_err!("Type not in type registry"))
    }

    fn get_registration_from_type_id<E: Error>(
        &self,
        type_id: TypeId,
        type_name: &str,
    ) -> Result<&TypeRegistration, E> {
        self.registry
            .get(type_id)
            .ok_or_else(|| custom_err!("Type not in type registry: {}", type_name))
    }
}

impl<'a, 'de> DeserializeSeed<'de> for CompactReflectDeserializer<'a> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_tuple(2, OuterTypeVisitor(self.0))
    }
}

struct OuterTypeVisitor<'a>(DeserializerInner<'a>);

impl<'a, 'de> Visitor<'de> for OuterTypeVisitor<'a> {
    type Value = Box<dyn Reflect>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Tuple like (type_idx_in_str_cache, type_data)")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let type_idx = seq
            .next_element::<u32>()?
            .ok_or_else(|| custom_err!("Missing type idx in top-level type"))?;
        let registration = self.0.get_registration_from_idx(type_idx)?;

        let reflect = seq
            .next_element_seed(ReflectValueDeserializer {
                inner: self.0,
                registration,
            })?
            .ok_or_else(|| custom_err!("Missing type idx in top-level type"))?;

        Ok(reflect)
    }
}

#[derive(Copy, Clone)]
struct ReflectValueDeserializer<'a> {
    inner: DeserializerInner<'a>,
    registration: &'a TypeRegistration,
}

impl<'a, 'de> DeserializeSeed<'de> for ReflectValueDeserializer<'a> {
    type Value = Box<dyn Reflect>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Handle both Value case and types that have a custom `ReflectDeserialize`
        if let Some(deserialize_reflect) = self.registration.data::<ReflectDeserialize>() {
            let value = deserialize_reflect.deserialize(deserializer)?;
            return Ok(value);
        }

        match self.registration.type_info() {
            TypeInfo::Struct(info) => {
                let len = info.field_len();
                let mut dynamic_struct = deserializer.deserialize_tuple(
                    len,
                    StructVisitor {
                        inner: self.inner,
                        info,
                    },
                )?;
                dynamic_struct.set_name(info.type_name().into());

                Ok(Box::new(dynamic_struct))
            }
            TypeInfo::TupleStruct(info) => {
                let len = info.field_len();
                let mut dynamic_tuple_struct = deserializer.deserialize_tuple(
                    len,
                    TupleStructVisitor {
                        inner: self.inner,
                        info,
                    },
                )?;
                dynamic_tuple_struct.set_name(info.type_name().into());

                Ok(Box::new(dynamic_tuple_struct))
            }
            TypeInfo::Tuple(info) => {
                let len = info.field_len();
                let mut dynamic_tuple = deserializer.deserialize_tuple(
                    len,
                    TupleVisitor {
                        inner: self.inner,
                        info,
                    },
                )?;
                dynamic_tuple.set_name(info.type_name().into());

                Ok(Box::new(dynamic_tuple))
            }
            TypeInfo::List(info) => {
                let mut dynamic_list = deserializer.deserialize_seq(ListVisitor {
                    inner: self.inner,
                    info,
                })?;
                dynamic_list.set_name(info.type_name().into());

                Ok(Box::new(dynamic_list))
            }
            TypeInfo::Array(info) => {
                let mut dynamic_list = deserializer.deserialize_seq(ArrayVisitor {
                    inner: self.inner,
                    info,
                })?;
                dynamic_list.set_name(info.type_name().into());

                Ok(Box::new(dynamic_list))
            }
            TypeInfo::Map(info) => {
                let mut dynamic_map = deserializer.deserialize_map(MapVisitor {
                    inner: self.inner,
                    info,
                })?;
                dynamic_map.set_name(info.type_name().into());

                Ok(Box::new(dynamic_map))
            }
            TypeInfo::Value(_) => {
                // This case should already be handled
                Err(custom_err!(
                    "the TypeRegistration for {} doesn't have ReflectDeserialize",
                    self.registration.type_name()
                ))
            }
            TypeInfo::Dynamic(_) => {
                // We could potentially allow this but we'd have no idea what the actual types of the
                // fields are and would rely on the deserializer to determine them (e.g. `i32` vs `i64`)
                Err(custom_err!(
                    "cannot deserialize arbitrary dynamic type yet: {}",
                    self.registration.type_name()
                ))
            }
        }
    }
}

struct StructVisitor<'a> {
    inner: DeserializerInner<'a>,
    info: &'a StructInfo,
}

impl<'a, 'de> Visitor<'de> for StructVisitor<'a> {
    type Value = DynamicStruct;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Tuple with struct fields")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        trace!("Visiting struct {}", self.info.type_name());
        let mut index = 0usize;
        let mut output = DynamicStruct::default();

        while let Some(value) = seq.next_element_seed({
            let field = self.info.field_at(index).unwrap();
            let type_id = field.type_id();
            ReflectValueDeserializer {
                inner: self.inner,
                registration: self
                    .inner
                    .get_registration_from_type_id(type_id, field.type_name())?,
            }
        })? {
            let name = self.info.field_at(index).unwrap().name();
            trace!("Visiting field {}", name);
            output.insert_boxed(name, value);
            index += 1;
            if index >= self.info.field_len() {
                break;
            }
        }

        Ok(output)
    }
}

struct TupleStructVisitor<'a> {
    inner: DeserializerInner<'a>,
    info: &'a TupleStructInfo,
}

impl<'a, 'de> Visitor<'de> for TupleStructVisitor<'a> {
    type Value = DynamicTupleStruct;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Tuple with tuple struct fields")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        trace!("Visiting tuple struct {}", self.info.type_name());
        let mut index = 0usize;
        let mut output = DynamicTupleStruct::default();

        while let Some(value) = seq.next_element_seed({
            let field = self.info.field_at(index).unwrap();
            let type_id = field.type_id();
            ReflectValueDeserializer {
                inner: self.inner,
                registration: self
                    .inner
                    .get_registration_from_type_id(type_id, field.type_name())?,
            }
        })? {
            output.insert_boxed(value);
            index += 1;
            if index >= self.info.field_len() {
                break;
            }
        }

        Ok(output)
    }
}

struct TupleVisitor<'a> {
    inner: DeserializerInner<'a>,
    info: &'a TupleInfo,
}

impl<'a, 'de> Visitor<'de> for TupleVisitor<'a> {
    type Value = DynamicTuple;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Tuple with tuple fields")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        trace!("Visiting tuple {}", self.info.type_name());
        let mut index = 0usize;
        let mut output = DynamicTuple::default();

        while let Some(value) = seq.next_element_seed({
            let field = self.info.field_at(index).unwrap();
            let type_id = field.type_id();
            ReflectValueDeserializer {
                inner: self.inner,
                registration: self
                    .inner
                    .get_registration_from_type_id(type_id, field.type_name())?,
            }
        })? {
            output.insert_boxed(value);
            index += 1;
            if index >= self.info.field_len() {
                break;
            }
        }

        Ok(output)
    }
}

struct ListVisitor<'a> {
    inner: DeserializerInner<'a>,
    info: &'a ListInfo,
}

impl<'a, 'de> Visitor<'de> for ListVisitor<'a> {
    type Value = DynamicList;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Tuple with tuple list fields")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        trace!("Visiting list {}", self.info.type_name());
        let mut output = DynamicList::default();

        let type_id = self.info.item_type_id();

        while let Some(value) = seq.next_element_seed({
            ReflectValueDeserializer {
                inner: self.inner,
                registration: self
                    .inner
                    .get_registration_from_type_id(type_id, self.info.item_type_name())?,
            }
        })? {
            output.push_box(value);
        }

        Ok(output)
    }
}

struct ArrayVisitor<'a> {
    inner: DeserializerInner<'a>,
    info: &'a ArrayInfo,
}

impl<'a, 'de> Visitor<'de> for ArrayVisitor<'a> {
    type Value = DynamicList;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Tuple with tuple list fields")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        trace!("Visiting array {}", self.info.type_name());
        let mut output = DynamicList::default();

        let type_id = self.info.item_type_id();

        while let Some(value) = seq.next_element_seed({
            ReflectValueDeserializer {
                inner: self.inner,
                registration: self
                    .inner
                    .get_registration_from_type_id(type_id, self.info.item_type_name())?,
            }
        })? {
            output.push_box(value);
        }

        Ok(output)
    }
}

struct MapVisitor<'a> {
    inner: DeserializerInner<'a>,
    info: &'a MapInfo,
}

impl<'a, 'de> Visitor<'de> for MapVisitor<'a> {
    type Value = DynamicMap;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("map")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        trace!("Visiting map {}", self.info.type_name());
        let mut output = DynamicMap::default();

        let key_registration = self
            .inner
            .get_registration_from_type_id(self.info.key_type_id(), self.info.key_type_name())?;
        let value_registration = self.inner.get_registration_from_type_id(
            self.info.value_type_id(),
            self.info.value_type_name(),
        )?;

        while let Some(key) = map.next_key_seed(ReflectValueDeserializer {
            inner: self.inner,
            registration: key_registration,
        })? {
            let value = map.next_value_seed(ReflectValueDeserializer {
                registration: value_registration,
                inner: self.inner,
            })?;
            output.insert_boxed(key, value);
        }

        Ok(output)
    }
}
