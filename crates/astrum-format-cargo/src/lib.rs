mod hosted;
pub mod index;
mod remote;

use astrum_content::store;
use astrum_content::{
    Asset, BrowseLayout, CHALLENGE, ContentError, ContentRequest, Ctx, Format, Serve, View, group,
    serve,
};
use astrum_repo::RepositoryType;
use astrum_security::api_keys;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::Response;
use serde_json::{Value, json};

pub const ID: &str = "cargo";
pub const CRATE: &str = "crate";
pub const INDEX: &str = "index";
pub const CRATE_TYPE: &str = "application/gzip";
pub const INDEX_TYPE: &str = "text/plain";
pub const REALM: &str = api_keys::CARGO;
const MAX_PUBLISH: usize = 256 * 1024 * 1024;
const MAX_SEARCH: u64 = 100;

pub fn crate_path(name: &str, version: &str) -> String {
    format!("/crates/{name}/{version}/{name}-{version}.crate")
}

#[derive(Default)]
pub struct Cargo {
    cache: remote::Cache,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Config,
    Index(String),
    Publish,
    Yank(String, String),
    Unyank(String, String),
    Download(String, String),
    File(String, String),
    Search,
    Owners(String),
    Me,
    Directory,
    Other,
}

fn route(path: &str) -> Route {
    match path {
        "/config.json" => return Route::Config,
        "/api/v1/crates/new" => return Route::Publish,
        "/api/v1/crates" => return Route::Search,
        "/me" => return Route::Me,
        _ => {}
    }
    if path.ends_with('/') {
        return Route::Directory;
    }
    if let Some(name) = index::name_of_index_path(path) {
        return Route::Index(name);
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let named =
        |name: &str, version: &str| index::valid_name(name) && index::valid_version(version);
    match parts.as_slice() {
        ["api", "v1", "crates", name, version, "download"] if named(name, version) => {
            Route::Download((*name).to_owned(), (*version).to_owned())
        }
        ["api", "v1", "crates", name, version, "yank"] if named(name, version) => {
            Route::Yank((*name).to_owned(), (*version).to_owned())
        }
        ["api", "v1", "crates", name, version, "unyank"] if named(name, version) => {
            Route::Unyank((*name).to_owned(), (*version).to_owned())
        }
        ["api", "v1", "crates", name, "owners"] if index::valid_name(name) => {
            Route::Owners((*name).to_owned())
        }
        ["crates", name, version, file]
            if named(name, version) && *file == format!("{name}-{version}.crate") =>
        {
            Route::File((*name).to_owned(), (*version).to_owned())
        }
        _ => Route::Other,
    }
}

fn json_response(status: StatusCode, value: &Value) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .expect("a JSON response is valid")
}

fn cargo_challenge(ctx: &Ctx) -> String {
    format!("Cargo login_url=\"{}/me\"", ctx.repository_url())
}

fn api_error(ctx: &Ctx, error: ContentError) -> Response {
    let status = error.status();
    let detail = match &error {
        ContentError::Internal(inner) => {
            tracing::error!(error = format!("{inner:#}"), "a cargo request failed");
            "the server failed; see its log".to_owned()
        }
        other => other.to_string(),
    };
    let mut response = json_response(status, &json!({"errors": [{"detail": detail}]}));
    if status == StatusCode::UNAUTHORIZED {
        add_challenges(ctx, &mut response);
    }
    response
}

fn add_challenges(ctx: &Ctx, response: &mut Response) {
    let headers = response.headers_mut();
    if let Ok(value) = HeaderValue::from_str(&cargo_challenge(ctx)) {
        headers.append(header::WWW_AUTHENTICATE, value);
    }
    headers.append(
        header::WWW_AUTHENTICATE,
        HeaderValue::from_static(CHALLENGE),
    );
}

fn requires_authentication(ctx: &Ctx) -> bool {
    ctx.repository
        .attributes
        .cargo
        .as_ref()
        .and_then(|cargo| cargo.require_authentication)
        .unwrap_or(false)
}

fn anonymous(ctx: &Ctx) -> bool {
    ctx.subject.as_ref().is_none_or(|subject| subject.anonymous)
}

fn anonymous_reads(ctx: &Ctx) -> bool {
    !requires_authentication(ctx)
        && ctx.security.anonymous_subject().is_some_and(|subject| {
            subject.is_permitted(&format!("astrum:repository-view:{ID}:{}:read", ctx.name()))
        })
}

fn unauthorized(ctx: &Ctx, message: &str) -> Response {
    let mut response = Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(header::CONTENT_TYPE, "text/plain;charset=utf-8")
        .body(Body::from(message.to_owned()))
        .expect("a plain response is valid");
    add_challenges(ctx, &mut response);
    response
}

