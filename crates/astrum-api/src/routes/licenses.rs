use std::collections::HashMap;

use astrum_policy::license;
use astrum_policy::subject::COMPONENT_PREFIX;
use axum::body::Body;
use axum::extract::{Query, State};
use axum::http::{HeaderValue, header};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use bytes::Bytes;
use futures_util::stream;
use sea_orm::{ConnectionTrait, DatabaseConnection, DbBackend, DbErr, QueryResult, Statement};
use serde_json::{Value, json};

use crate::routes::findings::{
    BATCH_MIN, PATHS, Params, Scope, repository_param, statement, visible,
};
use crate::{ApiError, AppState, Caller, REST};

pub const PAGE: usize = 100;
pub const EXPORT_ROW_CAP: usize = 500_000;
const HEADER: &str = "repository,format,namespace,name,version,declared,expression,state,verdict\n";
const SOURCE: &str = "FROM component c LEFT JOIN component_license cl ON cl.component_id = c.id";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(&format!("{REST}/v1/security/licenses"), get(licenses))
        .route(
            &format!("{REST}/v1/security/licenses/export.csv"),
            get(export),
        )
}

#[derive(Default)]
struct Filters {
    format: Option<String>,
    state: Option<license::State>,
    spellings: Option<Vec<String>>,
}

struct Cursor {
    repository: String,
    component: i64,
}

impl Cursor {
    fn token(&self, state: &AppState) -> String {
        let sealed = state
            .secrets()
            .seal(&format!("after:{}:{}", self.repository, self.component));
        URL_SAFE_NO_PAD.encode(sealed)
    }

    fn parse(state: &AppState, token: &str) -> Result<Self, ApiError> {
        let invalid = || ApiError::BadRequest("the continuation token is not valid".to_owned());
        let sealed = URL_SAFE_NO_PAD
            .decode(token)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .ok_or_else(invalid)?;
        let text = state.secrets().open(&sealed).map_err(|_| invalid())?;
        let rest = text.strip_prefix("after:").ok_or_else(invalid)?;
        let (repository, component) = rest.rsplit_once(':').ok_or_else(invalid)?;
        if repository.is_empty() {
            return Err(invalid());
        }
        let component = component.parse().map_err(|_| invalid())?;
        Ok(Self {
            repository: repository.to_owned(),
            component,
        })
    }
}

struct Row {
    component: i64,
    repository: String,
    format: String,
    namespace: String,
    name: String,
    version: String,
    declared: String,
    expression: Option<String>,
    ids: Vec<String>,
    state: String,
    verdict: String,
    paths: Option<Vec<String>>,
}

impl Row {
    fn read(row: &QueryResult) -> Result<Self, DbErr> {
        Ok(Self {
            component: row.try_get("", "component_id")?,
            repository: row.try_get("", "repository")?,
            format: row.try_get("", "format")?,
            namespace: row.try_get("", "namespace")?,
            name: row.try_get("", "name")?,
            version: row.try_get("", "version")?,
            declared: row.try_get("", "declared")?,
            expression: row.try_get("", "expression")?,
            ids: row.try_get("", "ids")?,
            state: row.try_get("", "state")?,
            verdict: row.try_get("", "verdict")?,
            paths: row.try_get("", "paths")?,
        })
    }

    fn cursor(&self) -> Cursor {
        Cursor {
            repository: self.repository.clone(),
            component: self.component,
        }
    }

    fn json(&self) -> Value {
        json!({
            "repository": self.repository,
            "format": self.format,
            "namespace": self.namespace,
            "name": self.name,
            "version": self.version,
            "declared": self.declared,
            "expression": self.expression,
            "ids": self.ids,
            "state": self.state,
            "verdict": self.verdict,
        })
    }

    fn csv(&self) -> String {
        let fields = [
            self.repository.as_str(),
            self.format.as_str(),
            self.namespace.as_str(),
            self.name.as_str(),
            self.version.as_str(),
            self.declared.as_str(),
            self.expression.as_deref().unwrap_or_default(),
            self.state.as_str(),
            self.verdict.as_str(),
        ];
        let mut line = fields
            .iter()
            .map(|field| csv_field(field))
            .collect::<Vec<_>>()
            .join(",");
        line.push('\n');
        line
    }
}

fn csv_field(value: &str) -> String {
    let guarded = match value.chars().next() {
        Some('=' | '+' | '-' | '@' | '\t' | '\r') => format!("'{value}"),
        _ => value.to_owned(),
    };
    match guarded.contains([',', '"', '\n', '\r']) {
        true => format!("\"{}\"", guarded.replace('"', "\"\"")),
        false => guarded,
    }
}

