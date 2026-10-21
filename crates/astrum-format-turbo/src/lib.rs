use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, ContentRequest, Ctx, Format, Serve, View, serve};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Map, Value, json};

pub const ID: &str = "turbo";
const OCTET_STREAM: &str = "application/octet-stream";
const ALLOW: &str = "GET,HEAD,PUT,POST,DELETE,OPTIONS";
const DEFAULT_TEAM: &str = "_";
const MAX_HASH: usize = 128;
const MAX_TEAM: usize = 100;
const MAX_QUERY: usize = 1000;
const JSON_LIMIT: usize = 1 << 20;
const DURATION: &str = "x-artifact-duration";
const TAG: &str = "x-artifact-tag";
const SHA: &str = "x-artifact-sha";
const DIRTY_HASH: &str = "x-artifact-dirty-hash";
const PREFLIGHT_HEADERS: &str = "Authorization, Content-Type, User-Agent, x-artifact-duration, x-artifact-tag, \
x-artifact-sha, x-artifact-dirty-hash, x-artifact-client-ci, x-artifact-client-interactive";

pub struct Turbo;

#[derive(Debug, PartialEq, Eq)]
enum Route<'a> {
    Status,
    Events,
    Query,
    Artifact(&'a str),
}

fn route(path: &str) -> Option<Route<'_>> {
    let rest = path.strip_prefix("/v8").unwrap_or(path);
    let rest = rest.trim_end_matches('/');
    match rest {
        "/artifacts" => Some(Route::Query),
        "/artifacts/status" => Some(Route::Status),
        "/artifacts/events" => Some(Route::Events),
        _ => rest
            .strip_prefix("/artifacts/")
            .filter(|hash| valid_hash(hash))
            .map(Route::Artifact),
    }
}

fn valid_hash(hash: &str) -> bool {
    !hash.is_empty() && hash.len() <= MAX_HASH && hash.chars().all(|c| c.is_ascii_hexdigit())
}

fn valid_team(team: &str) -> bool {
    !team.is_empty()
        && team.len() <= MAX_TEAM
        && team != "."
        && team != ".."
        && team
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

fn team(query: Option<&str>) -> Result<String, ContentError> {
    let pairs: Vec<(String, String)> =
        url::form_urlencoded::parse(query.unwrap_or_default().as_bytes())
            .into_owned()
            .collect();
    let value = |name: &str| {
        pairs
            .iter()
            .find(|(key, value)| key == name && !value.is_empty())
            .map(|(_, value)| value.clone())
    };
    match value("slug").or_else(|| value("teamId")) {
        None => Ok(DEFAULT_TEAM.to_owned()),
        Some(team) if valid_team(&team) => Ok(team),
        Some(team) => Err(ContentError::BadRequest(format!(
            "the team {team} may only hold letters, digits, '.', '-' and '_', at most {MAX_TEAM}"
        ))),
    }
}

fn stored(path: &str) -> bool {
    let mut parts = path.trim_start_matches('/').split('/');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(team), Some(hash), None) => valid_team(team) && valid_hash(hash),
        _ => false,
    }
}

fn stored_path(team: &str, hash: &str) -> String {
    format!("/{team}/{}", hash.to_ascii_lowercase())
}

fn component(team: &str, hash: &str) -> ComponentKey {
    ComponentKey {
        namespace: team.to_owned(),
        name: hash.to_ascii_lowercase(),
        ..Default::default()
    }
}

fn header_text(headers: &HeaderMap, name: &str, limit: usize) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= limit)
        .map(str::to_owned)
}

fn metadata(headers: &HeaderMap) -> Result<Value, ContentError> {
    let mut fields = Map::new();
    if let Some(duration) = header_text(headers, DURATION, 20) {
        let duration: u64 = duration.parse().map_err(|_| {
            ContentError::BadRequest(format!("{DURATION} must be a whole number of milliseconds"))
        })?;
        fields.insert("duration".into(), json!(duration));
    }
    for (name, key, limit) in [
        (TAG, "tag", 1024),
        (SHA, "sha", 128),
        (DIRTY_HASH, "dirtyHash", 256),
    ] {
        if let Some(value) = header_text(headers, name, limit) {
            fields.insert(key.into(), json!(value));
        }
    }
    Ok(json!({ ID: fields }))
}

