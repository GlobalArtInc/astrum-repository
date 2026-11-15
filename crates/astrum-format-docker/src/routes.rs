use std::collections::HashMap;
use std::sync::Arc;

use astrum_api::domains::DomainRoute;
use astrum_api::{AppState, Caller};
use astrum_content::ContentRequest;
use axum::Router;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Path, Query, Request, State};
use axum::http::{HeaderMap, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, get};
use serde::Deserialize;
use serde_json::json;

use astrum_api::ApiError;
use astrum_content::{ContentError, View};

use crate::error::RegistryError;
use crate::reference::{Digest, Reference};
use crate::sbom::{self, Outcome};
use crate::{
    Docker, ID, Mount, admit, auth, insert, json_response, link, paginate, storage, supply,
};

pub const FIRST_SUGGESTED_PORT: u16 = 8082;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Connector(pub u16);

#[derive(Deserialize)]
struct SupplyChainQuery {
    image: Option<String>,
    reference: Option<String>,
}

#[derive(Deserialize)]
struct SbomQuery {
    image: Option<String>,
    digest: Option<String>,
    page: Option<String>,
    filter: Option<String>,
}

pub fn routes() -> Router<AppState> {
    routes_with_limit(sbom::shared().clone())
}

pub fn routes_with_sbom_limit(permits: usize) -> Router<AppState> {
    routes_with_limit(sbom::Limit::new(permits))
}

fn routes_with_limit(limit: sbom::Limit) -> Router<AppState> {
    let docker = Arc::new(Docker::default());
    let catalog_docker = docker.clone();
    Router::new()
        .route(
            "/service/rest/internal/ui/docker/suggest-port",
            get(suggest_port),
        )
        .route(
            "/service/rest/v1/docker/{repository}/supply-chain",
            get(supply_chain),
        )
        .route(
            "/service/rest/v1/docker/{repository}/sbom",
            get(
                move |state: State<AppState>,
                      caller: Caller,
                      repository: Path<String>,
                      query: Result<Query<SbomQuery>, QueryRejection>,
                      headers: HeaderMap,
                      uri: Uri| {
                    let limit = limit.clone();
                    async move {
                        sbom_packages(limit, state, caller, repository, query, headers, uri).await
                    }
                },
            ),
        )
        .route("/v2", get(auth::ping))
        .route("/v2/", get(auth::ping))
        .route("/v2/token", get(auth::token).post(auth::oauth))
        .route(
            "/v2/_catalog",
            get(
                move |state: State<AppState>, caller: Caller, request: Request| {
                    let docker = catalog_docker.clone();
                    async move { catalog(docker, state, caller, request).await }
                },
            ),
        )
        .route(
            "/v2/{*rest}",
            any(
                move |state: State<AppState>,
                      caller: Caller,
                      rest: Path<String>,
                      request: Request| {
                    let docker = docker.clone();
                    async move { routed(docker, state, caller, rest, request).await }
                },
            ),
        )
}

fn v2_rule_path(request: &ContentRequest) -> String {
    format!("/v2{}", request.path)
}

fn path_enabled(repository: &astrum_repo::config::Repository) -> bool {
    repository.format.id == ID
        && repository
            .attributes
            .docker
            .as_ref()
            .and_then(|docker| docker.path_enabled)
            .unwrap_or(false)
}

fn unreachable_name(name: &str) -> RegistryError {
    RegistryError::new(
        StatusCode::NOT_FOUND,
        "NAME_UNKNOWN",
        format!("repository {name} is not a docker repository reachable by path"),
    )
}

fn subdomain_of(headers: &HeaderMap, uri: &Uri) -> Option<String> {
    let base = astrum_api::routes::repositories::base_url(headers, uri);
    let host = auth::service(&base);
    let host = match host.strip_prefix('[') {
        Some(_) => return None,
        None => host.rsplit_once(':').map_or(host, |(host, _)| host),
    };
    if host.parse::<std::net::IpAddr>().is_ok() {
        return None;
    }
    let (label, rest) = host.split_once('.')?;
    (!label.is_empty() && !rest.is_empty()).then(|| label.to_ascii_lowercase())
}