fn text_param<'a>(
    params: &'a HashMap<String, String>,
    key: &str,
) -> Result<Option<&'a str>, ApiError> {
    match params.get(key) {
        None => Ok(None),
        Some(value) if value.trim().is_empty() => Err(ApiError::invalid(key, "must not be empty")),
        Some(value) => Ok(Some(value.as_str())),
    }
}

async fn spellings(db: &DatabaseConnection, wanted: &str) -> Result<Vec<String>, ApiError> {
    let rows = db
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            "SELECT DISTINCT unnest(ids) AS id FROM component_license",
        ))
        .await?;
    let mut found = Vec::new();
    for row in rows {
        let id: String = row.try_get("", "id")?;
        if license::same_id(&id, wanted) {
            found.push(id);
        }
    }
    Ok(found)
}

async fn filters(app: &AppState, params: &HashMap<String, String>) -> Result<Filters, ApiError> {
    let format = text_param(params, "format")?.map(str::to_owned);
    let state =
        match text_param(params, "state")? {
            None => None,
            Some(value) => Some(license::State::parse(value).ok_or_else(|| {
                ApiError::invalid("state", "must be spdx, mapped, unknown or none")
            })?),
        };
    let spellings = match text_param(params, "license")? {
        None => None,
        Some(wanted) => Some(spellings(app.db().conn(), wanted).await?),
    };
    Ok(Filters {
        format,
        state,
        spellings,
    })
}

fn after(app: &AppState, params: &HashMap<String, String>) -> Result<Option<Cursor>, ApiError> {
    params
        .get("continuationToken")
        .filter(|token| !token.is_empty())
        .map(|token| Cursor::parse(app, token))
        .transpose()
}

fn conditions(
    params: &mut Params,
    full: &str,
    selected: &str,
    filters: &Filters,
    after: Option<&Cursor>,
) -> Vec<String> {
    let mut conditions = vec![visible(full, selected)];
    match filters.state {
        None => conditions.push("cl.component_id IS NOT NULL".to_owned()),
        Some(license::State::None) => {
            conditions.push("(cl.component_id IS NULL OR cl.state = 'none')".to_owned());
        }
        Some(state) => {
            let state = params.bind(state.as_str().to_owned());
            conditions.push(format!("cl.state = {state}"));
        }
    }
    if let Some(spellings) = &filters.spellings {
        let spellings = params.bind(spellings.clone());
        conditions.push(format!("cl.ids && {spellings}"));
    }
    if let Some(format) = &filters.format {
        let format = params.bind(format.clone());
        conditions.push(format!("c.format = {format}"));
    }
    if let Some(after) = after {
        let repository = params.bind(after.repository.clone());
        let component = params.bind(after.component);
        conditions.push(format!(
            "(c.repository, c.id) > ({repository}, {component})"
        ));
    }
    conditions
}

async fn fetch(
    db: &DatabaseConnection,
    scope: &Scope,
    filters: &Filters,
    after: Option<&Cursor>,
    limit: usize,
) -> Result<Vec<Row>, ApiError> {
    let mut params = Params::default();
    let full = params.bind(scope.full.clone());
    let selected = params.bind(scope.selected.clone());
    let conditions = conditions(&mut params, &full, &selected, filters, after);
    let size = params.bind(limit as i64);
    let sql = format!(
        "SELECT c.id AS component_id, c.repository, c.format, c.namespace, c.name, c.version, \
                COALESCE(cl.declared, '') AS declared, cl.expression, \
                COALESCE(cl.ids, '{{}}') AS ids, COALESCE(cl.state, 'none') AS state, \
                CASE WHEN pv.blocked THEN 'blocked' WHEN pv.state = 'report' THEN 'reported' \
                     WHEN pv.state = 'pass' THEN 'pass' ELSE 'unjudged' END AS verdict, \
                CASE WHEN c.repository = ANY({selected}) THEN {PATHS} END AS paths \
         {SOURCE} \
         JOIN repository r ON r.name = c.repository \
         LEFT JOIN content_policy cp ON cp.name = r.attributes #>> '{{policy,name}}' \
         LEFT JOIN policy_verdict pv ON pv.repository = c.repository \
           AND pv.subject = '{COMPONENT_PREFIX}' || c.id::text \
           AND pv.policy = cp.name AND pv.policy_revision = cp.revision \
         WHERE {where_clause} \
         ORDER BY c.repository, c.id LIMIT {size}",
        where_clause = conditions.join(" AND "),
    );
    let rows = db.query_all(statement(sql, params)).await?;
    Ok(rows.iter().map(Row::read).collect::<Result<Vec<_>, _>>()?)
}

