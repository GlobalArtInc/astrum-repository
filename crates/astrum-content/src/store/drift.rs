use std::collections::HashSet;

use astrum_blob::Written;
use astrum_db::entity::component;
use astrum_repo::config::WritePolicy;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use serde_json::{Map, Value};

use super::{ComponentIdentity, ComponentKey, PutAsset, Resolved, changed_meanwhile, identity_of};
use crate::ContentError;

pub(super) struct Drift {
    pub(super) id: i64,
    pub(super) identity: ComponentIdentity,
    pub(super) attributes: Value,
}

pub(super) fn carried_of(key: &ComponentKey, carried: &[&str]) -> Map<String, Value> {
    carried
        .iter()
        .filter_map(|name| {
            key.attributes
                .get(*name)
                .filter(|value| !value.is_null())
                .map(|value| ((*name).to_owned(), value.clone()))
        })
        .collect()
}

pub(super) fn merged_attributes(
    stored: &Value,
    wanted: &Map<String, Value>,
) -> Result<Option<Value>, ContentError> {
    let mut merged = match stored {
        Value::Null => Map::new(),
        Value::Object(members) => members.clone(),
        _ => {
            return Err(ContentError::internal(anyhow::anyhow!(
                "the stored component attributes are not an object"
            )));
        }
    };
    let mut changed = false;
    for (name, value) in wanted {
        if merged.get(name) != Some(value) {
            merged.insert(name.clone(), value.clone());
            changed = true;
        }
    }
    Ok(changed.then_some(Value::Object(merged)))
}

pub(super) fn check_attribute_write<'a>(
    repository: &str,
    identity: &ComponentIdentity,
    mut puts: impl Iterator<Item = &'a PutAsset>,
) -> Result<(), ContentError> {
    let denied = puts.any(|put| {
        put.write_policy == Some(WritePolicy::Deny)
            && put
                .component
                .as_ref()
                .is_some_and(|found| identity_of(found) == *identity)
    });
    match denied {
        true => Err(ContentError::BadRequest(format!(
            "the repository {repository} is read-only"
        ))),
        false => Ok(()),
    }
}

pub(super) async fn drifting(
    txn: &impl ConnectionTrait,
    repository: &str,
    items: &[(PutAsset, Written)],
    resolved: &[Resolved<'_>],
    written: &HashSet<ComponentIdentity>,
    carried: &[&str],
) -> Result<Vec<Drift>, ContentError> {
    if carried.is_empty() {
        return Ok(Vec::new());
    }
    let untouched: Vec<&Resolved<'_>> = resolved
        .iter()
        .filter(|found| !found.created && !written.contains(&identity_of(found.key)))
        .filter(|found| found.id.is_some())
        .collect();
    let ids: Vec<i64> = untouched.iter().filter_map(|found| found.id).collect();
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = component::Entity::find()
        .filter(component::Column::Id.is_in(ids))
        .all(txn)
        .await?;
    let mut drift = Vec::new();
    for found in untouched {
        let Some(id) = found.id else { continue };
        let wanted = carried_of(found.key, carried);
        if wanted.is_empty() {
            continue;
        }
        let row = rows
            .iter()
            .find(|row| row.id == id)
            .ok_or_else(|| changed_meanwhile(repository))?;
        let Some(attributes) = merged_attributes(&row.attributes, &wanted)? else {
            continue;
        };
        let identity = identity_of(found.key);
        check_attribute_write(repository, &identity, items.iter().map(|(put, _)| put))?;
        drift.push(Drift {
            id,
            identity,
            attributes,
        });
    }
    Ok(drift)
}