pub async fn dedicated(
    state: &AppState,
    connector: Option<Connector>,
    route: Option<&DomainRoute>,
    headers: &HeaderMap,
    uri: &Uri,
) -> Result<Option<astrum_repo::config::Repository>, RegistryError> {
    let db = state.db().conn();
    let internal = |error| RegistryError::from(astrum_content::ContentError::internal(error));
    if let Some(DomainRoute::Root { repository, .. }) = route {
        return match astrum_repo::store::find(db, repository)
            .await
            .map_err(internal)?
        {
            Some(found) if found.format.id == ID => Ok(Some(found)),
            _ => Err(unreachable_name(repository)),
        };
    }
    if route.is_some() {
        return Ok(None);
    }
    if let Some(Connector(port)) = connector {
        return match astrum_repo::store::docker_on_port(db, port)
            .await
            .map_err(internal)?
        {
            Some(repository) => Ok(Some(repository)),
            None => Err(RegistryError::new(
                StatusCode::NOT_FOUND,
                "NAME_UNKNOWN",
                format!("no Docker repository answers on the port {port}"),
            )),
        };
    }
    let Some(subdomain) = subdomain_of(headers, uri) else {
        return Ok(None);
    };
    astrum_repo::store::docker_on_subdomain(db, &subdomain)
        .await
        .map_err(internal)
}

async fn suggest_port(State(state): State<AppState>, caller: Caller, uri: Uri) -> Response {
    if caller.subject().is_none_or(|subject| subject.anonymous) {
        return astrum_api::ApiError::Unauthorized.into_response();
    }
    let repositories = match astrum_repo::store::list(state.db().conn()).await {
        Ok(repositories) => repositories,
        Err(error) => return astrum_api::ApiError::internal(error).into_response(),
    };
    let mut used: std::collections::BTreeSet<i32> = repositories
        .iter()
        .filter_map(|repository| repository.attributes.docker.as_ref())
        .flat_map(|docker| docker.ports().map(|(_, port)| port).collect::<Vec<_>>())
        .collect();
    used.extend(state.application_port().map(i32::from));
    used.extend(
        url::form_urlencoded::parse(uri.query().unwrap_or_default().as_bytes())
            .filter(|(key, _)| key == "exclude")
            .filter_map(|(_, value)| value.parse::<i32>().ok()),
    );
    let port =
        (i32::from(FIRST_SUGGESTED_PORT)..=i32::from(u16::MAX)).find(|port| !used.contains(port));
    match port {
        Some(port) => axum::Json(port).into_response(),
        None => astrum_api::ApiError::NotFound("every port is taken".to_owned()).into_response(),
    }
}

async fn supply_chain(
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    Query(query): Query<SupplyChainQuery>,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, ApiError> {
    let image = query
        .image
        .filter(|image| !image.is_empty())
        .ok_or_else(|| ApiError::BadRequest("image is required".to_owned()))?;
    let raw = query
        .reference
        .filter(|reference| !reference.is_empty())
        .ok_or_else(|| ApiError::BadRequest("reference is required".to_owned()))?;
    let (ctx, _) =
        astrum_api::routes::content::context(&state, caller, &repository, &headers, &uri)
            .await
            .map_err(content_failure)?;
    ctx.require(View::Read, &storage::manifest_path(&image, &raw))
        .map_err(content_failure)?;
    if ctx.format_id() != ID {
        return Err(ApiError::BadRequest(format!(
            "{repository} is not a Docker repository"
        )));
    }
    let reference = Reference::parse(&raw)
        .ok_or_else(|| ApiError::not_found(format!("{raw} is not a reference")))?;
    let listing = supply::list(&ctx, &image, &reference, state.policy_cache())
        .await
        .map_err(content_failure)?
        .ok_or_else(|| ApiError::not_found(format!("{raw} is not stored for {image}")))?;
    Ok(axum::Json(listing).into_response())
}

fn refusal(status: StatusCode, message: String, category: &str) -> Response {
    (
        status,
        axum::Json(json!({"id": "*", "message": message, "category": category})),
    )
        .into_response()
}

fn busy() -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        [(header::RETRY_AFTER, sbom::RETRY_AFTER_SECONDS)],
        axum::Json(json!({
            "id": "*",
            "message": "too many SBOM documents are being read; retry shortly",
            "category": "busy"
        })),
    )
        .into_response()
}

