use std::fmt;

use serde::Deserialize;
use serde::de::{self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};

use crate::MAX_LICENSES;
use crate::sink::{DUPLICATE, LIMIT, Sink};
use crate::text::Text;

#[derive(Deserialize, Default)]
#[serde(default)]
struct Named {
    id: Text,
    name: Text,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct Entry {
    license: Named,
    expression: Text,
}

#[derive(Default)]
struct Licenses(Vec<String>);

impl Entry {
    fn value(self) -> Option<String> {
        [self.license.id, self.license.name, self.expression]
            .into_iter()
            .filter_map(|text| text.0)
            .map(|value| value.trim().to_owned())
            .find(|value| !value.is_empty())
    }
}

impl<'de> Deserialize<'de> for Licenses {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Collect;

        impl<'de> Visitor<'de> for Collect {
            type Value = Licenses;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a list of licenses")
            }

            fn visit_unit<E: de::Error>(self) -> Result<Licenses, E> {
                Ok(Licenses::default())
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Licenses, A::Error> {
                let mut kept = Vec::new();
                while let Some(entry) = seq.next_element::<Entry>()? {
                    if kept.len() < MAX_LICENSES
                        && let Some(value) = entry.value()
                    {
                        kept.push(value);
                    }
                }
                Ok(Licenses(kept))
            }
        }

        deserializer.deserialize_any(Collect)
    }
}

#[derive(Clone, Copy)]
enum Field {
    Kind,
    Name,
    Version,
    Purl,
    Licenses,
    Components,
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

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Field, E> {
                Ok(match value {
                    "type" => Field::Kind,
                    "name" => Field::Name,
                    "version" => Field::Version,
                    "purl" => Field::Purl,
                    "licenses" => Field::Licenses,
                    "components" => Field::Components,
                    _ => Field::Other,
                })
            }
        }

        deserializer.deserialize_identifier(Names)
    }
}

struct Components<'a>(&'a mut Sink);

impl<'de> DeserializeSeed<'de> for Components<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Components<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a list of components")
    }

    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while seq.next_element_seed(Component(&mut *self.0))?.is_some() {}
        Ok(())
    }
}

struct Component<'a>(&'a mut Sink);

impl<'de> DeserializeSeed<'de> for Component<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for Component<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a component")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<(), A::Error> {
        let mut kind = Text::default();
        let mut name = Text::default();
        let mut version = Text::default();
        let mut purl = Text::default();
        let mut licenses = Licenses::default();
        let mut seen = [false; 6];
        while let Some(field) = map.next_key::<Field>()? {
            if !matches!(field, Field::Other) && std::mem::replace(&mut seen[field as usize], true)
            {
                return Err(de::Error::custom(DUPLICATE));
            }
            match field {
                Field::Kind => kind = map.next_value()?,
                Field::Name => name = map.next_value()?,
                Field::Version => version = map.next_value()?,
                Field::Purl => purl = map.next_value()?,
                Field::Licenses => licenses = map.next_value()?,
                Field::Components => map.next_value_seed(Components(&mut *self.0))?,
                Field::Other => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        let file = kind
            .0
            .as_deref()
            .is_some_and(|kind| kind.eq_ignore_ascii_case("file"));
        if file && purl.0.as_deref().is_none_or(str::is_empty) {
            return Ok(());
        }
        self.0
            .add(name, version, purl.0, licenses.0)
            .map_err(|()| de::Error::custom(LIMIT))
    }
}

enum Top {
    Format,
    Version,
    Components,
    Other,
}

impl<'de> Deserialize<'de> for Top {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Names;

        impl Visitor<'_> for Names {
            type Value = Top;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a field name")
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Top, E> {
                Ok(match value {
                    "bomFormat" => Top::Format,
                    "specVersion" => Top::Version,
                    "components" => Top::Components,
                    _ => Top::Other,
                })
            }
        }

        deserializer.deserialize_identifier(Names)
    }
}

pub(crate) struct Header {
    pub(crate) format: Option<String>,
    pub(crate) version: Option<String>,
}

pub(crate) struct Document<'a>(pub(crate) &'a mut Sink);

impl<'de> DeserializeSeed<'de> for Document<'_> {
    type Value = Header;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Header, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for Document<'_> {
    type Value = Header;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a CycloneDX document")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Header, A::Error> {
        let mut format = Text::default();
        let mut version = Text::default();
        while let Some(field) = map.next_key::<Top>()? {
            match field {
                Top::Format => format = map.next_value()?,
                Top::Version => version = map.next_value()?,
                Top::Components => map.next_value_seed(Components(&mut *self.0))?,
                Top::Other => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(Header {
            format: format.0,
            version: version.0,
        })
    }
}
