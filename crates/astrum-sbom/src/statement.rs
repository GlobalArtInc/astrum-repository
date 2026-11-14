use std::fmt;

use serde::Deserialize;
use serde::de::{DeserializeSeed, Deserializer, IgnoredAny, MapAccess, Visitor};

enum Field {
    Predicate,
    Other,
}

impl<'de> Deserialize<'de> for Field {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Names;

        impl Visitor<'_> for Names {
            type Value = Field;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a field name")
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Field, E> {
                Ok(match value {
                    "predicate" => Field::Predicate,
                    _ => Field::Other,
                })
            }
        }

        deserializer.deserialize_identifier(Names)
    }
}

pub(crate) struct Predicate<S>(pub(crate) S);

impl<'de, S: DeserializeSeed<'de>> DeserializeSeed<'de> for Predicate<S> {
    type Value = Option<S::Value>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de, S: DeserializeSeed<'de>> Visitor<'de> for Predicate<S> {
    type Value = Option<S::Value>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an in-toto statement")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut seed = Some(self.0);
        let mut found = None;
        while let Some(field) = map.next_key::<Field>()? {
            match field {
                Field::Predicate => match seed.take() {
                    Some(seed) => found = Some(map.next_value_seed(seed)?),
                    None => {
                        map.next_value::<IgnoredAny>()?;
                    }
                },
                Field::Other => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(found)
    }
}