fn attribute<'a>(asset: &'a Asset, key: &str) -> Option<&'a Value> {
    asset.attributes.get(ID).and_then(|fields| fields.get(key))
}

fn duration_of(asset: &Asset) -> u64 {
    attribute(asset, "duration")
        .and_then(Value::as_u64)
        .unwrap_or(0)
}

fn text_of<'a>(asset: &'a Asset, key: &str) -> Option<&'a str> {
    attribute(asset, key).and_then(Value::as_str)
}

fn with_metadata(mut response: Response, asset: &Asset) -> Response {
    let headers = response.headers_mut();
    if let Ok(value) = HeaderValue::from_str(&duration_of(asset).to_string()) {
        headers.insert(DURATION, value);
    }
    for (name, key) in [(TAG, "tag"), (SHA, "sha"), (DIRTY_HASH, "dirtyHash")] {
        if let Some(value) = text_of(asset, key).and_then(|value| HeaderValue::from_str(value).ok())
        {
            headers.insert(name, value);
        }
    }
    response
}

fn json_response(status: StatusCode, value: &Value) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(value.to_string()))
        .expect("a JSON answer is valid")
}

fn preflight(request: &ContentRequest) -> Response {
    let wanted: String = request
        .header("access-control-request-headers")
        .unwrap_or(PREFLIGHT_HEADERS)
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ',' | ' '))
        .collect();
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET, HEAD, PUT, POST, OPTIONS",
        )
        .header(header::ACCESS_CONTROL_ALLOW_HEADERS, wanted)
        .header(header::ACCESS_CONTROL_MAX_AGE, "600")
        .body(Body::empty())
        .expect("a preflight answer is valid")
}

fn error_code(status: StatusCode) -> &'static str {
    match status {
        StatusCode::BAD_REQUEST => "bad_request",
        StatusCode::UNAUTHORIZED => "unauthorized",
        StatusCode::FORBIDDEN => "forbidden",
        StatusCode::NOT_FOUND => "not_found",
        StatusCode::METHOD_NOT_ALLOWED => "method_not_allowed",
        StatusCode::CONFLICT => "conflict",
        StatusCode::SERVICE_UNAVAILABLE => "unavailable",
        _ => "internal_error",
    }
}

fn error_json(error: ContentError) -> Response {
    let message = match &error {
        ContentError::Internal(_) => "the server failed; see its log".to_owned(),
        other => other.to_string(),
    };
    let code = error_code(error.status());
    let mut response = error.into_response();
    *response.body_mut() =
        Body::from(json!({ "error": { "code": code, "message": message } }).to_string());
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    response
}

fn not_allowed(method: &Method) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: ALLOW,
    }
}

impl Turbo {
    async fn status(&self, ctx: &Ctx) -> Result<Response, ContentError> {
        ctx.require(View::Read, "/")?;
        Ok(json_response(
            StatusCode::OK,
            &json!({ "status": "enabled" }),
        ))
    }

    async fn events(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        ctx.require(View::Read, "/")?;
        let body = request.bytes(JSON_LIMIT).await?;
        let events: Vec<Value> = serde_json::from_slice(&body)
            .map_err(|_| ContentError::BadRequest("the events must be a JSON array".to_owned()))?;
        tracing::debug!(
            repository = ctx.name(),
            events = events.len(),
            "turbo cache events"
        );
        Ok(StatusCode::OK.into_response())
    }

