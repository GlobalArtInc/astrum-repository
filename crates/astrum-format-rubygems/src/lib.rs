pub mod gem;
pub mod index;
mod marshal;
pub mod testing;

use std::collections::{BTreeMap, HashMap};

use astrum_blob::{BlobId, ContentStream, Digests, Written};
use astrum_content::format::SearchTarget;
use astrum_content::proxy;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, BrowseLayout, CHALLENGE, ContentError, ContentRequest, Ctx, Format, Freshness, SearchKey,
    Serve, UploadField, UploadForm, UploadSpec, View, group, serve,
};
use astrum_db::entity::{asset, format_token};
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use chrono::Utc;
use futures_util::TryStreamExt;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveValue, ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
use serde_json::json;

use crate::gem::{GemError, Spec};
use crate::index::{InfoEntry, Specs, VersionsLine};

pub const ID: &str = "rubygems";
pub const GEM: &str = "GEM";
pub const GEMSPEC: &str = "GEMSPEC";
pub const INDEX: &str = "INDEX";
pub const REALM: &str = "RubyGemsApiKey";
const TEXT: &str = "text/plain; charset=utf-8";
const BINARY: &str = "application/octet-stream";
const GEM_TYPE: &str = "application/octet-stream";
const MAX_GEMS: usize = 200;
const MAX_INDEX: u64 = 256 * 1024 * 1024;

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "rubygems.platform",
        target: SearchTarget::Asset,
        pointer: &["rubygems", "platform"],
    },
    SearchKey {
        param: "rubygems.summary",
        target: SearchTarget::Asset,
        pointer: &["rubygems", "summary"],
    },
    SearchKey {
        param: "rubygems.description",
        target: SearchTarget::Asset,
        pointer: &["rubygems", "description"],
    },
];

#[derive(Default)]
pub struct Rubygems;

#[derive(Debug, PartialEq, Eq)]
enum Route {
    Versions,
    Names,
    Info(String),
    Specs(Specs),
    Quick(String),
    Gem(String),
    Dependencies,
    Push,
    Yank,
    ApiKey,
    Directory,
    Other,
}

fn file_name<'a>(path: &'a str, prefix: &str, extension: &str) -> Option<&'a str> {
    let file = path.strip_prefix(prefix)?;
    if !gem::valid_file_name(file) {
        return None;
    }
    file.strip_suffix(extension).filter(|full| !full.is_empty())
}

fn route(path: &str) -> Route {
    match path {
        "/versions" => return Route::Versions,
        "/names" => return Route::Names,
        "/api/v1/dependencies" => return Route::Dependencies,
        "/api/v1/gems" => return Route::Push,
        "/api/v1/gems/yank" => return Route::Yank,
        "/api/v1/api_key" => return Route::ApiKey,
        _ => {}
    }
    if path.ends_with('/') {
        return Route::Directory;
    }
    if let Some(kind) = Specs::from_path(path) {
        return Route::Specs(kind);
    }
    if let Some(name) = path.strip_prefix("/info/")
        && gem::valid_name(name)
    {
        return Route::Info(name.to_owned());
    }
    if let Some(full) = file_name(path, gem::QUICK_PREFIX, ".gemspec.rz") {
        return Route::Quick(full.to_owned());
    }
    if let Some(full) = file_name(path, gem::GEMS_PREFIX, ".gem") {
        return Route::Gem(full.to_owned());
    }
    Route::Other
}

fn text(status: StatusCode, message: impl Into<String>) -> Response {
    let body = message.into();
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
        .body(Body::from(body))
        .expect("a plain response is valid")
}

fn body_response(
    request: &ContentRequest,
    content_type: &str,
    body: Vec<u8>,
) -> Result<Response, ContentError> {
    let etag = format!("\"{}\"", index::md5_hex(&body));
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::CACHE_CONTROL, "no-cache");
    if unchanged {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .map_err(ContentError::internal);
    }
    let length = body.len();
    builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, length)
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .map_err(ContentError::internal)
}

fn spec_of(found: &Asset) -> Option<Spec> {
    Spec::from_attributes(found.attributes.get("rubygems")?)
}

fn component(spec: &Spec) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: spec.name.clone(),
        version: spec.version.clone(),
        kind: String::new(),
        attributes: json!({}),
    }
}

