use astrum_content::{ContentError, ContentRequest, Ctx, View};
use astrum_db::entity::gitlfs_lock;
use axum::http::StatusCode;
use axum::response::Response;
use chrono::{SecondsFormat, SubsecRound, Utc};
use sea_orm::sea_query::OnConflict;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, QuerySelect};
use serde_json::{Map, Value, json};

use crate::{JsonBody, Rejected, error, json_response, read_json, require_some};

pub const DEFAULT_LIMIT: u64 = 100;
pub const MAX_LIMIT: u64 = 1000;
const MAX_PATH: usize = 4096;

fn describe(lock: &gitlfs_lock::Model) -> Value {
    json!({
        "id": lock.id.to_string(),
        "path": lock.path,
        "locked_at": lock.locked_at.with_timezone(&Utc).to_rfc3339_opts(SecondsFormat::Secs, true),
        "owner": { "name": lock.owner },
    })
}

fn one(status: StatusCode, lock: &gitlfs_lock::Model) -> Response {
    json_response(status, &json!({ "lock": describe(lock) }))
}

pub fn limit(value: Option<&str>) -> Result<u64, String> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(DEFAULT_LIMIT);
    };
    match value.parse::<i64>() {
        Ok(limit) if limit < 1 => Err("The limit must be positive".to_owned()),
        Ok(limit) => Ok((limit as u64).min(MAX_LIMIT)),
        Err(_) => Err("The limit must be a number".to_owned()),
    }
}

fn cursor(value: Option<&str>) -> Option<i64> {
    value
        .filter(|value| !value.is_empty())
        .map(|value| value.parse::<i64>().unwrap_or(i64::MAX))
}

fn query(request: &ContentRequest) -> Map<String, Value> {
    url::form_urlencoded::parse(request.query.as_deref().unwrap_or("").as_bytes())
        .map(|(key, value)| (key.into_owned(), Value::String(value.into_owned())))
        .collect()
}

async fn page(
    ctx: &Ctx,
    path: Option<&str>,
    after: Option<i64>,
    limit: u64,
) -> Result<(Vec<gitlfs_lock::Model>, Option<String>), ContentError> {
    let mut select =
        gitlfs_lock::Entity::find().filter(gitlfs_lock::Column::Repository.eq(ctx.name()));
    if let Some(path) = path {
        select = select.filter(gitlfs_lock::Column::Path.eq(path));
    }
    if let Some(after) = after {
        select = select.filter(gitlfs_lock::Column::Id.gt(after));
    }
    let mut rows = select
        .order_by_asc(gitlfs_lock::Column::Id)
        .limit(limit + 1)
        .all(ctx.db())
        .await?;
    let next = match rows.len() as u64 > limit {
        true => {
            rows.truncate(limit as usize);
            rows.last().map(|lock| lock.id.to_string())
        }
        false => None,
    };
    Ok((rows, next))
}

async fn find(ctx: &Ctx, id: &str) -> Result<Option<gitlfs_lock::Model>, ContentError> {
    let Ok(id) = id.parse::<i64>() else {
        return Ok(None);
    };
    Ok(gitlfs_lock::Entity::find()
        .filter(gitlfs_lock::Column::Repository.eq(ctx.name()))
        .filter(gitlfs_lock::Column::Id.eq(id))
        .one(ctx.db())
        .await?)
}

async fn optional_body(request: &ContentRequest) -> Result<Map<String, Value>, Rejected> {
    Ok(match read_json(request).await? {
        JsonBody::Object(body) => body,
        JsonBody::Missing => Map::new(),
    })
}

fn text<'a>(body: &'a Map<String, Value>, key: &str) -> Option<&'a str> {
    body.get(key).and_then(Value::as_str)
}

pub async fn list(ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
    require_some(ctx, View::Read, &request.path)?;
    let query = query(request);
    let path = text(&query, "path");
    let (locks, next) = match text(&query, "id") {
        Some(id) => {
            let found = find(ctx, id)
                .await?
                .filter(|lock| path.is_none_or(|path| path == lock.path));
            (found.into_iter().collect(), None)
        }
        None => {
            let limit = match limit(text(&query, "limit")) {
                Ok(limit) => limit,
                Err(message) => return Ok(error(StatusCode::UNPROCESSABLE_ENTITY, message)),
            };
            page(ctx, path, cursor(text(&query, "cursor")), limit).await?
        }
    };
    let mut response = json!({ "locks": locks.iter().map(describe).collect::<Vec<_>>() });
    if let Some(next) = next {
        response["next_cursor"] = json!(next);
    }
    Ok(json_response(StatusCode::OK, &response))
}