    async fn query(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let team = team(request.query.as_deref())?;
        let body = request.bytes(JSON_LIMIT).await?;
        let wanted: Value = serde_json::from_slice(&body)
            .map_err(|_| ContentError::BadRequest("the query must be a JSON object".to_owned()))?;
        let hashes: Vec<&str> = wanted
            .get("hashes")
            .and_then(Value::as_array)
            .ok_or_else(|| ContentError::BadRequest("the query needs a list of hashes".to_owned()))?
            .iter()
            .map(|hash| hash.as_str().unwrap_or_default())
            .collect();
        if hashes.len() > MAX_QUERY {
            return Err(ContentError::BadRequest(format!(
                "a query asks for at most {MAX_QUERY} hashes"
            )));
        }
        let mut answer = Map::new();
        for hash in hashes {
            answer.insert(hash.to_owned(), self.describe(ctx, &team, hash).await?);
        }
        Ok(json_response(StatusCode::OK, &Value::Object(answer)))
    }

    async fn describe(&self, ctx: &Ctx, team: &str, hash: &str) -> Result<Value, ContentError> {
        if !valid_hash(hash) {
            return Ok(json!({ "error": { "message": "the hash must be hexadecimal" } }));
        }
        let path = stored_path(team, hash);
        if ctx.require(View::Read, &path).is_err() {
            return Ok(json!({ "error": { "message": "you may not read this artifact" } }));
        }
        let Some(asset) = store::find(ctx.db(), ctx.name(), &path).await? else {
            return Ok(Value::Null);
        };
        let mut info = Map::new();
        info.insert("size".into(), json!(asset.size.max(0)));
        info.insert("taskDurationMs".into(), json!(duration_of(&asset)));
        for key in ["tag", "sha", "dirtyHash"] {
            if let Some(value) = text_of(&asset, key) {
                info.insert(key.into(), json!(value));
            }
        }
        Ok(Value::Object(info))
    }

    async fn artifact(
        &self,
        ctx: Ctx,
        request: ContentRequest,
        hash: &str,
    ) -> Result<Response, ContentError> {
        let team = team(request.query.as_deref())?;
        let path = stored_path(&team, hash);
        match request.method {
            Method::GET | Method::HEAD => {
                ctx.require(View::Read, &path)?;
                let asset = store::find(ctx.db(), ctx.name(), &path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(format!("no artifact {hash}")))?;
                let response = serve(
                    &ctx,
                    &asset,
                    &request.method,
                    &request.headers,
                    Serve::download(),
                )
                .await?;
                Ok(with_metadata(response, &asset))
            }
            Method::PUT => {
                let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
                ctx.require(if exists { View::Edit } else { View::Add }, &path)?;
                let attributes = metadata(&request.headers)?;
                let mut put = PutAsset::hosted(&ctx, path.clone())
                    .component(component(&team, hash))
                    .content_type(OCTET_STREAM)
                    .attributes(attributes);
                put.check_content_type = false;
                store::put(&ctx, put, request.into_stream()).await?;
                let url = format!(
                    "{}/v8/artifacts/{}",
                    ctx.repository_url(),
                    hash.to_ascii_lowercase()
                );
                Ok(json_response(
                    StatusCode::ACCEPTED,
                    &json!({ "urls": [url] }),
                ))
            }
            Method::DELETE => {
                ctx.require(View::Delete, &path)?;
                match store::delete(&ctx, &path).await? {
                    true => Ok(StatusCode::NO_CONTENT.into_response()),
                    false => Err(ContentError::not_found(format!("no artifact {hash}"))),
                }
            }
            Method::OPTIONS => Ok(preflight(&request)),
            ref method => Err(not_allowed(method)),
        }
    }
}