async fn read_bytes(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > limit {
        return Err(ContentError::BadGateway(format!("{} is too large", found.path)));
    }
    let store = ctx.content.blob_stores().get(&found.blob_store).await?;
    let Some(content) = store.read(found.blob_id.into()).await? else {
        return Err(ContentError::not_found(found.path.clone()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

async fn read_spec(ctx: &Ctx, store_name: &str, id: BlobId, path: &str) -> Result<Spec, ContentError> {
    let store = ctx.content.blob_stores().get(store_name).await?;
    let Some(content) = store.read(id).await? else {
        return Err(ContentError::not_found(path.to_owned()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    tokio::task::spawn_blocking(move || gem::read_gem(body.as_slice()))
        .await
        .map_err(ContentError::internal)?
        .map_err(|error| match error {
            GemError::NotGem(message) => ContentError::BadRequest(format!("Not a gem file: {message}")),
            GemError::Invalid(message) => ContentError::BadRequest(message),
        })
}

async fn gems(ctx: &Ctx, name: Option<&str>) -> Result<Vec<Asset>, ContentError> {
    let mut query = asset::Entity::find()
        .filter(asset::Column::Repository.eq(ctx.name()))
        .filter(asset::Column::Kind.eq(GEM));
    if let Some(name) = name {
        let pattern = format!(
            "{}{}-%",
            gem::GEMS_PREFIX,
            name.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
        );
        query = query.filter(Expr::col(asset::Column::Path).like(pattern));
    }
    Ok(query
        .order_by_asc(asset::Column::Path)
        .all(ctx.db())
        .await?
        .into_iter()
        .filter(|found| {
            name.is_none_or(|name| spec_of(found).is_some_and(|spec| spec.name == name))
        })
        .collect())
}

struct Hosted {
    entries: BTreeMap<String, Vec<InfoEntry>>,
    updated: Option<chrono::DateTime<Utc>>,
}

async fn hosted_entries(ctx: &Ctx, name: Option<&str>) -> Result<Hosted, ContentError> {
    let mut entries: BTreeMap<String, Vec<InfoEntry>> = BTreeMap::new();
    let mut updated = None;
    for found in gems(ctx, name).await? {
        if !ctx.allowed(View::Read, &found.path) {
            continue;
        }
        let Some(spec) = spec_of(&found) else {
            continue;
        };
        let at = found.blob_updated_at.with_timezone(&Utc);
        updated = updated.max(Some(at));
        entries
            .entry(spec.name.clone())
            .or_default()
            .push(InfoEntry::of(&spec, &found.sha256));
    }
    Ok(Hosted { entries, updated })
}

fn hosted_versions(hosted: &Hosted) -> Vec<u8> {
    let lines: Vec<VersionsLine> = hosted
        .entries
        .iter()
        .map(|(name, list)| VersionsLine {
            name: name.clone(),
            versions: index::versions_of(list),
            checksum: index::md5_hex(&index::info(list)),
        })
        .collect();
    index::versions(
        hosted.updated.unwrap_or_else(|| chrono::DateTime::<Utc>::UNIX_EPOCH),
        &lines,
    )
}

fn versions_map(text: &str) -> BTreeMap<String, Vec<String>> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for line in index::body_lines(text) {
        if let Some(parsed) = VersionsLine::parse(line) {
            let list = map.entry(parsed.name).or_default();
            for version in parsed.versions {
                match version.strip_prefix('-') {
                    Some(yanked) => list.retain(|known| known != yanked),
                    None => list.push(version),
                }
            }
        }
    }
    map
}

impl Rubygems {
    async fn index(&self, ctx: &Ctx, path: &str) -> Result<Option<Vec<u8>>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => self.hosted_index(ctx, path).await,
            RepositoryType::Proxy => {
                let found = proxy::fetch(ctx, path, None, Freshness::Metadata, || {
                    PutAsset::cached(path).kind(INDEX).content_type(match path.ends_with(".gz") {
                        true => BINARY,
                        false => TEXT,
                    })
                })
                .await?;
                match found {
                    Some(found) => Ok(Some(read_bytes(ctx, &found, MAX_INDEX).await?)),
                    None => Ok(None),
                }
            }
            RepositoryType::Group => self.group_index(ctx, path).await,
        }
    }

    async fn hosted_index(&self, ctx: &Ctx, path: &str) -> Result<Option<Vec<u8>>, ContentError> {
        Ok(match route(path) {
            Route::Versions => Some(hosted_versions(&hosted_entries(ctx, None).await?)),
            Route::Names => Some(index::names(hosted_entries(ctx, None).await?.entries.keys())),
            Route::Info(name) => {
                let hosted = hosted_entries(ctx, Some(&name)).await?;
                hosted.entries.get(&name).map(|list| index::info(list))
            }
            Route::Specs(kind) => {
                let hosted = hosted_entries(ctx, None).await?;
                let versions: BTreeMap<String, Vec<String>> = hosted
                    .entries
                    .iter()
                    .map(|(name, list)| (name.clone(), index::versions_of(list)))
                    .collect();
                Some(index::legacy_specs(kind, &versions))
            }
            _ => None,
        })
    }

    async fn member_indexes(&self, ctx: &Ctx, path: &str) -> Result<Vec<(Ctx, String)>, ContentError> {
        let mut found = Vec::new();
        for member in group::members(ctx).await? {
            let member = ctx.member(member);
            match Box::pin(self.index(&member, path)).await {
                Ok(Some(bytes)) => found.push((member, String::from_utf8_lossy(&bytes).into_owned())),
                Ok(None) => {}
                Err(error) => {
                    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member has no index");
                }
            }
        }
        Ok(found)
    }

    async fn group_versions(&self, ctx: &Ctx) -> Result<(String, BTreeMap<String, Vec<String>>), ContentError> {
        let members = self.member_indexes(ctx, "/versions").await?;
        let mut by_name: BTreeMap<String, Vec<(usize, VersionsLine)>> = BTreeMap::new();
        for (index, (_, text)) in members.iter().enumerate() {
            let map = versions_map(text);
            let checksums: HashMap<String, String> = index::body_lines(text)
                .into_iter()
                .filter_map(VersionsLine::parse)
                .map(|line| (line.name, line.checksum))
                .collect();
            for (name, versions) in map {
                let checksum = checksums.get(&name).cloned().unwrap_or_default();
                by_name.entry(name.clone()).or_default().push((
                    index,
                    VersionsLine {
                        name,
                        versions,
                        checksum,
                    },
                ));
            }
        }
        let mut lines = Vec::with_capacity(by_name.len());
        let mut versions = BTreeMap::new();
        for (name, mut found) in by_name {
            if found.len() == 1 {
                let (_, line) = found.remove(0);
                versions.insert(name, line.versions.clone());
                lines.push(line);
                continue;
            }
            let mut infos = Vec::new();
            for (index, _) in &found {
                let member = &members[*index].0;
                if let Some(bytes) = Box::pin(self.index(member, &format!("/info/{name}"))).await? {
                    infos.push(String::from_utf8_lossy(&bytes).into_owned());
                }
            }
            let merged = index::merge_info(&infos);
            let list = index::versions_of(&index::parse_info(&String::from_utf8_lossy(&merged)));
            if list.is_empty() {
                continue;
            }
            versions.insert(name.clone(), list.clone());
            lines.push(VersionsLine {
                name,
                versions: list,
                checksum: index::md5_hex(&merged),
            });
        }
        let text = String::from_utf8(index::versions(Utc::now(), &lines)).unwrap_or_default();
        let header_end = text.find(&format!("{}\n", index::SEPARATOR)).unwrap_or(0);
        let stable = format!("created_at: 1970-01-01T00:00:00Z\n{}", &text[header_end..]);
        Ok((stable, versions))
    }

    async fn group_index(&self, ctx: &Ctx, path: &str) -> Result<Option<Vec<u8>>, ContentError> {
        Ok(match route(path) {
            Route::Versions => Some(self.group_versions(ctx).await?.0.into_bytes()),
            Route::Names => {
                let members = self.member_indexes(ctx, "/names").await?;
                let names: std::collections::BTreeSet<String> = members
                    .iter()
                    .flat_map(|(_, text)| {
                        index::body_lines(text)
                            .into_iter()
                            .map(str::trim)
                            .filter(|name| !name.is_empty())
                            .map(str::to_owned)
                            .collect::<Vec<_>>()
                    })
                    .collect();
                Some(index::names(names.iter()))
            }
            Route::Info(_) => {
                let infos: Vec<String> = self
                    .member_indexes(ctx, path)
                    .await?
                    .into_iter()
                    .map(|(_, text)| text)
                    .collect();
                match infos.is_empty() {
                    true => None,
                    false => Some(index::merge_info(&infos)),
                }
            }
            Route::Specs(kind) => Some(index::legacy_specs(kind, &self.group_versions(ctx).await?.1)),
            _ => None,
        })
    }

    async fn info_entries(&self, ctx: &Ctx, name: &str) -> Result<Option<Vec<InfoEntry>>, ContentError> {
        Ok(self
            .index(ctx, &format!("/info/{name}"))
            .await?
            .map(|bytes| index::parse_info(&String::from_utf8_lossy(&bytes))))
    }

    async fn dependencies(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let mut names: Vec<String> = Vec::new();
        for (key, value) in url::form_urlencoded::parse(request.query.as_deref().unwrap_or("").as_bytes()) {
            if key != "gems" {
                continue;
            }
            for name in value.split(',').map(str::trim).filter(|name| !name.is_empty()) {
                if !gem::valid_name(name) {
                    return Ok(text(StatusCode::BAD_REQUEST, format!("Invalid gem name: {name}")));
                }
                if !names.iter().any(|known| known == name) {
                    names.push(name.to_owned());
                }
            }
        }
        if names.len() > MAX_GEMS {
            return Ok(text(
                StatusCode::UNPROCESSABLE_ENTITY,
                format!("Too many gems (use less than {MAX_GEMS} gems)"),
            ));
        }
        let mut entries = Vec::new();
        for name in names {
            if let Some(list) = self.info_entries(ctx, &name).await? {
                entries.push((name, list));
            }
        }
        body_response(request, BINARY, index::dependencies(&entries))
    }

    async fn quick(&self, ctx: &Ctx, request: &ContentRequest, full: &str) -> Result<Response, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let found = store::find(ctx.db(), ctx.name(), &gem::gem_path(full))
                    .await?
                    .and_then(|found| spec_of(&found))
                    .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
                body_response(request, BINARY, index::quick_spec(&found))
            }
            _ => {
                let (owner, found) = self
                    .lookup(ctx, &request.path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
                serve(&owner, &found, &request.method, &request.headers, Serve::default()).await
            }
        }
    }

    async fn lookup(&self, ctx: &Ctx, path: &str) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => Ok(group::first(ctx, |member| async move { self.get(&member, path).await })
                .await?),
            _ => Ok(self.get(ctx, path).await?.map(|found| (ctx.clone(), found))),
        }
    }

    async fn api_key(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let Some(subject) = ctx.subject.as_ref().filter(|subject| !subject.anonymous) else {
            return Ok(Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .header(header::WWW_AUTHENTICATE, CHALLENGE)
                .header(header::CONTENT_TYPE, "text/plain; charset=utf-8")
                .body(Body::from("Authentication required to obtain a RubyGems API key"))
                .map_err(ContentError::internal)?);
        };
        if request.method == Method::DELETE {
            let deleted = format_token::Entity::delete_many()
                .filter(format_token::Column::Realm.eq(REALM))
                .filter(format_token::Column::UserId.eq(&subject.user_id))
                .exec(ctx.db())
                .await?;
            return Ok(match deleted.rows_affected {
                0 => text(StatusCode::NOT_FOUND, "No RubyGems API key to revoke"),
                _ => text(StatusCode::OK, "RubyGems API key revoked"),
            });
        }
        if !ctx.security.realm_active(REALM) {
            return Ok(text(
                StatusCode::CONFLICT,
                "The RubyGems API Key Realm is not active; ask an administrator to activate it, or use 'Basic <base64 of user:password>' as the API key",
            ));
        }
        let token = format!("{REALM}.{}", astrum_security::password::random(40));
        let now = Utc::now().fixed_offset();
        format_token::Entity::insert(format_token::ActiveModel {
            token_hash: ActiveValue::Set(astrum_security::tokens::hash(&token)),
            realm: ActiveValue::Set(REALM.to_owned()),
            user_id: ActiveValue::Set(subject.user_id.clone()),
            created_at: ActiveValue::Set(now),
            expires_at: ActiveValue::Set(None),
            last_used_at: ActiveValue::Set(None),
        })
        .exec(ctx.db())
        .await?;
        Ok(Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/plain")
            .body(Body::from(token))
            .map_err(ContentError::internal)?)
    }

    async fn push(&self, ctx: &Ctx, body: ContentStream) -> Result<Response, ContentError> {
        let written = store::write_blob(ctx, body).await?;
        if written.size == 0 {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Ok(text(StatusCode::BAD_REQUEST, "Missing gem file in the request body"));
        }
        let spec = match read_spec(ctx, &written.store, written.id, "/api/v1/gems").await {
            Ok(spec) => spec,
            Err(error) => {
                store::discard(ctx.content.clone(), &written.store, written.id.uuid());
                return Ok(text(StatusCode::UNPROCESSABLE_ENTITY, error.to_string()));
            }
        };
        let path = spec.gem_path();
        let redeploys = ctx
            .repository
            .attributes
            .storage
            .as_ref()
            .and_then(|storage| storage.write_policy)
            .unwrap_or(WritePolicy::Allow);
        let exists = store::find(ctx.db(), ctx.name(), &path).await?.is_some();
        let view = if exists && redeploys == WritePolicy::Allow { View::Edit } else { View::Add };
        if let Err(error) = ctx.require(view, &path) {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Err(error);
        }
        if exists && redeploys == WritePolicy::AllowOnce {
            store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            return Ok(text(
                StatusCode::CONFLICT,
                format!(
                    "Repushing of gem versions is not allowed: repository {} does not allow redeployment",
                    ctx.name()
                ),
            ));
        }
        self.adopt(ctx, &spec, PutAsset::hosted(ctx, &path), &written).await?;
        Ok(text(
            StatusCode::OK,
            format!("Successfully registered gem: {} ({})", spec.name, spec.version_and_platform()),
        ))
    }

    async fn adopt(&self, ctx: &Ctx, spec: &Spec, put: PutAsset, written: &Written) -> Result<Asset, ContentError> {
        let put = put
            .component(component(spec))
            .kind(GEM)
            .content_type(GEM_TYPE)
            .attributes(json!({"rubygems": spec.attributes()}));
        store::adopt(ctx, put, written).await
    }

    async fn yank(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let mut form: HashMap<String, String> = url::form_urlencoded::parse(request.query.as_deref().unwrap_or("").as_bytes())
            .into_owned()
            .collect();
        let body = request.bytes(64 * 1024).await?;
        for (key, value) in url::form_urlencoded::parse(&body).into_owned() {
            form.entry(key).or_insert(value);
        }
        let name = form.get("gem_name").map(String::as_str).unwrap_or_default();
        let version = form.get("version").map(|value| value.trim()).unwrap_or_default();
        let platform = form.get("platform").map(String::as_str).filter(|value| !value.is_empty());
        if !gem::valid_name(name) {
            return Ok(text(StatusCode::BAD_REQUEST, "A valid gem_name is required"));
        }
        if !gem::valid_version(version) {
            return Ok(text(StatusCode::BAD_REQUEST, "A valid version is required"));
        }
        if let Some(platform) = platform
            && !gem::valid_platform(platform)
        {
            return Ok(text(StatusCode::BAD_REQUEST, format!("Invalid platform: {platform}")));
        }
        let platform = gem::normalize_platform(platform);
        let path = gem::gem_path(&gem::full_name(name, version, &platform));
        let version_and_platform = gem::version_and_platform(version, &platform);
        let Some(found) = store::find(ctx.db(), ctx.name(), &path).await? else {
            return Ok(text(
                StatusCode::NOT_FOUND,
                format!("The version {version_and_platform} of {name} does not exist."),
            ));
        };
        ctx.require(View::Delete, &path)?;
        store::delete_asset(&ctx.content, found).await?;
        Ok(text(
            StatusCode::OK,
            format!("Successfully deleted gem: {name} ({version_and_platform})"),
        ))
    }

    async fn proxy_gem(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        let kind = match route(path) {
            Route::Gem(_) => GEM,
            Route::Quick(_) => GEMSPEC,
            _ => INDEX,
        };
        let freshness = match kind {
            INDEX => Freshness::Metadata,
            _ => Freshness::Content,
        };
        let found = proxy::fetch(ctx, path, None, freshness, || {
            PutAsset::cached(path).kind(kind).content_type(BINARY)
        })
        .await?;
        let Some(found) = found else {
            return Ok(None);
        };
        if kind != GEM || found.attributes.get("rubygems").is_some() {
            return Ok(Some(found));
        }
        match read_spec(ctx, &found.blob_store, found.blob_id.into(), path).await {
            Ok(spec) => {
                let written = Written {
                    id: found.blob_id.into(),
                    store: found.blob_store.clone(),
                    size: found.size.max(0) as u64,
                    digests: Digests {
                        sha1: found.sha1.clone(),
                        sha256: found.sha256.clone(),
                        sha512: found.sha512.clone(),
                        md5: found.md5.clone(),
                    },
                };
                let mut put = PutAsset::cached(path);
                put.remote_etag = found.remote_etag.clone();
                put.remote_last_modified = found.remote_last_modified;
                Ok(Some(self.adopt(ctx, &spec, put, &written).await?))
            }
            Err(error) => {
                tracing::warn!(repository = ctx.name(), path, %error, "the specification of a cached gem cannot be read");
                Ok(Some(found))
            }
        }
    }

    async fn read(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.clone();
        let route = route(&path);
        if route == Route::ApiKey {
            return self.api_key(ctx, &request).await;
        }
        if route == Route::Directory {
            return Ok(text(StatusCode::OK, format!("{} is a RubyGems repository; use it as a gem source.", ctx.name())));
        }
        ctx.require(View::Read, &path)?;
        match route {
            Route::Versions | Route::Names | Route::Info(_) => {
                let body = self
                    .index(ctx, &path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.clone()))?;
                body_response(&request, TEXT, body)
            }
            Route::Specs(_) => {
                let body = self
                    .index(ctx, &path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.clone()))?;
                body_response(&request, BINARY, body)
            }
            Route::Dependencies => self.dependencies(ctx, &request).await,
            Route::Quick(full) => self.quick(ctx, &request, &full).await,
            Route::Gem(_) => {
                let (owner, found) = self
                    .lookup(ctx, &path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.clone()))?;
                serve(&owner, &found, &request.method, &request.headers, Serve::download()).await
            }
            _ => Err(ContentError::not_found(path)),
        }
    }
}

