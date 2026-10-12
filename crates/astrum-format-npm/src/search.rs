use std::collections::HashSet;

use astrum_content::{ContentError, ContentRequest, Ctx, View, group, proxy};
use astrum_db::entity::asset;
use astrum_repo::RepositoryType;
use axum::http::StatusCode;
use axum::response::Response;
use chrono::Utc;
use sea_orm::{DbBackend, EntityTrait, Statement};
use serde_json::{Value, json};

use crate::path::Name;
use crate::{PACKUMENT, reply};

const DEFAULT_SIZE: usize = 20;
const MAX_SIZE: usize = 250;
const MAX_MATCHES: usize = 1000;
const MAX_REMOTE: usize = 16 * 1024 * 1024;

struct Query {
    text: String,
    size: usize,
    from: usize,
}

fn query(raw: Option<&str>) -> Query {
    let mut parsed = Query {
        text: String::new(),
        size: DEFAULT_SIZE,
        from: 0,
    };
    for (key, value) in url::form_urlencoded::parse(raw.unwrap_or_default().as_bytes()) {
        match key.as_ref() {
            "text" => parsed.text = value.trim().to_owned(),
            "size" => parsed.size = value.parse().unwrap_or(DEFAULT_SIZE).clamp(1, MAX_SIZE),
            "from" => parsed.from = value.parse().unwrap_or(0),
            _ => {}
        }
    }
    parsed
}

pub async fn search(ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
    let wanted = query(request.query.as_deref());
    let anonymous = ctx.subject.as_ref().is_none_or(|subject| subject.anonymous);
    if anonymous && !ctx.allowed(View::Read, "/") {
        ctx.require(View::Read, "/")?;
    }
    let found = match ctx.kind() {
        RepositoryType::Group => {
            let members = group::members(ctx).await?;
            let searches = members.into_iter().map(|member| {
                let member_ctx = ctx.member(member);
                let query = request.query.clone();
                let text = wanted.text.clone();
                async move {
                    let found = leaf(&member_ctx, &text, query.as_deref()).await;
                    (member_ctx, found)
                }
            });
            let mut merged: Vec<Value> = Vec::new();
            let mut seen: HashSet<String> = HashSet::new();
            for (member, found) in futures_util::future::join_all(searches).await {
                match found {
                    Ok(objects) => {
                        for object in objects {
                            let name = package_name(&object).unwrap_or_default();
                            if seen.insert(name) {
                                merged.push(object);
                            }
                        }
                    }
                    Err(error) => {
                        tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member could not search; using the others");
                    }
                }
            }
            merged
        }
        _ => leaf(ctx, &wanted.text, request.query.as_deref()).await?,
    };
    let visible: Vec<Value> = found
        .into_iter()
        .filter(|object| {
            package_name(object)
                .and_then(|name| Name::parse(&name))
                .is_some_and(|name| ctx.allowed(View::Read, &name.packument_path()))
        })
        .collect();
    let total = visible.len();
    let objects: Vec<Value> = visible
        .into_iter()
        .skip(wanted.from)
        .take(wanted.size)
        .collect();
    Ok(reply::json(
        StatusCode::OK,
        &json!({
            "objects": objects,
            "total": total,
            "time": Utc::now().format("%a %b %d %Y %H:%M:%S GMT+0000 (UTC)").to_string(),
        }),
    ))
}

fn package_name(object: &Value) -> Option<String> {
    object
        .pointer("/package/name")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

async fn leaf(ctx: &Ctx, text: &str, raw_query: Option<&str>) -> Result<Vec<Value>, ContentError> {
    match ctx.kind() {
        RepositoryType::Hosted => local(ctx, text).await,
        RepositoryType::Proxy => remote(ctx, raw_query).await,
        RepositoryType::Group => Ok(Vec::new()),
    }
}

fn like(term: &str) -> String {
    format!(
        "%{}%",
        term.replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    )
}

async fn local(ctx: &Ctx, text: &str) -> Result<Vec<Value>, ContentError> {
    let mut sql = "SELECT * FROM asset WHERE repository = $1 AND kind = $2".to_owned();
    let mut values: Vec<sea_orm::Value> = vec![ctx.name().into(), PACKUMENT.into()];
    for term in text.split_whitespace().take(8) {
        values.push(like(term).into());
        let at = values.len();
        sql.push_str(&format!(
            " AND (path ILIKE ${at} OR attributes #>> '{{npm,description}}' ILIKE ${at} OR attributes #>> '{{npm,keywords}}' ILIKE ${at})"
        ));
    }
    sql.push_str(&format!(" ORDER BY path LIMIT {MAX_MATCHES}"));
    let rows = asset::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            values,
        ))
        .all(ctx.db())
        .await?;
    let lowered = text.to_lowercase();
    let mut objects: Vec<(f64, Value)> = rows
        .into_iter()
        .map(|row| {
            let npm = row.attributes.get("npm").cloned().unwrap_or_default();
            let name = npm
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| row.path.trim_start_matches('/').to_owned());
            let score = match name.to_lowercase() {
                exact if exact == lowered => 1.0,
                partial if !lowered.is_empty() && partial.contains(&lowered) => 0.8,
                _ => 0.5,
            };
            let scope = Name::parse(&name)
                .and_then(|name| name.scope)
                .unwrap_or_else(|| "unscoped".to_owned());
            let object = json!({
                "package": {
                    "name": name,
                    "scope": scope,
                    "version": npm.get("version").cloned().unwrap_or(Value::Null),
                    "description": npm.get("description").cloned().unwrap_or(Value::Null),
                    "keywords": npm.get("keywords").cloned().unwrap_or_else(|| json!([])),
                    "date": npm.get("modified").cloned().unwrap_or(Value::Null),
                    "links": {"npm": format!("{}/{name}", ctx.repository_url())},
                    "publisher": {"username": row.uploader},
                    "maintainers": [{"username": row.uploader}],
                },
                "score": {
                    "final": score,
                    "detail": {"quality": score, "popularity": score, "maintenance": score},
                },
                "searchScore": score,
            });
            (score, object)
        })
        .collect();
    objects.sort_by(|a, b| b.0.total_cmp(&a.0));
    Ok(objects.into_iter().map(|(_, object)| object).collect())
}

async fn remote(ctx: &Ctx, raw_query: Option<&str>) -> Result<Vec<Value>, ContentError> {
    let url = proxy::remote_url(&ctx.repository, "/-/v1/search", raw_query)?;
    let answer = proxy::request(ctx, url, &[]).await?;
    if answer.status != StatusCode::OK {
        return Err(ContentError::BadGateway(format!(
            "the remote answered {} to a search",
            answer.status
        )));
    }
    let body = answer.bytes(MAX_REMOTE).await?;
    let parsed: Value = serde_json::from_slice(&body).map_err(|error| {
        ContentError::BadGateway(format!("the remote search answer is not JSON: {error}"))
    })?;
    Ok(parsed
        .get("objects")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_have_bounds() {
        let parsed = query(Some("text=left%20pad&size=9999&from=5&quality=0.65"));
        assert_eq!(parsed.text, "left pad");
        assert_eq!(parsed.size, MAX_SIZE);
        assert_eq!(parsed.from, 5);
        let empty = query(None);
        assert_eq!((empty.text.as_str(), empty.size, empty.from), ("", 20, 0));
        assert_eq!(like("a_b%"), "%a\\_b\\%%");
    }
}