#[async_trait]
impl Format for Turbo {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "Turborepo caches are hosted repositories".to_owned(),
            ));
        }
        let path = request.path.clone();
        let Some(route) = route(&path) else {
            return match request.method {
                Method::OPTIONS => Ok(preflight(&request)),
                Method::GET | Method::HEAD if stored(&path) => {
                    ctx.require(View::Read, &path)?;
                    let asset = store::find(ctx.db(), ctx.name(), &path)
                        .await?
                        .ok_or_else(|| ContentError::not_found(path.clone()))?;
                    let response = serve(
                        &ctx,
                        &asset,
                        &request.method,
                        &request.headers,
                        Serve::download(),
                    )
                    .await?;
                    Ok(with_metadata(response, &asset))
                }
                _ => Err(ContentError::not_found(
                    "a Turborepo cache answers under /v8/artifacts",
                )),
            };
        };
        let answer = match (route, &request.method) {
            (_, &Method::OPTIONS) => Ok(preflight(&request)),
            (Route::Status, &Method::GET) => self.status(&ctx).await,
            (Route::Events, &Method::POST) => self.events(&ctx, &request).await,
            (Route::Query, &Method::POST) => self.query(&ctx, &request).await,
            (Route::Artifact(hash), _) => {
                let hash = hash.to_owned();
                self.artifact(ctx, request, &hash).await
            }
            (_, method) => Err(not_allowed(method)),
        };
        Ok(answer.unwrap_or_else(error_json))
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        store::find(ctx.db(), ctx.name(), path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_answer_with_and_without_the_version() {
        assert_eq!(route("/v8/artifacts/status"), Some(Route::Status));
        assert_eq!(route("/artifacts/status"), Some(Route::Status));
        assert_eq!(route("/v8/artifacts/events"), Some(Route::Events));
        assert_eq!(route("/v8/artifacts"), Some(Route::Query));
        assert_eq!(route("/v8/artifacts/"), Some(Route::Query));
        assert_eq!(
            route("/v8/artifacts/ab12CD"),
            Some(Route::Artifact("ab12CD"))
        );
        assert_eq!(route("/v8/artifacts/not-a-hash"), None);
        assert_eq!(route("/v8/artifacts/ab/cd"), None);
        assert_eq!(route("/elsewhere"), None);
    }

    #[test]
    fn a_hash_is_hexadecimal_and_bounded() {
        assert!(valid_hash("0123456789abcdefABCDEF"));
        assert!(!valid_hash(""));
        assert!(!valid_hash("xyz"));
        assert!(!valid_hash(&"a".repeat(MAX_HASH + 1)));
    }

    #[test]
    fn the_team_comes_from_the_slug_then_the_id() {
        assert_eq!(team(None).unwrap(), DEFAULT_TEAM);
        assert_eq!(team(Some("slug=web&teamId=team_1")).unwrap(), "web");
        assert_eq!(team(Some("teamId=team_1")).unwrap(), "team_1");
        assert_eq!(team(Some("slug=")).unwrap(), DEFAULT_TEAM);
        assert!(team(Some("slug=../etc")).is_err());
        assert!(team(Some("slug=a%2Fb")).is_err());
        assert!(team(Some("slug=..")).is_err());
    }

    #[test]
    fn artifacts_live_under_their_team_in_lower_case() {
        assert_eq!(stored_path("web", "ABCdef"), "/web/abcdef");
        assert!(stored("/web/abcdef"));
        assert!(!stored("/web/abcdef/more"));
        assert!(!stored("/v8/artifacts"));
        let key = component("web", "ABC");
        assert_eq!((key.namespace.as_str(), key.name.as_str()), ("web", "abc"));
    }

    #[test]
    fn errors_answer_in_the_shape_turbo_reads() {
        let response = error_json(ContentError::Forbidden);
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/json");
        let unauthorized = error_json(ContentError::Unauthorized);
        assert!(
            unauthorized
                .headers()
                .contains_key(header::WWW_AUTHENTICATE)
        );
        assert_eq!(error_code(StatusCode::NOT_FOUND), "not_found");
    }

    #[test]
    fn upload_headers_become_attributes() {
        let mut headers = HeaderMap::new();
        headers.insert(DURATION, HeaderValue::from_static("1234"));
        headers.insert(TAG, HeaderValue::from_static("signed"));
        headers.insert(SHA, HeaderValue::from_static("deadbeef"));
        assert_eq!(
            metadata(&headers).unwrap(),
            json!({ ID: { "duration": 1234, "tag": "signed", "sha": "deadbeef" } })
        );
        headers.insert(DURATION, HeaderValue::from_static("soon"));
        assert!(metadata(&headers).is_err());
        assert_eq!(metadata(&HeaderMap::new()).unwrap(), json!({ ID: {} }));
    }
}
