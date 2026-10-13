use std::convert::Infallible;

use astrum_db::audit::Event;
use astrum_repo::{Problem, Repository, RepositoryType, Request, Spec, WriteError, Writer, store};
use astrum_security::Subject;
use axum::extract::{FromRequestParts, Path, State};
use axum::http::request::Parts;
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::{Value, json};

use crate::error::{ApiError, FieldError};
use crate::{AppState, Caller};

pub const REPOSITORIES: &str = "/service/rest/v1/repositories";
pub const REPOSITORY_SETTINGS: &str = "/service/rest/v1/repositorySettings";
pub const AUDIT_DOMAIN: &str = "repository";

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(REPOSITORIES, get(list))
        .route(
            "/service/rest/v1/repositories/{name}",
            get(summary).delete(remove),
        )
        .route(
            "/service/rest/v1/repositories/{format}/{kind}",
            post(create),
        )
        .route(
            "/service/rest/v1/repositories/{name}/invalidate-cache",
            post(invalidate_cache),
        )
        .route(
            "/service/rest/v1/repositories/{name}/rebuild-index",
            post(rebuild_index),
        )
        .route(
            "/service/rest/v1/repositories/{format}/{kind}/{name}",
            get(read).put(update),
        )
        .route(REPOSITORY_SETTINGS, get(settings))
}

async fn list(
    State(state): State<AppState>,
    caller: Caller,
    base: BaseUrl,
) -> Result<Json<Vec<Value>>, ApiError> {
    let subject = signed(&caller)?;
    let repositories = store::list(state.db().conn()).await?;
    Ok(Json(
        repositories
            .iter()
            .filter(|repository| visible(subject, repository))
            .map(|repository| {
                let mut summary = repository.summary(&base.0);
                summary["size"] = json!(0);
                summary
            })
            .collect(),
    ))
}

async fn settings(
    State(state): State<AppState>,
    caller: Caller,
    base: BaseUrl,
) -> Result<Json<Vec<Value>>, ApiError> {
    let subject = signed(&caller)?;
    let repositories = store::list(state.db().conn()).await?;
    Ok(Json(
        repositories
            .iter()
            .filter(|repository| subject.is_permitted(&admin(repository, "read")))
            .map(|repository| repository.document(&base.0))
            .collect(),
    ))
}

async fn summary(
    State(state): State<AppState>,
    caller: Caller,
    base: BaseUrl,
    Path(name): Path<String>,
) -> Result<Json<Value>, ApiError> {
    let subject = signed(&caller)?;
    let repository = existing(&state, &name).await?;
    if !visible(subject, &repository) {
        return Err(denied(subject));
    }
    Ok(Json(repository.summary(&base.0)))
}

async fn read(
    State(state): State<AppState>,
    caller: Caller,
    base: BaseUrl,
    Path((format, kind, name)): Path<(String, String, String)>,
) -> Result<Json<Value>, ApiError> {
    let subject = signed(&caller)?;
    let spec = spec(&format, &kind)?;
    let repository = store::find(state.db().conn(), &name)
        .await?
        .filter(|repository| repository.spec() == spec)
        .ok_or_else(|| missing(&name))?;
    if !subject.is_permitted(&admin(&repository, "read")) {
        return Err(hidden_or_denied(subject, &repository));
    }
    Ok(Json(repository.document(&base.0)))
}

async fn create(
    State(state): State<AppState>,
    caller: Caller,
    Path((format, kind)): Path<(String, String)>,
    body: Result<crate::Json<Request>, ApiError>,
) -> Result<StatusCode, ApiError> {
    let subject = signed(&caller)?;
    let spec = spec(&format, &kind)?;
    let crate::Json(request) = body?;
    let seal = |plain: &str| state.secrets().seal(plain);
    let repository = Repository::prepare(spec, request, None, &seal).map_err(invalid)?;
    require(
        subject,
        &permission(spec.format.id, &repository.name, "add"),
    )?;
    let created = Writer::begin(state.db().conn())
        .await?
        .reserving(state.application_port())
        .create(repository)
        .await?;
    changed(&state, &caller, "created", &created).await
}