async fn sbom_packages(
    limit: sbom::Limit,
    State(state): State<AppState>,
    caller: Caller,
    Path(repository): Path<String>,
    query: Result<Query<SbomQuery>, QueryRejection>,
    headers: HeaderMap,
    uri: Uri,
) -> Result<Response, ApiError> {
    let Query(query) = query.map_err(|_| {
        ApiError::BadRequest(
            "the query string is not valid; each parameter may appear once".to_owned(),
        )
    })?;
    let image = query
        .image
        .filter(|image| !image.is_empty())
        .ok_or_else(|| ApiError::BadRequest("image is required".to_owned()))?;
    let raw = query
        .digest
        .filter(|digest| !digest.is_empty())
        .ok_or_else(|| ApiError::BadRequest("digest is required".to_owned()))?;
    let page = sbom::page_number(query.page.as_deref()).map_err(ApiError::BadRequest)?;
    let filter = query.filter.unwrap_or_default();
    if filter.chars().count() > sbom::FILTER_CHARS {
        return Err(ApiError::BadRequest(format!(
            "filter must be at most {} characters",
            sbom::FILTER_CHARS
        )));
    }
    let (ctx, _) =
        astrum_api::routes::content::context(&state, caller, &repository, &headers, &uri)
            .await
            .map_err(content_failure)?;
    ctx.require(View::Read, &storage::manifest_path(&image, &raw))
        .map_err(content_failure)?;
    if ctx.format_id() != ID {
        return Err(ApiError::BadRequest(format!(
            "{repository} is not a Docker repository"
        )));
    }
    let not_listed =
        || ApiError::not_found(format!("{raw} is not an SBOM or attestation of {image}"));
    let digest = Digest::parse(&raw).ok_or_else(not_listed)?;
    let artifact = supply::artifact(&ctx, &image, &digest)
        .await
        .map_err(content_failure)?
        .ok_or_else(not_listed)?;
    ctx.require(View::Read, &artifact.asset.path)
        .map_err(content_failure)?;
    let Some(_permit) = limit.take() else {
        return Ok(busy());
    };
    let outcome = sbom::load(&ctx, &image, &artifact)
        .await
        .map_err(content_failure)?;
    Ok(match outcome {
        Outcome::Parsed(parsed) => {
            let body = tokio::task::spawn_blocking(move || sbom::page_of(parsed, page, &filter))
                .await
                .map_err(ApiError::internal)?;
            axum::Json(body).into_response()
        }
        Outcome::TooLarge => refusal(
            StatusCode::PAYLOAD_TOO_LARGE,
            format!(
                "the SBOM documents of the artifact are larger than {} bytes",
                astrum_sbom::MAX_INPUT_BYTES
            ),
            "too_large",
        ),
        Outcome::Missing(message) => ApiError::NotFound(message).into_response(),
        Outcome::Refused(error) => refusal(
            StatusCode::UNPROCESSABLE_ENTITY,
            error.to_string(),
            error.category(),
        ),
        Outcome::NoDocument => refusal(
            StatusCode::UNPROCESSABLE_ENTITY,
            "the artifact has no SBOM document layer".to_owned(),
            "unsupported",
        ),
    })
}

