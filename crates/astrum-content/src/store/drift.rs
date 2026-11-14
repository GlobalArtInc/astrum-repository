use std::collections::{HashMap, HashSet};

use astrum_db::entity::component;
use chrono::{DateTime, FixedOffset};
use sea_orm::sea_query::Expr;
use sea_orm::{ColumnTrait, ConnectionTrait, EntityTrait, QueryFilter};
use serde_json::{Map, Value};

use super::{
    Asset, Candidate, ComponentIdentity, Decision, Replaceable, Resolved, changed_meanwhile,
    check_write, identity_of,
};
use crate::ContentError;
use crate::transfer::{AssetStatus, Conflict, Outcome, Status};

#[derive(Clone, Debug, PartialEq)]
pub struct Intent {
    pub identity: ComponentIdentity,
    pub attributes: Map<String, Value>,
    pub companion_only: bool,
}

pub(super) struct Drift {
    pub(super) id: i64,
    pub(super) identity: ComponentIdentity,
    pub(super) attributes: Value,
}

pub(super) fn merged(
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

pub(super) fn wanted<'a>(
    intents: &'a [Intent],
    identity: &ComponentIdentity,
) -> Option<&'a Map<String, Value>> {
    intents
        .iter()
        .find(|intent| intent.identity == *identity)
        .map(|intent| &intent.attributes)
        .filter(|attributes| !attributes.is_empty())
}

pub(super) fn keeps_stored(intents: &[Intent], identity: &ComponentIdentity) -> bool {
    intents
        .iter()
        .any(|intent| intent.companion_only && intent.identity == *identity)
}

pub(super) async fn renew(
    txn: &impl ConnectionTrait,
    id: i64,
    now: DateTime<FixedOffset>,
) -> Result<(), ContentError> {
    component::Entity::update_many()
        .col_expr(component::Column::UpdatedAt, Expr::value(now))
        .filter(component::Column::Id.eq(id))
        .exec(txn)
        .await?;
    Ok(())
}

pub(super) fn written_identities(
    candidates: &[Candidate<'_>],
    decision: &Decision,
) -> HashSet<ComponentIdentity> {
    candidates
        .iter()
        .zip(&decision.assets)
        .filter(|(_, outcome)| matches!(outcome.status, AssetStatus::New | AssetStatus::Replaced))
        .filter_map(|(candidate, _)| candidate.put.component.as_ref())
        .map(identity_of)
        .collect()
}

pub(super) async fn drifting(
    txn: &impl ConnectionTrait,
    repository: &str,
    resolved: &[Resolved<'_>],
    written: &HashSet<ComponentIdentity>,
    intents: &[Intent],
) -> Result<Vec<Drift>, ContentError> {
    if intents.is_empty() {
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
        let identity = identity_of(found.key);
        let Some(wanted) = wanted(intents, &identity) else {
            continue;
        };
        let row = rows
            .iter()
            .find(|row| row.id == id)
            .ok_or_else(|| changed_meanwhile(repository))?;
        if let Some(attributes) = merged(&row.attributes, wanted)? {
            drift.push(Drift {
                id,
                identity,
                attributes,
            });
        }
    }
    Ok(drift)
}

pub(super) struct Stance<'a> {
    pub(super) conflict: Conflict,
    pub(super) planning: bool,
    pub(super) replaceable: Option<Replaceable<'a>>,
}

pub(super) fn judge(
    repository: &str,
    candidates: &[Candidate<'_>],
    existing: &HashMap<String, Asset>,
    mut decision: Decision,
    drifted: &HashSet<ComponentIdentity>,
    stance: Stance<'_>,
) -> Result<(Decision, Vec<usize>), ContentError> {
    let Stance {
        conflict,
        planning,
        replaceable,
    } = stance;
    if drifted.is_empty() || !matches!(decision.status, Status::Copied | Status::Unchanged) {
        return Ok((decision, Vec::new()));
    }
    let indexes: Vec<usize> = candidates
        .iter()
        .enumerate()
        .filter(|(_, candidate)| {
            candidate
                .put
                .component
                .as_ref()
                .is_some_and(|key| drifted.contains(&identity_of(key)))
        })
        .map(|(index, _)| index)
        .collect();
    let refuse = |decision: &mut Decision, status: Status| {
        decision.status = status;
        decision.bytes_copied = 0;
        decision.bytes_reused = 0;
        decision.writes = false;
    };
    match conflict {
        Conflict::Fail if planning => {
            for index in &indexes {
                decision.assets[*index].status = AssetStatus::Conflict;
            }
            refuse(&mut decision, Status::Conflict);
            Ok((decision, Vec::new()))
        }
        Conflict::Fail => Err(ContentError::Conflict(format!(
            "{repository}: {} exist with different attributes",
            indexes
                .iter()
                .map(|index| candidates[*index].put.path.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        ))),
        Conflict::Skip => {
            refuse(&mut decision, Status::Skipped);
            Ok((decision, Vec::new()))
        }
        Conflict::Replace => {
            for index in &indexes {
                let put = candidates[*index].put;
                check_write(
                    repository,
                    put,
                    existing.get(&put.path),
                    Conflict::Replace,
                    replaceable.map(|allowed| allowed as &dyn Fn(&str) -> bool),
                )?;
            }
            Ok((decision, indexes))
        }
    }
}

pub(super) fn mark_replaced(outcome: &mut Outcome, indexes: &[usize]) {
    if indexes.is_empty() {
        return;
    }
    for index in indexes {
        outcome.assets[*index].status = AssetStatus::Replaced;
    }
    outcome.status = Status::Copied;
}