pub async fn create(ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
    require_some(ctx, View::Add, &request.path)?;
    let body = match read_json(request).await {
        Ok(JsonBody::Object(body)) => body,
        Ok(JsonBody::Missing) => return Ok(error(StatusCode::BAD_REQUEST, "Missing request body")),
        Err(rejected) => return Ok(rejected.response()),
    };
    let path = text(&body, "path").unwrap_or_default();
    if path.trim().is_empty() || path.len() > MAX_PATH || path.contains('\0') {
        return Ok(error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "A lock needs the path of a file",
        ));
    }
    let reference = body
        .get("ref")
        .and_then(|reference| reference.get("name"))
        .and_then(Value::as_str);
    let inserted = gitlfs_lock::Entity::insert(gitlfs_lock::ActiveModel {
        id: ActiveValue::NotSet,
        repository: ActiveValue::Set(ctx.name().to_owned()),
        path: ActiveValue::Set(path.to_owned()),
        ref_name: ActiveValue::Set(reference.map(str::to_owned)),
        owner: ActiveValue::Set(ctx.uploader()),
        locked_at: ActiveValue::Set(Utc::now().trunc_subsecs(0).fixed_offset()),
    })
    .on_conflict(
        OnConflict::columns([gitlfs_lock::Column::Repository, gitlfs_lock::Column::Path])
            .do_nothing()
            .to_owned(),
    )
    .exec_without_returning(ctx.db())
    .await?;
    let held = gitlfs_lock::Entity::find()
        .filter(gitlfs_lock::Column::Repository.eq(ctx.name()))
        .filter(gitlfs_lock::Column::Path.eq(path))
        .one(ctx.db())
        .await?
        .ok_or_else(|| ContentError::internal(std::io::Error::other("a new lock is missing")))?;
    if inserted == 0 {
        return Ok(json_response(
            StatusCode::CONFLICT,
            &json!({ "lock": describe(&held), "message": "already created lock" }),
        ));
    }
    Ok(one(StatusCode::CREATED, &held))
}

pub async fn verify(ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
    require_some(ctx, View::Add, &request.path)?;
    let body = match optional_body(request).await {
        Ok(body) => body,
        Err(rejected) => return Ok(rejected.response()),
    };
    let wanted = match body.get("limit") {
        Some(Value::Number(number)) => Some(number.to_string()),
        Some(Value::String(text)) => Some(text.clone()),
        _ => None,
    };
    let limit = match limit(wanted.as_deref()) {
        Ok(limit) => limit,
        Err(message) => return Ok(error(StatusCode::UNPROCESSABLE_ENTITY, message)),
    };
    let (locks, next) = page(ctx, None, cursor(text(&body, "cursor")), limit).await?;
    let user = ctx.uploader();
    let (ours, theirs): (Vec<_>, Vec<_>) = locks.iter().partition(|lock| lock.owner == user);
    let mut response = json!({
        "ours": ours.into_iter().map(describe).collect::<Vec<_>>(),
        "theirs": theirs.into_iter().map(describe).collect::<Vec<_>>(),
    });
    if let Some(next) = next {
        response["next_cursor"] = json!(next);
    }
    Ok(json_response(StatusCode::OK, &response))
}

pub async fn unlock(
    ctx: &Ctx,
    request: &ContentRequest,
    id: &str,
) -> Result<Response, ContentError> {
    require_some(ctx, View::Add, &request.path)?;
    let body = match optional_body(request).await {
        Ok(body) => body,
        Err(rejected) => return Ok(rejected.response()),
    };
    let Some(lock) = find(ctx, id).await? else {
        return Ok(error(StatusCode::NOT_FOUND, "Lock does not exist"));
    };
    if lock.owner != ctx.uploader() {
        if !body.get("force").and_then(Value::as_bool).unwrap_or(false) {
            return Ok(error(
                StatusCode::FORBIDDEN,
                format!("Lock on {} is held by {}", lock.path, lock.owner),
            ));
        }
        if !ctx.grant(View::Delete).any() {
            return Ok(error(
                StatusCode::FORBIDDEN,
                "Releasing the lock of another user needs the delete privilege",
            ));
        }
    }
    gitlfs_lock::Entity::delete_by_id(lock.id)
        .exec(ctx.db())
        .await?;
    Ok(one(StatusCode::OK, &lock))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_and_cursors_are_read_like_the_lock_api_wants() {
        assert_eq!(limit(None), Ok(DEFAULT_LIMIT));
        assert_eq!(limit(Some(" 5 ")), Ok(5));
        assert_eq!(limit(Some("5000")), Ok(MAX_LIMIT));
        assert!(limit(Some("0")).is_err());
        assert!(limit(Some("many")).is_err());
        assert_eq!(cursor(Some("12")), Some(12));
        assert_eq!(cursor(Some("x")), Some(i64::MAX));
        assert_eq!(cursor(Some("")), None);
    }
}
