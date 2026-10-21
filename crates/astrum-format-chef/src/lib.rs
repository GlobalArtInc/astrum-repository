pub mod archive;
mod docs;
mod hosted;
pub mod names;
mod remote;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use astrum_content::format::{SearchKey, SearchTarget};
use astrum_content::store;
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Grant, Serve, UploadField, UploadForm,
    UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use futures_util::TryStreamExt;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::docs::{Cookbook, Item, Query, Universe, Urls};
use crate::names::Wanted;

pub const ID: &str = "chef";
pub const COOKBOOK: &str = "cookbook";
pub const METADATA: &str = "metadata";
pub const GZIP_TYPE: &str = "application/gzip";
pub const MAX_TARBALL: u64 = 512 * 1024 * 1024;
const MAX_FIELD: u64 = 64 * 1024;
const MAX_RENDERED: usize = 16;
const LIST_PATH: &str = "/api/v1/cookbooks";
const SEARCH_PATH: &str = "/api/v1/search";
const TARBALL_FIELD: &str = "tarball";
const COOKBOOK_FIELD: &str = "cookbook";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "chef.license",
        target: SearchTarget::Component,
        pointer: &["chef", "license"],
    },
    SearchKey {
        param: "chef.maintainer",
        target: SearchTarget::Component,
        pointer: &["chef", "maintainer"],
    },
    SearchKey {
        param: "chef.category",
        target: SearchTarget::Component,
        pointer: &["chef", "category"],
    },
];

const KEYWORDS: &[&[&str]] = &[&["chef", "description"], &["chef", "maintainer"]];

struct Rendered {
    signature: String,
    body: Bytes,
    etag: String,
}

#[derive(Default)]
pub struct Chef {
    rendered: Mutex<HashMap<(String, String), Arc<Rendered>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Root,
    Universe,
    Cookbooks,
    Search,
    Cookbook(String),
    Version(String, Wanted),
    Download(String, Wanted),
    Tarball(String, String),
    Other,
}

impl Route {
    fn is_api(&self) -> bool {
        !matches!(self, Route::Root | Route::Tarball(..) | Route::Other)
    }
}

fn route(path: &str) -> Route {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Route::Root;
    }
    let parts: Vec<&str> = trimmed.split('/').collect();
    let named = |name: &str| names::valid_name(name);
    match parts.as_slice() {
        ["universe"] => Route::Universe,
        ["api", "v1", "cookbooks"] => Route::Cookbooks,
        ["api", "v1", "search"] => Route::Search,
        ["api", "v1", "cookbooks", name] if named(name) => Route::Cookbook((*name).to_owned()),
        ["api", "v1", "cookbooks", name, "versions", version] if named(name) => {
            match names::wanted(version) {
                Some(wanted) => Route::Version((*name).to_owned(), wanted),
                None => Route::Other,
            }
        }
        [
            "api",
            "v1",
            "cookbooks",
            name,
            "versions",
            version,
            "download",
        ] if named(name) => match names::wanted(version) {
            Some(wanted) => Route::Download((*name).to_owned(), wanted),
            None => Route::Other,
        },
        ["cookbooks", name, version, file]
            if !path.ends_with('/')
                && named(name)
                && names::valid_version(version)
                && *file == names::tarball_name(name, version) =>
        {
            Route::Tarball((*name).to_owned(), (*version).to_owned())
        }
        _ => Route::Other,
    }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn directory_page(ctx: &Ctx) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let url = escape(&ctx.repository_url());
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a Chef Supermarket repository of cookbooks. Use <code>source \"{url}\"</code> in a Berksfile, <code>default_source :supermarket, \"{url}\"</code> in a Policyfile, or <code>knife supermarket download &lt;cookbook&gt; -m {url}</code>; the universe is at <code>{url}/universe</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn may_read(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Read).any() {
        true => Ok(()),
        false => ctx.require(View::Read, "/"),
    }
}

fn signed_in(ctx: &Ctx) -> Result<(), ContentError> {
    match &ctx.subject {
        Some(subject) if !subject.anonymous => Ok(()),
        _ if ctx.is_trusted() => Ok(()),
        _ => Err(ContentError::Unauthorized),
    }
}