fn content_failure(error: ContentError) -> ApiError {
    match error {
        ContentError::NotFound(message) => ApiError::NotFound(message),
        ContentError::Unauthorized => ApiError::Unauthorized,
        ContentError::Forbidden => ApiError::forbidden(),
        ContentError::Blocked(message) => ApiError::Forbidden(message),
        ContentError::BadRequest(message) => ApiError::BadRequest(message),
        other => ApiError::internal(other),
    }
}

async fn routed(
    docker: Arc<Docker>,
    State(state): State<AppState>,
    caller: Caller,
    Path(rest): Path<String>,
    request: Request,
) -> Response {
    let (parts, body) = request.into_parts();
    let base = astrum_api::routes::repositories::base_url(&parts.headers, &parts.uri);
    let model = state.security().model();
    let connector = parts.extensions.get::<Connector>().copied();
    let route = parts.extensions.get::<DomainRoute>().cloned();
    let dedicated = match dedicated(
        &state,
        connector,
        route.as_ref(),
        &parts.headers,
        &parts.uri,
    )
    .await
    {
        Ok(dedicated) => dedicated,
        Err(error) => return auth::with_challenge(error.into_response(), &model, &base, None),
    };
    let (name, remainder, mount) = match &dedicated {
        Some(repository) => (repository.name.as_str(), rest.as_str(), Mount::Root),
        None => {
            let (segment, remainder) = rest.split_once('/').unwrap_or((rest.as_str(), ""));
            let name = match &route {
                Some(route) => match route.repository_for(segment) {
                    Some(name) => name,
                    None => {
                        return auth::with_challenge(
                            unreachable_name(segment).into_response(),
                            &model,
                            &base,
                            None,
                        );
                    }
                },
                None => segment,
            };
            let found = match state.content().repository(name).await {
                Ok(found) => found,
                Err(error) => {
                    return auth::with_challenge(
                        RegistryError::from(error).into_response(),
                        &model,
                        &base,
                        None,
                    );
                }
            };
            let reachable = match &route {
                Some(_) => found
                    .as_deref()
                    .is_some_and(|repository| repository.format.id == ID),
                None => found.as_deref().is_some_and(path_enabled),
            };
            if !reachable {
                return auth::with_challenge(
                    unreachable_name(name).into_response(),
                    &model,
                    &base,
                    None,
                );
            }
            (name, remainder, Mount::Path(segment.to_owned()))
        }
    };
    let context =
        astrum_api::routes::content::context(&state, caller, name, &parts.headers, &parts.uri)
            .await;
    let ctx = match context {
        Ok((ctx, _)) => ctx,
        Err(error) => {
            return auth::with_challenge(
                RegistryError::from(error).into_response(),
                &model,
                &base,
                None,
            );
        }
    };
    let request = match ContentRequest::new(
        parts.method,
        &format!("/{remainder}"),
        parts.uri.query().map(str::to_owned),
        parts.headers,
        body,
    ) {
        Ok(request) => request,
        Err(error) => {
            return auth::with_challenge(
                RegistryError::from(error).into_response(),
                &model,
                &base,
                None,
            );
        }
    };
    let rule_path = v2_rule_path(&request);
    match state
        .content()
        .routing_allows(&ctx.repository, &rule_path)
        .await
    {
        Ok(true) => {}
        Ok(false) => return astrum_api::routes::content::blocked(),
        Err(error) => return RegistryError::from(error).into_response(),
    }
    let ctx = ctx.with_request_path(&rule_path);
    docker.serve(ctx, request, mount).await
}

