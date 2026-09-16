use astrum_content::{ContentError, Ctx};
use astrum_db::entity::ansible_import;
use chrono::{Duration, Utc};
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::api::{Urls, time};
use crate::names::Coll;

pub const KEPT_DAYS: i64 = 7;

pub struct Outcome {
    pub coll: Option<(Coll, String)>,
    pub messages: Vec<(&'static str, String)>,
    pub error: Option<(&'static str, String)>,
}

/// Records a finished import so `ansible-galaxy collection publish` can read its result; imports
/// run while the upload is answered, so a task is never waiting or running.
pub async fn record(ctx: &Ctx, outcome: Outcome) -> Result<Uuid, ContentError> {
    let now = Utc::now();
    let cutoff = (now - Duration::days(KEPT_DAYS)).fixed_offset();
    ansible_import::Entity::delete_many()
        .filter(ansible_import::Column::CreatedAt.lt(cutoff))
        .exec(ctx.db())
        .await?;
    let id = Uuid::new_v4();
    let at = now.timestamp_micros() as f64 / 1_000_000.0;
    let messages: Vec<Value> = outcome
        .messages
        .iter()
        .map(|(level, message)| json!({"level": level, "message": message, "time": at}))
        .collect();
    let (namespace, name, version) = match &outcome.coll {
        Some((coll, version)) => (
            Some(coll.namespace.clone()),
            Some(coll.name.clone()),
            Some(version.clone()),
        ),
        None => (None, None, None),
    };
    ansible_import::Entity::insert(ansible_import::ActiveModel {
        id: ActiveValue::Set(id),
        repository: ActiveValue::Set(ctx.name().to_owned()),
        state: ActiveValue::Set(
            match outcome.error {
                Some(_) => "failed",
                None => "completed",
            }
            .to_owned(),
        ),
        namespace: ActiveValue::Set(namespace),
        name: ActiveValue::Set(name),
        version: ActiveValue::Set(version),
        messages: ActiveValue::Set(Value::Array(messages)),
        error: ActiveValue::Set(
            outcome
                .error
                .map(|(code, description)| json!({"code": code, "description": description, "traceback": null})),
        ),
        created_at: ActiveValue::Set(now.fixed_offset()),
        finished_at: ActiveValue::Set(Some(now.fixed_offset())),
    })
    .exec(ctx.db())
    .await?;
    Ok(id)
}

pub async fn document(ctx: &Ctx, id: &str) -> Result<Option<Value>, ContentError> {
    let Ok(id) = Uuid::parse_str(id) else {
        return Ok(None);
    };
    let Some(row) = ansible_import::Entity::find_by_id(id)
        .filter(ansible_import::Column::Repository.eq(ctx.name()))
        .one(ctx.db())
        .await?
    else {
        return Ok(None);
    };
    let created = time(row.created_at.with_timezone(&Utc));
    let finished = row.finished_at.map(|at| time(at.with_timezone(&Utc)));
    Ok(Some(json!({
        "id": row.id.to_string(),
        "href": Urls::of(ctx).import(&row.id.to_string()),
        "state": row.state,
        "created_at": created,
        "updated_at": finished.clone().unwrap_or_else(|| created.clone()),
        "started_at": created,
        "finished_at": finished,
        "error": row.error,
        "messages": row.messages,
        "namespace": row.namespace,
        "name": row.name,
        "version": row.version,
    })))
}