fn reader(ctx: &Ctx) -> impl Fn(&str) -> bool + '_ {
    let grant = ctx.grant(View::Read);
    let format = ctx.format_id();
    move |path: &str| match &grant {
        Grant::Whole => true,
        Grant::Selected(selectors) => selectors
            .iter()
            .any(|selector| selector.selector.matches(format, path)),
        Grant::Nothing => false,
    }
}

fn item_visible(read: &impl Fn(&str) -> bool, item: &Item) -> bool {
    read(&names::folder_path(&item.name))
        || item
            .versions
            .iter()
            .any(|version| read(&names::tarball_path(&item.name, version)))
}

fn log_member_failure(ctx: &Ctx, member: &Ctx, error: &ContentError) {
    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
}

fn missing_cookbook(name: &str) -> ContentError {
    ContentError::not_found(format!("cookbook {name} was not found"))
}

fn missing_version(name: &str, version: &str) -> ContentError {
    ContentError::not_found(format!(
        "version {version} of cookbook {name} was not found"
    ))
}

fn unauthorized_message(ctx: &Ctx, signed: bool) -> String {
    match signed {
        true => format!(
            "this request is signed with a Chef client key, which Astrum Repository does not check; publish the cookbook archive with a user of Astrum Repository instead, for example curl -u <user> -F 'cookbook={{\"category\":\"Other\"}}' -F tarball=@<name>.tgz {}/api/v1/cookbooks",
            ctx.repository_url()
        ),
        false => format!(
            "sign in with a user of Astrum Repository (basic authentication with a user name and a password or an access token) to use {}",
            ctx.name()
        ),
    }
}

fn not_allowed(method: &Method, kind: RepositoryType) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match kind {
            RepositoryType::Hosted => "GET,HEAD,POST,DELETE",
            RepositoryType::Proxy => "GET,HEAD,DELETE",
            RepositoryType::Group => "GET,HEAD",
        },
    }
}

enum Source {
    Built(String, Universe),
    Cached(Box<(Ctx, Asset)>),
}

impl Source {
    fn signature(&self) -> String {
        match self {
            Source::Built(signature, _) => signature.clone(),
            Source::Cached(cached) => {
                let (member, found) = cached.as_ref();
                format!("{}:{}:{}", member.name(), found.id, found.sha1)
            }
        }
    }
}