async fn update(
    State(state): State<AppState>,
    caller: Caller,
    Path((format, kind, name)): Path<(String, String, String)>,
    body: Result<crate::Json<Request>, ApiError>,
) -> Result<StatusCode, ApiError> {
    let subject = signed(&caller)?;
    let spec = spec(&format, &kind)?;
    let crate::Json(request) = body?;
    let writer = Writer::begin(state.db().conn())
        .await?
        .reserving(state.application_port());
    let existing = writer.find(&name).cloned().ok_or_else(|| missing(&name))?;
    require(subject, &admin(&existing, "edit"))?;
    store::unchanged(&existing, spec, request.name.as_deref())?;
    let seal = |plain: &str| state.secrets().seal(plain);
    let repository = Repository::prepare(spec, request, Some(&existing), &seal).map_err(invalid)?;
    let updated = writer.update(&existing.name, repository).await?;
    state.content().forget_repository(&updated.name);
    changed(&state, &caller, "updated", &updated).await
}

async fn remove(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    let subject = signed(&caller)?;
    let writer = Writer::begin(state.db().conn()).await?;
    let existing = writer.find(&name).cloned().ok_or_else(|| missing(&name))?;
    if !subject.is_permitted(&admin(&existing, "delete")) {
        return Err(hidden_or_denied(subject, &existing));
    }
    let blobs = astrum_content::store::blobs_of_repository(state.db().conn(), &existing.name)
        .await
        .map_err(ApiError::internal)?;
    let deleted = writer.delete(&existing.name).await?;
    state.content().forget_repository(&deleted.name);
    state.content().purge_blobs(blobs).await;
    changed(&state, &caller, "deleted", &deleted).await
}

async fn invalidate_cache(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    maintain(
        &state,
        &caller,
        &name,
        &[RepositoryType::Proxy, RepositoryType::Group],
        "the cache can be invalidated only for proxy and group repositories",
    )
    .await
}

async fn rebuild_index(
    State(state): State<AppState>,
    caller: Caller,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    maintain(
        &state,
        &caller,
        &name,
        &[RepositoryType::Hosted, RepositoryType::Proxy],
        "the search index can be rebuilt only for hosted and proxy repositories",
    )
    .await
}

async fn maintain(
    state: &AppState,
    caller: &Caller,
    name: &str,
    kinds: &[RepositoryType],
    refusal: &str,
) -> Result<StatusCode, ApiError> {
    let subject = signed(caller)?;
    let repository = existing(state, name).await?;
    require(subject, &admin(&repository, "edit"))?;
    if !kinds.contains(&repository.kind) {
        return Err(ApiError::BadRequest(refusal.to_owned()));
    }
    if repository.kind != RepositoryType::Hosted {
        astrum_content::store::mark_stale(state.db().conn(), &repository.name)
            .await
            .map_err(ApiError::internal)?;
        state.content().forget_repository(&repository.name);
    }
    tracing::info!(repository = %repository.name, "cached content marked for revalidation");
    Ok(StatusCode::NO_CONTENT)
}

async fn changed(
    state: &AppState,
    caller: &Caller,
    kind: &str,
    repository: &Repository,
) -> Result<StatusCode, ApiError> {
    state.security().reload().await?;
    state.repository_changes().notify_one();
    let event = Event::new(AUDIT_DOMAIN, kind, &repository.name).with(json!({
        "format": repository.format.id,
        "type": repository.kind.as_str(),
    }));
    state.audit(caller, event).await;
    Ok(match kind {
        "created" => StatusCode::CREATED,
        _ => StatusCode::NO_CONTENT,
    })
}

async fn existing(state: &AppState, name: &str) -> Result<Repository, ApiError> {
    store::find(state.db().conn(), name)
        .await?
        .ok_or_else(|| missing(name))
}

fn spec(format: &str, kind: &str) -> Result<Spec, ApiError> {
    Spec::from_api(format, kind).ok_or_else(|| {
        ApiError::not_found(format!(
            "there are no {kind} repositories of the format \"{format}\""
        ))
    })
}

fn missing(name: &str) -> ApiError {
    ApiError::not_found(format!("the repository \"{name}\" does not exist"))
}

fn signed(caller: &Caller) -> Result<&Subject, ApiError> {
    caller.subject().ok_or(ApiError::Unauthorized)
}

fn denied(subject: &Subject) -> ApiError {
    match subject.anonymous {
        true => ApiError::Unauthorized,
        false => ApiError::forbidden(),
    }
}