async fn catalog(
    docker: Arc<Docker>,
    State(state): State<AppState>,
    caller: Caller,
    request: Request,
) -> Response {
    let (parts, _) = request.into_parts();
    let (headers, uri) = (&parts.headers, &parts.uri);
    let base = astrum_api::routes::repositories::base_url(headers, uri);
    let model = state.security().model();
    let connector = parts.extensions.get::<Connector>().copied();
    let route = parts.extensions.get::<DomainRoute>().cloned();
    match dedicated(&state, connector, route.as_ref(), headers, uri).await {
        Ok(Some(repository)) => {
            let context = astrum_api::routes::content::context(
                &state,
                caller,
                &repository.name,
                headers,
                uri,
            )
            .await;
            let request = context.and_then(|(ctx, _)| {
                ContentRequest::new(
                    parts.method.clone(),
                    "/_catalog",
                    uri.query().map(str::to_owned),
                    headers.clone(),
                    axum::body::Body::empty(),
                )
                .map(|request| (ctx, request))
            });
            return match request {
                Ok((ctx, request)) => docker.serve(ctx, request, Mount::Root).await,
                Err(error) => auth::with_challenge(
                    RegistryError::from(error).into_response(),
                    &model,
                    &base,
                    Some("registry:catalog:*"),
                ),
            };
        }
        Ok(None) => {}
        Err(error) => {
            return auth::with_challenge(
                error.into_response(),
                &model,
                &base,
                Some("registry:catalog:*"),
            );
        }
    }
    let params: HashMap<String, String> = uri
        .query()
        .map(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .into_owned()
                .collect()
        })
        .unwrap_or_default();
    let result = async {
        if caller.subject().is_none() {
            return Err(RegistryError::unauthorized());
        }
        let repositories = astrum_repo::store::list(state.db().conn())
            .await
            .map_err(|error| RegistryError::from(astrum_content::ContentError::internal(error)))?;
        let mut names = Vec::new();
        for repository in repositories {
            let listed = match &route {
                Some(route) => route.admits(&repository.name) && repository.format.id == ID,
                None => path_enabled(&repository),
            };
            if !repository.online || !listed {
                continue;
            }
            let name = repository.name.clone();
            let ctx = admit(state.content().context(
                Arc::new(repository),
                model.clone(),
                caller.0.clone(),
                base.clone(),
                String::new(),
            ));
            if ctx.subject.is_none() {
                continue;
            }
            let shown = route
                .as_ref()
                .and_then(|route| route.segment_of(&name))
                .unwrap_or(&name)
                .to_owned();
            for image in docker.images_of(&ctx).await? {
                names.push(format!("{shown}/{image}"));
            }
        }
        let (repositories, next) = paginate(
            names,
            params.get("n").map(String::as_str),
            params.get("last").map(String::as_str),
        )?;
        let mut response = json_response(
            StatusCode::OK,
            "application/json",
            &json!({ "repositories": repositories }),
        );
        if let Some(link) = link("/v2/_catalog", next) {
            insert(response.headers_mut(), header::LINK.as_str(), &link);
        }
        Ok(response)
    }
    .await;
    let response = result.unwrap_or_else(IntoResponse::into_response);
    auth::with_challenge(response, &model, &base, Some("registry:catalog:*"))
}

#[cfg(test)]
mod tests {
    use axum::http::Method;

    use super::*;

    fn rule_path_for(remainder: &str) -> String {
        let request = ContentRequest::new(
            Method::GET,
            &format!("/{remainder}"),
            None,
            HeaderMap::new(),
            axum::body::Body::empty(),
        )
        .unwrap();
        v2_rule_path(&request)
    }

    #[test]
    fn well_formed_registry_paths_keep_their_rule_path() {
        for remainder in [
            "",
            "_catalog",
            "library/alpine/manifests/latest",
            "library/alpine/manifests/sha256:0123abcd",
            "team/app/blobs/sha256:0123abcd",
            "team/app/blobs/uploads/",
            "team/app/blobs/uploads/6f1c2b0e-7f6b-4a43-9f43-1c0b0cf4ac2d",
            "team/app/tags/list",
            "team/app/referrers/sha256:0123abcd",
        ] {
            assert_eq!(rule_path_for(remainder), format!("/v2/{remainder}"));
        }
    }

    #[test]
    fn doubled_slashes_leave_the_rule_path() {
        assert_eq!(
            rule_path_for("/library//alpine/manifests/latest"),
            "/v2/library/alpine/manifests/latest"
        );
    }
}