fn refused(ctx: &Ctx, path: &str) -> Option<Response> {
    if requires_authentication(ctx) && anonymous(ctx) && !ctx.is_trusted() {
        return Some(unauthorized(
            ctx,
            "this registry needs authentication; run cargo login for it",
        ));
    }
    match ctx.require(View::Read, path) {
        Ok(()) => None,
        Err(ContentError::Unauthorized) => Some(unauthorized(
            ctx,
            "this registry needs authentication; run cargo login for it",
        )),
        Err(error) => Some(api_error(ctx, error)),
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn html(body: String) -> Result<Response, ContentError> {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn directory_page(ctx: &Ctx) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let url = escape(&ctx.repository_url());
    html(format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Cargo registry. Add it to <code>.cargo/config.toml</code> as <code>[registries.{name}] index = \"sparse+{url}/\"</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    ))
}

fn text_body(
    status: StatusCode,
    content_type: &str,
    body: Vec<u8>,
    request: &ContentRequest,
) -> Response {
    let length = body.len();
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, length)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .expect("a text response is valid")
}

fn etagged(request: &ContentRequest, content_type: &str, body: Vec<u8>) -> Response {
    use sha2::{Digest, Sha256};
    let etag = format!("\"{}\"", hex::encode(&Sha256::digest(&body)[..16]));
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let mut response = if unchanged {
        Response::builder()
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .expect("an empty response is valid")
    } else {
        text_body(StatusCode::OK, content_type, body, request)
    };
    if let Ok(value) = HeaderValue::from_str(&etag) {
        response.headers_mut().insert(header::ETAG, value);
    }
    response
}

impl Cargo {
    fn config(&self, ctx: &Ctx, request: &ContentRequest) -> Response {
        let url = ctx.repository_url();
        let mut config = json!({
            "dl": format!("{url}/api/v1/crates"),
            "api": url,
        });
        if !anonymous_reads(ctx) {
            config["auth-required"] = json!(true);
        }
        etagged(request, "application/json", config.to_string().into_bytes())
    }

    async fn entries(&self, ctx: &Ctx, name: &str) -> Result<Option<Vec<Value>>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let entries = hosted::entries(ctx, name).await?;
                Ok((!entries.is_empty()).then_some(entries))
            }
            RepositoryType::Proxy => remote::entries(ctx, name).await,
            RepositoryType::Group => {
                let mut listings = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.entries(&member, name)).await {
                        Ok(Some(entries)) => listings.push(entries),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no index file");
                        }
                    }
                }
                let merged = index::merge(listings);
                Ok((!merged.is_empty()).then_some(merged))
            }
        }
    }

    async fn index_file(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        name: &str,
    ) -> Result<Response, ContentError> {
        if ctx.kind() == RepositoryType::Proxy {
            let found = remote::index_file(ctx, name)
                .await?
                .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
            let options = Serve {
                content_type: Some(INDEX_TYPE.to_owned()),
                ..Serve::default()
            };
            return serve(ctx, &found, &request.method, &request.headers, options).await;
        }
        let entries = self
            .entries(ctx, name)
            .await?
            .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
        Ok(etagged(
            request,
            INDEX_TYPE,
            index::render(&entries).into_bytes(),
        ))
    }

    async fn crate_asset(
        &self,
        ctx: &Ctx,
        name: &str,
        version: &str,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(hosted::crate_asset(ctx, name, version)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Proxy => Ok(remote::crate_file(ctx, &self.cache, name, version)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.crate_asset(&member, name, version)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => {
                            tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
                        }
                    }
                }
                Ok(None)
            }
        }
    }

    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        name: &str,
        version: &str,
    ) -> Result<Response, ContentError> {
        let (owner, found) = self.crate_asset(ctx, name, version).await?.ok_or_else(|| {
            ContentError::not_found(format!(
                "crate `{name}` does not have a version `{version}`"
            ))
        })?;
        serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            Serve::download().disposition("attachment"),
        )
        .await
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        let route = route(path);
        match &route {
            Route::Directory => return directory_page(ctx),
            Route::Me => return self.me(ctx).await,
            _ => {}
        }
        let checked = match &route {
            Route::Download(name, version) | Route::File(name, version) => {
                crate_path(name, version)
            }
            _ => path.to_owned(),
        };
        if let Some(response) = refused(ctx, &checked) {
            return Ok(response);
        }
        match route {
            Route::Config => Ok(self.config(ctx, request)),
            Route::Index(name) => self.index_file(ctx, request, &name).await,
            Route::Download(name, version) | Route::File(name, version) => {
                self.download(ctx, request, &name, &version).await
            }
            Route::Search => Ok(self
                .search(ctx, request)
                .await
                .unwrap_or_else(|error| api_error(ctx, error))),
            Route::Owners(name) => Ok(self
                .owners(ctx, &name)
                .await
                .unwrap_or_else(|error| api_error(ctx, error))),
            _ => Err(ContentError::not_found(path.to_owned())),
        }
    }

    async fn search(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let mut query = String::new();
        let mut limit = 10u64;
        for (key, value) in
            url::form_urlencoded::parse(request.query.as_deref().unwrap_or("").as_bytes())
        {
            match key.as_ref() {
                "q" => query = value.into_owned(),
                "per_page" => limit = value.parse().unwrap_or(10).clamp(1, MAX_SEARCH),
                _ => {}
            }
        }
        let mut crates: Vec<Value> = Vec::new();
        let mut total = 0;
        let leaves = match ctx.kind() {
            RepositoryType::Group => group::members(ctx)
                .await?
                .into_iter()
                .map(|member| ctx.member(member))
                .collect(),
            _ => vec![ctx.clone()],
        };
        for leaf in leaves {
            if leaf.kind() != RepositoryType::Hosted {
                continue;
            }
            let (found, count) = hosted::search(&leaf, &query, MAX_SEARCH).await?;
            total += count;
            for one in found {
                if crates.iter().any(|known| known["name"] == one.name) {
                    continue;
                }
                crates.push(json!({
                    "name": one.name,
                    "max_version": one.version,
                    "description": one.description.unwrap_or_default(),
                }));
            }
        }
        crates.truncate(limit as usize);
        Ok(json_response(
            StatusCode::OK,
            &json!({"crates": crates, "meta": {"total": total}}),
        ))
    }

    async fn owners(&self, ctx: &Ctx, name: &str) -> Result<Response, ContentError> {
        let leaf = match ctx.kind() {
            RepositoryType::Hosted => ctx.clone(),
            _ => {
                return Ok(json_response(StatusCode::OK, &json!({"users": []})));
            }
        };
        let mut users: Vec<String> = Vec::new();
        for found in hosted::versions(&leaf, name).await? {
            for asset in store::assets_of(leaf.db(), found.id).await? {
                if !users.contains(&asset.uploader) {
                    users.push(asset.uploader.clone());
                }
            }
        }
        if users.is_empty() {
            return Err(ContentError::not_found(format!(
                "crate `{name}` is not in this registry"
            )));
        }
        let users: Vec<Value> = users
            .iter()
            .enumerate()
            .map(|(at, user)| json!({"id": at + 1, "login": user, "name": user}))
            .collect();
        Ok(json_response(StatusCode::OK, &json!({ "users": users })))
    }

    async fn me(&self, ctx: &Ctx) -> Result<Response, ContentError> {
        let Some(subject) = ctx.subject.as_ref().filter(|subject| !subject.anonymous) else {
            return Ok(unauthorized(ctx, "sign in to get a token for cargo login"));
        };
        if !ctx.security.realm_active(REALM) {
            return Ok(json_response(
                StatusCode::CONFLICT,
                &json!({"errors": [{"detail": "the Cargo Bearer Token Realm is not active; ask an administrator to activate it, or use 'Basic <base64 of user:password>' as the Cargo token"}]}),
            ));
        }
        let key = api_keys::kept_api_key(
            ctx.db(),
            ctx.content.secrets(),
            REALM,
            &subject.user_id,
            || format!("{REALM}.{}", uuid::Uuid::new_v4()),
        )
        .await
        .map_err(ContentError::internal)?;
        Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from(format!("{}\n", key.key)))
            .map_err(ContentError::internal)
    }

    async fn writable(&self, ctx: &Ctx, path: &str) -> Result<Ctx, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(ctx.clone()),
            RepositoryType::Group => match group::writable_member(ctx).await? {
                Some(member) => {
                    if !ctx.allowed(View::Add, path) && !ctx.allowed(View::Edit, path) {
                        ctx.require(View::Add, path)?;
                    }
                    Ok(ctx.member(member))
                }
                None => Err(ContentError::MethodNotAllowed {
                    method: "PUT".to_owned(),
                    allow: "GET,HEAD",
                }),
            },
            RepositoryType::Proxy => Err(ContentError::MethodNotAllowed {
                method: "PUT".to_owned(),
                allow: "GET,HEAD,DELETE",
            }),
        }
    }

    async fn publish(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if anonymous(ctx) {
            return Err(ContentError::Unauthorized);
        }
        let body = request.bytes(MAX_PUBLISH).await?;
        let parsed = index::parse_publish(body)
            .map_err(|error| ContentError::BadRequest(error.to_string()))?;
        let target = self
            .writable(
                ctx,
                &crate_path(&parsed.metadata.name, &parsed.metadata.vers),
            )
            .await?;
        hosted::publish(&target, parsed).await?;
        Ok(json_response(
            StatusCode::OK,
            &json!({"warnings": {"invalid_categories": [], "invalid_badges": [], "other": []}}),
        ))
    }

    async fn yank(
        &self,
        ctx: &Ctx,
        name: &str,
        version: &str,
        yanked: bool,
    ) -> Result<Response, ContentError> {
        if anonymous(ctx) {
            return Err(ContentError::Unauthorized);
        }
        let target = self.writable(ctx, &crate_path(name, version)).await?;
        hosted::set_yanked(&target, name, version, yanked).await?;
        Ok(json_response(StatusCode::OK, &json!({"ok": true})))
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        if !store::delete(ctx, path).await? {
            return Err(ContentError::not_found(path.to_owned()));
        }
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }
}