async fn count(
    db: &DatabaseConnection,
    scope: &Scope,
    filters: &Filters,
    cap: usize,
) -> Result<usize, ApiError> {
    if scope.is_empty() {
        return Ok(0);
    }
    let mut params = Params::default();
    let full = params.bind(scope.full.clone());
    let selected = params.bind(scope.selected.clone());
    let conditions = conditions(&mut params, &full, &selected, filters, None);
    let limit = params.bind((cap + 1) as i64);
    let sql = format!(
        "SELECT count(*) AS n FROM (SELECT 1 {SOURCE} WHERE {where_clause} LIMIT {limit}) AS matched",
        where_clause = conditions.join(" AND "),
    );
    let row = db
        .query_one(statement(sql, params))
        .await?
        .ok_or_else(|| ApiError::internal(anyhow::anyhow!("a count returned no row")))?;
    let total: i64 = row.try_get("", "n")?;
    Ok(total as usize)
}

async fn page(
    app: &AppState,
    scope: &Scope,
    filters: &Filters,
    after: Option<Cursor>,
    limit: usize,
) -> Result<(Vec<Row>, Option<Cursor>), ApiError> {
    let mut items: Vec<Row> = Vec::new();
    if scope.is_empty() {
        return Ok((items, None));
    }
    let db = app.db().conn();
    let cap = app.examine_cap();
    let batch = (limit + 1).max(BATCH_MIN);
    let mut cursor = after;
    let mut examined = 0usize;
    loop {
        let rows = fetch(db, scope, filters, cursor.as_ref(), batch).await?;
        let fetched = rows.len();
        let mut last = None;
        for (index, row) in rows.into_iter().enumerate() {
            let key = row.cursor();
            if scope.admits(&row.repository, row.paths.as_deref()) {
                if items.len() == limit {
                    let next = items.last().map(Row::cursor);
                    return Ok((items, next));
                }
                items.push(row);
            }
            last = Some(key);
            examined += 1;
            let exhausted = fetched < batch && index + 1 == fetched;
            if examined >= cap && !exhausted {
                return Ok((items, last));
            }
        }
        if fetched < batch {
            return Ok((items, None));
        }
        cursor = last;
    }
}

async fn licenses(
    State(app): State<AppState>,
    caller: Caller,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, ApiError> {
    let repository = repository_param(&params)?;
    let scope = Scope::open(&app, &caller, repository).await?;
    let filters = filters(&app, &params).await?;
    let after = after(&app, &params)?;
    let (rows, next) = page(&app, &scope, &filters, after, PAGE).await?;
    let items: Vec<Value> = rows.iter().map(Row::json).collect();
    Ok(Json(json!({
        "items": items,
        "continuationToken": next.map(|cursor| cursor.token(&app)),
    })))
}

struct Walk {
    db: DatabaseConnection,
    scope: Scope,
    filters: Filters,
    after: Option<Cursor>,
    header: bool,
    finished: bool,
}

async fn export(
    State(app): State<AppState>,
    caller: Caller,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Response, ApiError> {
    let repository = repository_param(&params)?;
    let scope = Scope::open(&app, &caller, repository).await?;
    let filters = filters(&app, &params).await?;
    let db = app.db().conn().clone();
    let cap = app.licence_export_cap();
    if count(&db, &scope, &filters, cap).await? > cap {
        return Err(ApiError::BadRequest(format!(
            "more than {cap} components match; narrow the filter with repository, format, license or state and export again"
        )));
    }
    let walk = Walk {
        db,
        scope,
        filters,
        after: None,
        header: true,
        finished: false,
    };
    let body = stream::unfold(walk, |mut walk| async move {
        if walk.finished {
            return None;
        }
        let mut chunk = String::new();
        if walk.header {
            walk.header = false;
            chunk.push_str(HEADER);
        }
        if walk.scope.is_empty() {
            walk.finished = true;
            return Some((Ok::<_, std::io::Error>(Bytes::from(chunk)), walk));
        }
        match fetch(
            &walk.db,
            &walk.scope,
            &walk.filters,
            walk.after.as_ref(),
            BATCH_MIN,
        )
        .await
        {
            Ok(rows) => {
                let fetched = rows.len();
                for row in rows {
                    if walk.scope.admits(&row.repository, row.paths.as_deref()) {
                        chunk.push_str(&row.csv());
                    }
                    walk.after = Some(row.cursor());
                }
                walk.finished = fetched < BATCH_MIN;
                Some((Ok(Bytes::from(chunk)), walk))
            }
            Err(error) => {
                walk.finished = true;
                Some((Err(std::io::Error::other(error.to_string())), walk))
            }
        }
    });
    let mut response = Body::from_stream(body).into_response();
    let headers = response.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"licenses.csv\""),
    );
    Ok(response)
}