impl Chef {
    async fn sources(&self, ctx: &Ctx) -> Result<(Vec<Source>, bool), ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let built = hosted::universe(&hosted::stored(ctx, None).await?);
                let digest = Sha256::digest(serde_json::to_vec(&built).unwrap_or_default());
                let signature = format!("{}:{}", ctx.name(), hex::encode(&digest[..16]));
                Ok((vec![Source::Built(signature, built)], true))
            }
            RepositoryType::Proxy => Ok((
                remote::universe_asset(ctx)
                    .await?
                    .map(|found| Source::Cached(Box::new((ctx.clone(), found))))
                    .into_iter()
                    .collect(),
                true,
            )),
            RepositoryType::Group => {
                let mut all = Vec::new();
                let mut complete = true;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.sources(&member)).await {
                        Ok((found, whole)) => {
                            complete &= whole;
                            all.extend(found);
                        }
                        Err(error) => {
                            complete = false;
                            log_member_failure(ctx, &member, &error);
                        }
                    }
                }
                Ok((all, complete))
            }
        }
    }

    fn cached(&self, key: &(String, String), signature: &str) -> Option<Arc<Rendered>> {
        self.rendered
            .lock()
            .expect("the universe cache lock is never poisoned")
            .get(key)
            .filter(|cached| cached.signature == signature)
            .cloned()
    }

    fn remember(&self, key: (String, String), rendered: Arc<Rendered>) {
        let mut cache = self
            .rendered
            .lock()
            .expect("the universe cache lock is never poisoned");
        if cache.len() >= MAX_RENDERED && !cache.contains_key(&key) {
            cache.clear();
        }
        cache.insert(key, rendered);
    }

    async fn universe(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        may_read(ctx)?;
        let (sources, mut complete) = self.sources(ctx).await?;
        let signature = sources
            .iter()
            .map(Source::signature)
            .collect::<Vec<_>>()
            .join("|");
        let whole = matches!(ctx.grant(View::Read), Grant::Whole);
        let urls = Urls::of(ctx);
        let key = (ctx.name().to_owned(), urls.base.clone());
        if whole && let Some(cached) = self.cached(&key, &signature) {
            return Ok(docs::etagged(request, cached.body.clone(), &cached.etag));
        }
        let mut universe = Universe::new();
        for source in sources {
            let part = match source {
                Source::Built(_, built) => built,
                Source::Cached(cached) => {
                    let (member, found) = *cached;
                    match remote::universe_of(&member, &found).await {
                        Ok(part) => part,
                        Err(error) => {
                            complete = false;
                            log_member_failure(ctx, &member, &error);
                            continue;
                        }
                    }
                }
            };
            docs::merge_universe(&mut universe, part);
        }
        let body = match whole {
            true => {
                let urls = urls.clone();
                tokio::task::spawn_blocking(move || {
                    docs::render_universe(&urls, &universe, |_, _| true)
                })
                .await
                .map_err(ContentError::internal)?
            }
            false => {
                let read = reader(ctx);
                docs::render_universe(&urls, &universe, |name, version| {
                    read(&names::tarball_path(name, version))
                })
            }
        };
        let etag = hex::encode(&Sha256::digest(&body)[..16]);
        let body = Bytes::from(body);
        if whole && complete {
            self.remember(
                key,
                Arc::new(Rendered {
                    signature,
                    body: body.clone(),
                    etag: etag.clone(),
                }),
            );
        }
        Ok(docs::etagged(request, body, &etag))
    }

    async fn cookbook(&self, ctx: &Ctx, name: &str) -> Result<Option<Cookbook>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(hosted::cookbook(
                name,
                &hosted::stored(ctx, Some(name)).await?,
            )),
            RepositoryType::Proxy => remote::cookbook(ctx, name).await,
            RepositoryType::Group => {
                let mut merged: Option<Cookbook> = None;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.cookbook(&member, name)).await {
                        Ok(Some(found)) => match &mut merged {
                            Some(known) => known.merge(found),
                            None => merged = Some(found),
                        },
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(merged)
            }
        }
    }

    async fn version(
        &self,
        ctx: &Ctx,
        name: &str,
        version: &str,
    ) -> Result<Option<Map<String, Value>>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(hosted::stored(ctx, Some(name))
                .await?
                .iter()
                .find(|stored| stored.version() == version)
                .map(hosted::Stored::version_document)),
            RepositoryType::Proxy => remote::version(ctx, name, version).await,
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.version(&member, name, version)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn resolve(
        &self,
        ctx: &Ctx,
        name: &str,
        wanted: &Wanted,
    ) -> Result<String, ContentError> {
        match wanted {
            Wanted::Exact(version) => Ok(version.clone()),
            Wanted::Latest => {
                let found = self
                    .cookbook(ctx, name)
                    .await?
                    .ok_or_else(|| missing_cookbook(name))?;
                let read = reader(ctx);
                names::highest(
                    found
                        .versions
                        .iter()
                        .map(String::as_str)
                        .filter(|version| read(&names::tarball_path(name, version))),
                )
                .map(str::to_owned)
                .ok_or_else(|| missing_cookbook(name))
            }
        }
    }

    async fn tarball(
        &self,
        ctx: &Ctx,
        name: &str,
        version: &str,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                Ok(
                    store::find(ctx.db(), ctx.name(), &names::tarball_path(name, version))
                        .await?
                        .map(|found| (ctx.clone(), found)),
                )
            }
            RepositoryType::Proxy => Ok(remote::tarball(ctx, name, version)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.tarball(&member, name, version)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn items(
        &self,
        ctx: &Ctx,
        path: &str,
        query: &Query,
        wanted: usize,
    ) -> Result<(usize, Vec<Item>), ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let mut items = hosted::items(hosted::stored(ctx, None).await?);
                items.retain(|item| {
                    item.matches(&query.q)
                        && query.user.as_deref().is_none_or(|user| {
                            item.maintainer
                                .as_str()
                                .is_some_and(|maintainer| maintainer.eq_ignore_ascii_case(user))
                        })
                });
                Ok((items.len(), items))
            }
            RepositoryType::Proxy => remote::gather(ctx, path, query, wanted).await,
            RepositoryType::Group => {
                let mut merged: Vec<Item> = Vec::new();
                let mut total = 0;
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.items(&member, path, query, wanted)).await {
                        Ok((count, items)) => {
                            total += count;
                            for item in items {
                                match merged.iter_mut().find(|known| known.name == item.name) {
                                    Some(known) => {
                                        total = total.saturating_sub(1);
                                        for version in item.versions {
                                            if !known.versions.contains(&version) {
                                                known.versions.push(version);
                                            }
                                        }
                                    }
                                    None => merged.push(item),
                                }
                            }
                        }
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok((total.max(merged.len()), merged))
            }
        }
    }

    async fn listing(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        path: &str,
    ) -> Result<Response, ContentError> {
        may_read(ctx)?;
        let mut query = Query::of(request.query.as_deref());
        if path == LIST_PATH {
            query.q.clear();
        }
        let urls = Urls::of(ctx);
        let read = reader(ctx);
        if ctx.kind() == RepositoryType::Proxy {
            let (total, items) =
                remote::listing(ctx, path, &query, query.start, query.items).await?;
            let shown: Vec<Value> = items
                .iter()
                .filter(|item| item_visible(&read, item))
                .map(|item| item.render(&urls))
                .collect();
            return Ok(docs::json(
                StatusCode::OK,
                Some(request),
                &serde_json::json!({"start": query.start, "total": total, "items": shown}),
            ));
        }
        let (total, mut items) = self
            .items(ctx, path, &query, query.start + query.items)
            .await?;
        let before = items.len();
        items.retain(|item| item_visible(&read, item));
        let total = total.saturating_sub(before - items.len());
        query.sort(&mut items);
        Ok(docs::json(
            StatusCode::OK,
            Some(request),
            &query.page(&urls, total, &items),
        ))
    }

    async fn cookbook_document(&self, ctx: &Ctx, name: &str) -> Result<Value, ContentError> {
        may_read(ctx)?;
        let read = reader(ctx);
        self.cookbook(ctx, name)
            .await?
            .and_then(|found| {
                found.render(&Urls::of(ctx), |version| {
                    read(&names::tarball_path(name, version))
                })
            })
            .ok_or_else(|| missing_cookbook(name))
    }

    async fn version_document(
        &self,
        ctx: &Ctx,
        name: &str,
        wanted: &Wanted,
    ) -> Result<Value, ContentError> {
        may_read(ctx)?;
        let version = self.resolve(ctx, name, wanted).await?;
        if !ctx.allowed(View::Read, &names::tarball_path(name, &version)) {
            return Err(missing_version(name, &version));
        }
        let document = self
            .version(ctx, name, &version)
            .await?
            .ok_or_else(|| missing_version(name, &version))?;
        Ok(docs::render_version(
            &Urls::of(ctx),
            name,
            &version,
            document,
        ))
    }

    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        name: &str,
        wanted: &Wanted,
    ) -> Result<Response, ContentError> {
        let version = match wanted {
            Wanted::Exact(version) => version.clone(),
            Wanted::Latest => {
                may_read(ctx)?;
                self.resolve(ctx, name, wanted).await?
            }
        };
        ctx.require(View::Read, &names::tarball_path(name, &version))?;
        let (owner, found) = self
            .tarball(ctx, name, &version)
            .await?
            .ok_or_else(|| missing_version(name, &version))?;
        serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await
    }

    async fn publish(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        signed_in(ctx)?;
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(format!(
                "{} is a {} repository; publish cookbooks to a hosted Chef Supermarket repository",
                ctx.name(),
                ctx.kind()
            )));
        }
        hosted::may_add(ctx)?;
        let boundary = request
            .header("content-type")
            .and_then(|value| multer::parse_boundary(value).ok())
            .ok_or_else(|| {
                ContentError::BadRequest(
                    "publish a cookbook as a multipart/form-data request with a part named tarball (the cookbook archive) and optionally a part named cookbook (JSON such as {\"category\":\"Other\"})"
                        .to_owned(),
                )
            })?;
        let limits = multer::Constraints::new().size_limit(
            multer::SizeLimit::new()
                .per_field(MAX_FIELD)
                .for_field(TARBALL_FIELD, MAX_TARBALL),
        );
        let mut multipart = multer::Multipart::with_constraints(
            request.take_body().into_data_stream(),
            boundary,
            limits,
        );
        let bad = |error: multer::Error| match error {
            multer::Error::FieldSizeExceeded { .. } | multer::Error::StreamSizeExceeded { .. } => {
                ContentError::BadRequest(format!(
                    "a part of the upload is too large; the cookbook archive may have {MAX_TARBALL} bytes"
                ))
            }
            other => ContentError::BadRequest(format!("the upload form cannot be read: {other}")),
        };
        let mut uploaded = None;
        let mut described: Option<String> = None;
        let discard = |written: &Option<astrum_blob::Written>| {
            if let Some(written) = written {
                store::discard(ctx.content.clone(), &written.store, written.id.uuid());
            }
        };
        loop {
            let part = match multipart.next_field().await {
                Ok(Some(part)) => part,
                Ok(None) => break,
                Err(error) => {
                    discard(&uploaded);
                    return Err(bad(error));
                }
            };
            match part.name().unwrap_or_default() {
                TARBALL_FIELD if uploaded.is_none() => {
                    let body: astrum_blob::ContentStream =
                        Box::pin(part.map_err(std::io::Error::other));
                    uploaded = Some(store::write_blob(ctx, body).await.map_err(|error| {
                        tracing::info!(repository = ctx.name(), %error, "a cookbook upload could not be stored");
                        ContentError::BadRequest(
                            "the cookbook archive could not be received; check its size and publish again"
                                .to_owned(),
                        )
                    })?);
                }
                COOKBOOK_FIELD if described.is_none() => match part.text().await {
                    Ok(text) => described = Some(text),
                    Err(error) => {
                        discard(&uploaded);
                        return Err(bad(error));
                    }
                },
                _ => {
                    let mut part = part;
                    loop {
                        match part.chunk().await {
                            Ok(Some(_)) => {}
                            Ok(None) => break,
                            Err(error) => {
                                discard(&uploaded);
                                return Err(bad(error));
                            }
                        }
                    }
                }
            }
        }
        let Some(written) = uploaded else {
            return Err(ContentError::BadRequest(
                "the upload has no part named tarball with the cookbook archive".to_owned(),
            ));
        };
        let category = match described.as_deref().map(str::trim) {
            None | Some("") => None,
            Some(text) => match serde_json::from_str::<Value>(text) {
                Ok(Value::Object(fields)) => fields
                    .get("category")
                    .and_then(Value::as_str)
                    .map(str::to_owned),
                _ => {
                    discard(&Some(written));
                    return Err(ContentError::BadRequest(
                        "the cookbook part must be a JSON object such as {\"category\":\"Other\"}"
                            .to_owned(),
                    ));
                }
            },
        };
        let metadata = hosted::adopt(ctx, written, category).await?;
        let document = hosted::cookbook(
            &metadata.name,
            &hosted::stored(ctx, Some(&metadata.name)).await?,
        )
        .and_then(|found| found.render(&Urls::of(ctx), |_| true))
        .ok_or_else(|| missing_cookbook(&metadata.name))?;
        Ok(docs::json(StatusCode::CREATED, None, &document))
    }

    async fn delete_path(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        if !store::delete(ctx, path).await? {
            return Err(ContentError::not_found(path.to_owned()));
        }
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }

    async fn delete_version(
        &self,
        ctx: &Ctx,
        name: &str,
        version: &str,
    ) -> Result<Response, ContentError> {
        let path = names::tarball_path(name, version);
        if ctx.kind() == RepositoryType::Proxy {
            ctx.require(View::Delete, &path)?;
            let cached = store::delete(ctx, &remote::version_path(name, version)).await?;
            let stored = store::delete(ctx, &path).await?;
            return match cached || stored {
                true => Ok(StatusCode::NO_CONTENT.into_response()),
                false => Err(missing_version(name, version)),
            };
        }
        let stored = hosted::stored(ctx, Some(name)).await?;
        let found = stored
            .into_iter()
            .find(|stored| stored.version() == version)
            .ok_or_else(|| missing_version(name, version))?;
        ctx.require(View::Delete, &path)?;
        let document =
            docs::render_version(&Urls::of(ctx), name, version, found.version_document());
        store::delete_asset(&ctx.content, found.asset).await?;
        Ok(docs::json(StatusCode::OK, None, &document))
    }

    async fn delete_cookbook(&self, ctx: &Ctx, name: &str) -> Result<Response, ContentError> {
        if ctx.kind() == RepositoryType::Proxy {
            return self.delete_path(ctx, &remote::cookbook_path(name)).await;
        }
        let stored = hosted::stored(ctx, Some(name)).await?;
        if stored.is_empty() {
            return Err(missing_cookbook(name));
        }
        for one in &stored {
            ctx.require(View::Delete, &one.asset.path)?;
        }
        let document = hosted::cookbook(name, &stored)
            .and_then(|found| found.render(&Urls::of(ctx), |_| true))
            .ok_or_else(|| missing_cookbook(name))?;
        for one in stored {
            store::delete_asset(&ctx.content, one.asset).await?;
        }
        Ok(docs::json(StatusCode::OK, None, &document))
    }

    async fn read_other(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        if request.is_directory() {
            return directory_page(ctx);
        }
        ctx.require(View::Read, path)?;
        let found = match ctx.kind() {
            RepositoryType::Group => {
                group::first(ctx, |member| async move {
                    store::find(member.db(), member.name(), path).await
                })
                .await?
            }
            _ => store::find(ctx.db(), ctx.name(), path)
                .await?
                .map(|found| (ctx.clone(), found)),
        };
        let (owner, found) = found.ok_or_else(|| ContentError::not_found(path.to_owned()))?;
        serve(
            &owner,
            &found,
            &request.method,
            &request.headers,
            Serve::default(),
        )
        .await
    }

    async fn dispatch(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        route: Route,
    ) -> Result<Response, ContentError> {
        let method = request.method.clone();
        let kind = ctx.kind();
        let ok = |value: Value| docs::json(StatusCode::OK, Some(&request), &value);
        match (&method, route) {
            (&Method::GET | &Method::HEAD, Route::Root) => directory_page(ctx),
            (&Method::GET | &Method::HEAD, Route::Universe) => self.universe(ctx, &request).await,
            (&Method::GET | &Method::HEAD, Route::Cookbooks) => {
                self.listing(ctx, &request, LIST_PATH).await
            }
            (&Method::GET | &Method::HEAD, Route::Search) => {
                self.listing(ctx, &request, SEARCH_PATH).await
            }
            (&Method::GET | &Method::HEAD, Route::Cookbook(name)) => {
                Ok(ok(self.cookbook_document(ctx, &name).await?))
            }
            (&Method::GET | &Method::HEAD, Route::Version(name, wanted)) => {
                Ok(ok(self.version_document(ctx, &name, &wanted).await?))
            }
            (&Method::GET | &Method::HEAD, Route::Download(name, wanted)) => {
                self.download(ctx, &request, &name, &wanted).await
            }
            (&Method::GET | &Method::HEAD, Route::Tarball(name, version)) => {
                self.download(ctx, &request, &name, &Wanted::Exact(version))
                    .await
            }
            (&Method::GET | &Method::HEAD, Route::Other) => self.read_other(ctx, &request).await,
            (&Method::POST, Route::Cookbooks) => self.publish(ctx, request).await,
            (&Method::DELETE, _) if kind == RepositoryType::Group => {
                Err(not_allowed(&method, kind))
            }
            (&Method::DELETE, Route::Cookbook(name)) => self.delete_cookbook(ctx, &name).await,
            (&Method::DELETE, Route::Version(name, Wanted::Exact(version))) => {
                self.delete_version(ctx, &name, &version).await
            }
            (
                &Method::DELETE,
                Route::Tarball(name, version) | Route::Download(name, Wanted::Exact(version)),
            ) => {
                self.delete_path(ctx, &names::tarball_path(&name, &version))
                    .await
            }
            (&Method::DELETE, Route::Universe) if kind == RepositoryType::Proxy => {
                self.delete_path(ctx, remote::UNIVERSE_PATH).await
            }
            (&Method::DELETE, Route::Other) => self.delete_path(ctx, &request.path).await,
            (method, _) => Err(not_allowed(method, kind)),
        }
    }
}

