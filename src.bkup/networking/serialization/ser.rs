use bevy::reflect::{serde::Serializable, ReflectRef, TypeRegistryInternal};
use serde::ser::{Error, SerializeMap, SerializeSeq, SerializeTuple};

use super::StringCache;
use crate::prelude::*;

pub struct CompactReflectSerializer<'a>(SerializerInner<'a>);

#[derive(Clone, Copy)]
struct SerializerInner<'a> {
    value: &'a dyn Reflect,
    registry: &'a TypeRegistryInternal,
    type_names: &'a StringCache,
}

impl<'a> SerializerInner<'a> {
    fn serializable<E: serde::ser::Error>(&self) -> Result<Serializable, E> {
        let reflect_serialize = self
            .registry
            .get_type_data::<ReflectSerialize>(self.value.type_id())
            .ok_or_else(|| {
                serde::ser::Error::custom(format_args!(
                    "Type '{}' did not register ReflectSerialize",
                    self.value.type_name()
                ))
            })?;
        Ok(reflect_serialize.get_serializable(self.value))
    }

    fn for_value<'b>(&self, value: &'a dyn Reflect) -> SerializerInner<'b>
    where
        'a: 'b,
    {
        let mut s = *self;
        s.value = value;
        s
    }
}

impl<'a> CompactReflectSerializer<'a> {
    pub fn new(
        value: &'a dyn Reflect,
        registry: &'a TypeRegistryInternal,
        type_name_cache: &'a StringCache,
    ) -> Self {
        CompactReflectSerializer(SerializerInner {
            value,
            registry,
            type_names: type_name_cache,
        })
    }

    fn value_serializer(&self) -> ValueSerializer {
        ValueSerializer(self.0)
    }
}

impl<'a> Serialize for CompactReflectSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut tup = serializer.serialize_tuple(2)?;

        let type_name = self.0.value.type_name();
        let type_idx = self.0.type_names.get_idx(type_name).ok_or_else(|| {
            Error::custom(format_args!("Type name not in string cache: {}", type_name))
        })?;
        tup.serialize_element(&type_idx)?;
        tup.serialize_element(&self.value_serializer())?;

        tup.end()
    }
}

struct ValueSerializer<'a>(SerializerInner<'a>);

impl<'a> Serialize for ValueSerializer<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let serializable = self.0.serializable::<S::Error>();
        if let Ok(serializable) = serializable {
            return serializable.borrow().serialize(serializer);
        }

        trace!("Serializing {}", self.0.value.type_name());

        match self.0.value.reflect_ref() {
            ReflectRef::Struct(s) => {
                let len = s.field_len();
                let mut tup = serializer.serialize_tuple(len)?;
                for (i, field) in s.iter_fields().enumerate() {
                    trace!("Serializing field named `{:?}`", s.name_at(i).unwrap());
                    let field_serializer = ValueSerializer(SerializerInner {
                        value: field,
                        ..self.0
                    });

                    tup.serialize_element(&field_serializer)?;
                }
                tup.end()
            }
            ReflectRef::TupleStruct(s) => {
                let len = s.field_len();
                let mut tup = serializer.serialize_tuple(len)?;
                for field in s.iter_fields() {
                    let field_serializer = ValueSerializer(SerializerInner {
                        value: field,
                        ..self.0
                    });

                    tup.serialize_element(&field_serializer)?;
                }
                tup.end()
            }
            ReflectRef::Tuple(t) => {
                let len = t.field_len();
                let mut tup = serializer.serialize_tuple(len)?;
                for field in t.iter_fields() {
                    let field_serializer = ValueSerializer(SerializerInner {
                        value: field,
                        ..self.0
                    });

                    tup.serialize_element(&field_serializer)?;
                }
                tup.end()
            }
            ReflectRef::List(l) => {
                let len = l.len();
                let mut seq = serializer.serialize_seq(Some(len))?;
                for field in l.iter() {
                    let field_serializer = ValueSerializer(SerializerInner {
                        value: field,
                        ..self.0
                    });

                    seq.serialize_element(&field_serializer)?;
                }
                seq.end()
            }
            ReflectRef::Array(a) => {
                let len = a.len();
                let mut seq = serializer.serialize_seq(Some(len))?;
                for field in a.iter() {
                    let field_serializer = ValueSerializer(self.0.for_value(field));

                    seq.serialize_element(&field_serializer)?;
                }
                seq.end()
            }
            ReflectRef::Map(m) => {
                let len = m.len();
                let mut map = serializer.serialize_map(Some(len))?;
                for (key, value) in m.iter() {
                    map.serialize_entry(
                        &ValueSerializer(self.0.for_value(key)),
                        &ValueSerializer(self.0.for_value(value)),
                    )?;
                }

                map.end()
            }
            ReflectRef::Value(_) => Err(serializable.err().unwrap()),
        }
    }
}
