use std::fmt;

use serde::Deserialize;
use serde::de::{self, DeserializeSeed, Deserializer, IgnoredAny, MapAccess, SeqAccess, Visitor};

use crate::sink::{LIMIT, Sink};
use crate::text::Text;

const NOASSERTION: &str = "NOASSERTION";
const NONE: &str = "NONE";
const NOT_LIBRARIES: [&str; 3] = ["FILE", "APPLICATION", "SOURCE"];

#[derive(Default)]
struct Purl(Option<String>);

#[derive(Deserialize, Default)]
#[serde(default)]
struct Reference {
    #[serde(rename = "referenceType")]
    kind: Text,
    #[serde(rename = "referenceLocator")]
    locator: Text,
}

#[derive(Deserialize, Default)]
#[serde(default)]
struct RawPackage {
    name: Text,
    #[serde(rename = "versionInfo")]
    version: Text,
    #[serde(rename = "primaryPackagePurpose")]
    purpose: Text,
    #[serde(rename = "externalRefs")]
    references: Purl,
    #[serde(rename = "licenseConcluded")]
    concluded: Text,
    #[serde(rename = "licenseDeclared")]
    declared: Text,
}

impl<'de> Deserialize<'de> for Purl {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Scan;

        impl<'de> Visitor<'de> for Scan {
            type Value = Purl;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a list of references")
            }

            fn visit_unit<E: de::Error>(self) -> Result<Purl, E> {
                Ok(Purl(None))
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Purl, A::Error> {
                let mut found = None;
                while let Some(reference) = seq.next_element::<Reference>()? {
                    if found.is_none()
                        && reference.kind.0.as_deref() == Some("purl")
                        && let Some(locator) = reference.locator.0
                    {
                        found = Some(locator);
                    }
                }
                Ok(Purl(found))
            }
        }

        deserializer.deserialize_any(Scan)
    }
}

fn license(concluded: &Text, declared: &Text) -> Vec<String> {
    [concluded, declared]
        .into_iter()
        .filter_map(|text| text.0.as_deref())
        .map(str::trim)
        .find(|value| !value.is_empty() && *value != NOASSERTION && *value != NONE)
        .map(|value| vec![value.to_owned()])
        .unwrap_or_default()
}

struct Packages<'a>(&'a mut Sink);

impl<'de> DeserializeSeed<'de> for Packages<'_> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<(), D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de> Visitor<'de> for Packages<'_> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a list of packages")
    }

    fn visit_unit<E: de::Error>(self) -> Result<(), E> {
        Ok(())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        while let Some(package) = seq.next_element::<RawPackage>()? {
            let licenses = license(&package.concluded, &package.declared);
            let purl = package.references.0.filter(|purl| !purl.is_empty());
            let skipped = purl.is_none()
                && package.purpose.0.as_deref().is_some_and(|purpose| {
                    NOT_LIBRARIES
                        .iter()
                        .any(|kind| purpose.eq_ignore_ascii_case(kind))
                });
            if skipped {
                continue;
            }
            self.0
                .add(package.name, package.version, purl, licenses)
                .map_err(|()| de::Error::custom(LIMIT))?;
        }
        Ok(())
    }
}

enum Field {
    Version,
    Packages,
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
                    "spdxVersion" => Field::Version,
                    "packages" => Field::Packages,
                    _ => Field::Other,
                })
            }
        }

        deserializer.deserialize_identifier(Names)
    }
}

pub(crate) struct Document<'a>(pub(crate) &'a mut Sink);

impl<'de> DeserializeSeed<'de> for Document<'_> {
    type Value = Option<String>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for Document<'_> {
    type Value = Option<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an SPDX document")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut version = Text::default();
        while let Some(field) = map.next_key::<Field>()? {
            match field {
                Field::Version => version = map.next_value()?,
                Field::Packages => map.next_value_seed(Packages(&mut *self.0))?,
                Field::Other => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(version.0)
    }
}