#[async_trait]
impl Format for Chef {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        let api_route = route.is_api();
        let signed = request.headers.contains_key("x-ops-userid")
            || request.headers.contains_key("x-ops-sign");
        match self.dispatch(&ctx, request, route).await {
            Ok(response) => Ok(response),
            Err(ContentError::Unauthorized) if api_route => Ok(docs::error(
                ContentError::Unauthorized,
                Some(unauthorized_message(&ctx, signed)),
            )),
            Err(error) if api_route => Ok(docs::error(error, None)),
            Err(error) => Ok(error.into_response()),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match route(path) {
            Route::Tarball(name, version) => Ok(self
                .tarball(ctx, &name, &version)
                .await?
                .map(|(_, found)| found)),
            _ => match ctx.kind() {
                RepositoryType::Group => Ok(group::first(ctx, |member| async move {
                    store::find(member.db(), member.name(), path).await
                })
                .await?
                .map(|(_, found)| found)),
                _ => store::find(ctx.db(), ctx.name(), path).await,
            },
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
            .get(&format!("{ID}.asset"))
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest(format!("{ID}.asset is required")))?;
        hosted::may_add(&ctx)?;
        if file.size > MAX_TARBALL {
            return Err(ContentError::BadRequest(format!(
                "the cookbook archive is larger than {MAX_TARBALL} bytes"
            )));
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        hosted::adopt(&ctx, written, None).await.map(|_| ())
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        KEYWORDS
    }

    fn browse_layout(&self) -> astrum_content::BrowseLayout {
        astrum_content::BrowseLayout {
            path: None,
            generated: &["/universe"],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_supermarket_api() {
        assert_eq!(route("/"), Route::Root);
        assert_eq!(route("/universe"), Route::Universe);
        assert_eq!(route("/api/v1/cookbooks"), Route::Cookbooks);
        assert_eq!(route("/api/v1/cookbooks/"), Route::Cookbooks);
        assert_eq!(route("/api/v1/search"), Route::Search);
        assert_eq!(
            route("/api/v1/cookbooks/build-essential"),
            Route::Cookbook("build-essential".to_owned())
        );
        assert_eq!(
            route("/api/v1/cookbooks/apt/versions/7_4_0"),
            Route::Version("apt".to_owned(), Wanted::Exact("7.4.0".to_owned()))
        );
        assert_eq!(
            route("/api/v1/cookbooks/apt/versions/latest"),
            Route::Version("apt".to_owned(), Wanted::Latest)
        );
        assert_eq!(
            route("/api/v1/cookbooks/apt/versions/7.4.0/download"),
            Route::Download("apt".to_owned(), Wanted::Exact("7.4.0".to_owned()))
        );
        assert_eq!(
            route("/cookbooks/apt/7.4.0/apt-7.4.0.tar.gz"),
            Route::Tarball("apt".to_owned(), "7.4.0".to_owned())
        );
        for other in [
            "/api/v1/cookbooks/apt/versions/7.4.0.1",
            "/api/v1/cookbooks/apt/versions/newest",
            "/api/v1/cookbooks/a b",
            "/cookbooks/apt/7.4.0/other-7.4.0.tar.gz",
            "/cookbooks/apt/7.4.0/apt-7.4.0.tar.gz/",
            "/api/v2/cookbooks",
            "/api/v1/tools",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
        assert!(route("/universe").is_api());
        assert!(!route("/cookbooks/apt/7.4.0/apt-7.4.0.tar.gz").is_api());
    }
}