fn hidden_or_denied(subject: &Subject, repository: &Repository) -> ApiError {
    match visible(subject, repository) {
        true => denied(subject),
        false => missing(&repository.name),
    }
}

fn require(subject: &Subject, permission: &str) -> Result<(), ApiError> {
    match subject.is_permitted(permission) {
        true => Ok(()),
        false => Err(denied(subject)),
    }
}

fn permission(format: &str, name: &str, action: &str) -> String {
    format!("astrum:repository-admin:{format}:{name}:{action}")
}

fn admin(repository: &Repository, action: &str) -> String {
    permission(repository.format.id, &repository.name, action)
}

fn visible(subject: &Subject, repository: &Repository) -> bool {
    let view = |action: &str| {
        format!(
            "astrum:repository-view:{}:{}:{action}",
            repository.format.id, repository.name
        )
    };
    subject.is_permitted(&view("browse"))
        || subject.is_permitted(&view("read"))
        || subject.is_permitted(&admin(repository, "read"))
}

fn invalid(problems: Vec<Problem>) -> ApiError {
    ApiError::Invalid(
        problems
            .into_iter()
            .map(|problem| FieldError {
                id: problem.id,
                message: problem.message,
            })
            .collect(),
    )
}

impl From<astrum_repo::Error> for ApiError {
    fn from(error: astrum_repo::Error) -> Self {
        ApiError::internal(error)
    }
}

impl From<WriteError> for ApiError {
    fn from(error: WriteError) -> Self {
        match error {
            WriteError::Invalid(problems) => invalid(problems),
            WriteError::Refused(message) => ApiError::BadRequest(message),
            WriteError::NotFound(name) => missing(&name),
            WriteError::Store(error) => error.into(),
        }
    }
}

pub struct BaseUrl(pub String);

impl<S: Send + Sync> FromRequestParts<S> for BaseUrl {
    type Rejection = Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(BaseUrl(base_url(&parts.headers, &parts.uri)))
    }
}

pub fn base_url(headers: &HeaderMap, uri: &Uri) -> String {
    let first = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    let scheme = first("x-forwarded-proto")
        .map(|scheme| scheme.to_ascii_lowercase())
        .filter(|scheme| scheme == "http" || scheme == "https")
        .or_else(|| uri.scheme_str().map(str::to_owned))
        .unwrap_or_else(|| "http".to_owned());
    let host = first("x-forwarded-host")
        .filter(|host| plain_host(host))
        .or_else(|| first(header::HOST.as_str()).filter(|host| plain_host(host)))
        .or_else(|| uri.authority().map(|authority| authority.to_string()))
        .unwrap_or_else(|| "localhost".to_owned());
    format!("{scheme}://{host}")
}

fn plain_host(host: &str) -> bool {
    host.chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '.' | ':' | '[' | ']' | '_'))
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    fn headers(pairs: &[(&'static str, &'static str)]) -> HeaderMap {
        let mut headers = HeaderMap::new();
        for (name, value) in pairs {
            headers.insert(*name, HeaderValue::from_static(value));
        }
        headers
    }

    #[test]
    fn the_base_url_comes_from_the_request() {
        let uri: Uri = "/service/rest/v1/repositories".parse().unwrap();
        assert_eq!(base_url(&HeaderMap::new(), &uri), "http://localhost");
        assert_eq!(
            base_url(&headers(&[("host", "repo.example.org:8081")]), &uri),
            "http://repo.example.org:8081"
        );
        assert_eq!(
            base_url(
                &headers(&[
                    ("host", "10.0.0.5:8081"),
                    ("x-forwarded-proto", "HTTPS"),
                    ("x-forwarded-host", "astrum.example.org, proxy.internal")
                ]),
                &uri
            ),
            "https://astrum.example.org"
        );
        assert_eq!(
            base_url(
                &headers(&[
                    ("host", "repo.example.org"),
                    ("x-forwarded-proto", "gopher"),
                    ("x-forwarded-host", "evil.example/<script>")
                ]),
                &uri
            ),
            "http://repo.example.org"
        );
        let absolute: Uri = "https://direct.example.org/service/rest".parse().unwrap();
        assert_eq!(
            base_url(&HeaderMap::new(), &absolute),
            "https://direct.example.org"
        );
    }
}