#[async_trait]
impl Format for Rubygems {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        match (&request.method, route, ctx.kind()) {
            (_, Route::ApiKey, _) => self.api_key(&ctx, &request).await,
            (&Method::GET | &Method::HEAD, _, _) => self.read(&ctx, request).await,
            (&Method::POST, Route::Push, RepositoryType::Hosted) => {
                self.push(&ctx, request.into_stream()).await
            }
            (&Method::DELETE, Route::Yank, RepositoryType::Hosted) => self.yank(&ctx, request).await,
            (&Method::POST, Route::Push, _) | (&Method::DELETE, Route::Yank, _) => Ok(text(
                StatusCode::METHOD_NOT_ALLOWED,
                format!("{} is a {} repository and does not take gems", ctx.name(), ctx.kind().as_str()),
            )),
            (method, _, _) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: "GET,HEAD",
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => store::find(ctx.db(), ctx.name(), path).await,
            RepositoryType::Proxy => self.proxy_gem(ctx, path).await,
            RepositoryType::Group => Ok(group::first(ctx, |member| async move { self.get(&member, path).await })
                .await?
                .map(|(_, found)| found)),
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: false,
            component_fields: vec![],
            asset_fields: vec![UploadField::file("asset")],
        })
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "only hosted repositories take uploads".to_owned(),
            ));
        }
        let file = form
            .files
            .get("rubygems.asset")
            .ok_or_else(|| ContentError::BadRequest("rubygems.asset is required".to_owned()))?;
        let response = self.push(&ctx, UploadForm::stream(file).await?).await?;
        let status = response.status();
        if status.is_success() {
            return Ok(());
        }
        let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
            .await
            .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
            .unwrap_or_default();
        Err(match status {
            StatusCode::CONFLICT => ContentError::Conflict(body),
            _ => ContentError::BadRequest(body),
        })
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NULL THEN a.path ELSE '/' || c.name || '/' || c.version || regexp_replace(a.path, '^.*/', '/') END",
            ),
            generated: &[],
        }
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_indexes_gems_and_the_api() {
        assert_eq!(route("/versions"), Route::Versions);
        assert_eq!(route("/info/rails"), Route::Info("rails".to_owned()));
        assert_eq!(route("/info/../x"), Route::Other);
        assert_eq!(route("/specs.4.8.gz"), Route::Specs(Specs::All));
        assert_eq!(route("/gems/hola-1.0.0.gem"), Route::Gem("hola-1.0.0".to_owned()));
        assert_eq!(
            route("/quick/Marshal.4.8/hola-1.0.0.gemspec.rz"),
            Route::Quick("hola-1.0.0".to_owned())
        );
        assert_eq!(route("/api/v1/gems"), Route::Push);
        assert_eq!(route("/api/v1/gems/yank"), Route::Yank);
        assert_eq!(route("/api/v1/api_key"), Route::ApiKey);
        assert_eq!(route("/gems/.hidden.gem"), Route::Other);
    }

    #[test]
    fn versions_files_drop_yanked_versions() {
        let map = versions_map("created_at: x\n---\nhola 1.0.0,1.1.0 abc\nhola -1.0.0 def\n");
        assert_eq!(map["hola"], ["1.1.0"]);
    }
}
