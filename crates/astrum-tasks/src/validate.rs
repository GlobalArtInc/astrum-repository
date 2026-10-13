use astrum_blob::Settings;
use serde_json::{Map, Value};

use crate::kind::{FieldType, Properties, TaskKind};
use crate::kinds::ALL;
use crate::{Problems, Tasks};

#[derive(Clone, Debug, Default)]
pub struct Choices {
    pub repositories: Vec<(String, String, String)>,
    pub blob_stores: Vec<(String, bool)>,
}

impl Tasks {
    pub async fn choices(&self) -> Result<Choices, crate::TaskError> {
        let repositories = astrum_repo::store::list(self.db())
            .await?
            .into_iter()
            .map(|repository| {
                (
                    repository.name.clone(),
                    repository.format.id.to_owned(),
                    repository.kind.as_str().to_owned(),
                )
            })
            .collect();
        let blob_stores = self
            .content()
            .blob_stores()
            .configs()
            .await?
            .into_iter()
            .map(|config| {
                let group = matches!(config.settings, Settings::Group(_));
                (config.name, group)
            })
            .collect();
        Ok(Choices {
            repositories,
            blob_stores,
        })
    }
}

pub fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some(String::new()),
        Value::String(text) => Some(text.trim().to_owned()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Number(number) => Some(number.to_string()),
        _ => None,
    }
}

pub fn properties(
    kind: &dyn TaskKind,
    given: Option<&Map<String, Value>>,
    existing: Option<&Properties>,
    choices: &Choices,
    problems: &mut Problems,
) -> Properties {
    let mut properties = Properties::new();
    for field in kind.fields() {
        let id = format!("properties.{}", field.id);
        let value = match given.and_then(|given| given.get(field.id)) {
            Some(value) => match text_of(value) {
                Some(text) => text,
                None => {
                    problems.add(id, "must be text, a number or true or false");
                    continue;
                }
            },
            None => match existing.and_then(|existing| existing.get(field.id)) {
                Some(stored) => stored.clone(),
                None if given.is_some() || existing.is_none() => {
                    field.initial.unwrap_or_default().to_owned()
                }
                None => String::new(),
            },
        };
        if value.is_empty() {
            if field.required {
                problems.add(id, format!("{} is required", field.label));
            }
            if field.kind == FieldType::Checkbox {
                properties.insert(field.id.to_owned(), "false".to_owned());
            }
            continue;
        }
        match field.kind {
            FieldType::Text => {}
            FieldType::Checkbox => {
                if !matches!(value.as_str(), "true" | "false") {
                    problems.add(id, "must be true or false");
                    continue;
                }
            }
            FieldType::Number => match value.parse::<i64>() {
                Ok(number) if field.min.is_some_and(|min| number < min) => {
                    problems.add(
                        id,
                        format!("must be at least {}", field.min.unwrap_or_default()),
                    );
                    continue;
                }
                Ok(number) if field.max.is_some_and(|max| number > max) => {
                    problems.add(
                        id,
                        format!("must be at most {}", field.max.unwrap_or_default()),
                    );
                    continue;
                }
                Ok(_) => {}
                Err(_) => {
                    problems.add(id, "must be a whole number");
                    continue;
                }
            },
            FieldType::Repository => {
                let fits = |format: &str, kind: &str| {
                    (field.formats.is_empty() || field.formats.contains(&format))
                        && (field.types.is_empty() || field.types.contains(&kind))
                };
                let known = value == ALL && field.all
                    || choices
                        .repositories
                        .iter()
                        .any(|(name, format, kind)| *name == value && fits(format, kind));
                if !known {
                    problems.add(id, format!("{value} is not a repository this task can use"));
                    continue;
                }
            }
            FieldType::BlobStore => {
                let known = value == ALL && field.all
                    || choices
                        .blob_stores
                        .iter()
                        .any(|(name, group)| *name == value && (field.groups || !group));
                if !known {
                    problems.add(id, format!("{value} is not a blob store this task can use"));
                    continue;
                }
            }
        }
        properties.insert(field.id.to_owned(), value);
    }
    properties
}

pub fn email(text: &str) -> bool {
    let Some((local, domain)) = text.split_once('@') else {
        return false;
    };
    !local.is_empty()
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && !text.chars().any(|c| c.is_whitespace() || c == ',')
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn choices() -> Choices {
        Choices {
            repositories: vec![
                (
                    "docker-hosted".to_owned(),
                    "docker".to_owned(),
                    "hosted".to_owned(),
                ),
                ("npm-proxy".to_owned(), "npm".to_owned(), "proxy".to_owned()),
            ],
            blob_stores: vec![("default".to_owned(), false), ("pool".to_owned(), true)],
        }
    }

    fn check(kind: &dyn TaskKind, given: Value) -> Result<Properties, Vec<String>> {
        let mut problems = Problems::default();
        let properties = properties(kind, given.as_object(), None, &choices(), &mut problems);
        match problems.is_empty() {
            true => Ok(properties),
            false => Err(problems
                .into_inner()
                .into_iter()
                .map(|problem| problem.id)
                .collect()),
        }
    }

    #[test]
    fn properties_are_checked_against_their_fields() {
        let purge = crate::kinds::purge_unused::PurgeUnused;
        let given = check(
            &purge,
            json!({"repositoryName": "npm-proxy", "lastUsed": 7, "extra": "x"}),
        )
        .unwrap();
        assert_eq!(given.get("lastUsed").map(String::as_str), Some("7"));
        assert!(!given.contains_key("extra"));
        assert_eq!(
            check(&purge, json!({"repositoryName": "*"})).unwrap()["lastUsed"],
            "30"
        );
        assert_eq!(
            check(
                &purge,
                json!({"repositoryName": "docker-hosted", "lastUsed": 0})
            ),
            Err(vec![
                "properties.repositoryName".to_owned(),
                "properties.lastUsed".to_owned()
            ])
        );
        assert_eq!(
            check(&purge, json!({})),
            Err(vec!["properties.repositoryName".to_owned()])
        );
        let reconcile = crate::kinds::reconcile::Reconcile;
        assert_eq!(
            check(&reconcile, json!({"blobstoreName": "pool"})),
            Err(vec!["properties.blobstoreName".to_owned()])
        );
        let compact = crate::kinds::compact::Compact;
        assert!(
            check(
                &compact,
                json!({"blobstoreName": "pool", "blobsOlderThan": ""})
            )
            .is_ok()
        );
    }

    #[test]
    fn addresses_look_like_addresses() {
        assert!(email("ops@example.org"));
        assert!(!email("ops"));
        assert!(!email("ops@example"));
        assert!(!email("a@b.c, d@e.f"));
    }
}