#[async_trait]
impl Format for Cargo {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        let answer = match (&request.method, &route) {
            (&Method::GET | &Method::HEAD, _) => return self.read(&ctx, &request).await,
            (&Method::PUT, Route::Publish) => self.publish(&ctx, request).await,
            (&Method::DELETE, Route::Yank(name, version)) => {
                self.yank(&ctx, name, version, true).await
            }
            (&Method::PUT, Route::Unyank(name, version)) => {
                self.yank(&ctx, name, version, false).await
            }
            (&Method::PUT | &Method::DELETE, Route::Owners(_)) => Ok(json_response(
                StatusCode::OK,
                &json!({"ok": true, "msg": "crate owners are managed with repository privileges in Astrum Repository"}),
            )),
            (&Method::DELETE, Route::File(_, _) | Route::Index(_))
                if ctx.kind() != RepositoryType::Group =>
            {
                return self.delete(&ctx, &request.path).await;
            }
            (method, _) => {
                return Err(ContentError::MethodNotAllowed {
                    method: method.to_string(),
                    allow: match ctx.kind() {
                        RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
                        RepositoryType::Proxy => "GET,HEAD,DELETE",
                        RepositoryType::Group => "GET,HEAD,PUT,DELETE",
                    },
                });
            }
        };
        Ok(answer.unwrap_or_else(|error| api_error(&ctx, error)))
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match route(path) {
            Route::File(name, version) | Route::Download(name, version) => Ok(self
                .crate_asset(ctx, &name, &version)
                .await?
                .map(|(_, found)| found)),
            Route::Index(name) if ctx.kind() == RepositoryType::Proxy => {
                remote::index_file(ctx, &name).await
            }
            _ => match ctx.kind() {
                RepositoryType::Group => {
                    Ok(
                        group::first(ctx, |member| async move { self.get(&member, path).await })
                            .await?
                            .map(|(_, found)| found),
                    )
                }
                _ => store::find(ctx.db(), ctx.name(), path).await,
            },
        }
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: None,
            generated: &["/config.json"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_index_the_api_and_crate_files() {
        assert_eq!(route("/config.json"), Route::Config);
        assert_eq!(route("/se/rd/serde"), Route::Index("serde".to_owned()));
        assert_eq!(route("/3/a/abc"), Route::Index("abc".to_owned()));
        assert_eq!(route("/api/v1/crates/new"), Route::Publish);
        assert_eq!(route("/api/v1/crates"), Route::Search);
        assert_eq!(
            route("/api/v1/crates/demo/1.0.0/download"),
            Route::Download("demo".to_owned(), "1.0.0".to_owned())
        );
        assert_eq!(
            route("/api/v1/crates/demo/1.0.0-beta.1/yank"),
            Route::Yank("demo".to_owned(), "1.0.0-beta.1".to_owned())
        );
        assert_eq!(
            route("/api/v1/crates/demo/1.0.0/unyank"),
            Route::Unyank("demo".to_owned(), "1.0.0".to_owned())
        );
        assert_eq!(
            route("/api/v1/crates/demo/owners"),
            Route::Owners("demo".to_owned())
        );
        assert_eq!(
            route("/crates/demo/1.0.0/demo-1.0.0.crate"),
            Route::File("demo".to_owned(), "1.0.0".to_owned())
        );
        assert_eq!(route("/me"), Route::Me);
        assert_eq!(route("/"), Route::Directory);
        for other in [
            "/crates/demo/1.0.0/other-1.0.0.crate",
            "/crates/demo/demo-1.0.0.crate",
            "/api/v1/crates/demo/x/download",
            "/se/rd/Serde",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
        assert_eq!(
            crate_path("demo", "1.0.0"),
            "/crates/demo/1.0.0/demo-1.0.0.crate"
        );
    }
}
