use std::fmt;
use std::marker::PhantomData;

use serde::Deserialize;
use serde::de::value::MapAccessDeserializer;
use serde::de::{DeserializeOwned, Deserializer, MapAccess, Visitor};
use serde_json::error::Category;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum JsonError {
    NotObject,
    Syntax,
    Structure,
}

pub(super) struct Object<T>(pub T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Object<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
            type Value = Object<T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                T::deserialize(MapAccessDeserializer::new(map)).map(Object)
            }
        }

        deserializer.deserialize_map(ObjectVisitor(PhantomData))
    }
}

pub(super) fn parse_object<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, JsonError> {
    let first = bytes
        .iter()
        .find(|byte| !matches!(byte, b' ' | b'\t' | b'\n' | b'\r'));
    if first != Some(&b'{') {
        return Err(JsonError::NotObject);
    }
    serde_json::from_slice::<Object<T>>(bytes)
        .map(|parsed| parsed.0)
        .map_err(|error| match error.classify() {
            Category::Data => JsonError::Structure,
            _ => JsonError::Syntax,
        })
}
