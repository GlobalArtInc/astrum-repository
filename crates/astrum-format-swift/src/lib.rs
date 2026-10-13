mod archive;
mod paths;

use std::collections::{BTreeMap, BTreeSet};
use std::io;

use astrum_blob::ContentStream;
use astrum_content::format::SearchTarget;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, Component, ContentError, ContentRequest, Ctx, Format, SearchKey, Serve, View, group,
    serve,
};
use astrum_db::entity::{asset, component};
use astrum_repo::RepositoryType;
use astrum_repo::config::WritePolicy;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use futures_util::{StreamExt, TryStreamExt};
use sea_orm::sea_query::{Expr, Func};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Map, Value, json};

use paths::{Package, Route};

pub const ID: &str = "swift";
pub const ARCHIVE: &str = "ARCHIVE";
pub const MANIFEST_KIND: &str = "MANIFEST";
const API_VERSION: &str = "1";
const MAX_PART: usize = 1024 * 1024;
const MAX_ARCHIVE: u64 = 512 * 1024 * 1024;

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "swift.description",
        target: SearchTarget::Component,
        pointer: &["swift", "description"],
    },
    SearchKey {
        param: "swift.repositoryURL",
        target: SearchTarget::Component,
        pointer: &["swift", "repositoryURLs"],
    },
];

#[derive(Default)]
pub struct Swift;

#[derive(Debug)]
pub struct Problem {
    status: StatusCode,
    detail: String,
}

impl Problem {
    fn new(status: StatusCode, detail: impl Into<String>) -> Self {
        Self {
            status,
            detail: detail.into(),
        }
    }

    fn not_found(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, detail)
    }

    fn unprocessable(detail: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, detail)
    }

    fn reply(self) -> Response {
        let title = self.status.canonical_reason().unwrap_or("Error");
        let body = json!({"status": self.status.as_u16(), "title": title, "detail": self.detail});
        let mut response = (self.status, body.to_string()).into_response();
        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/problem+json"),
        );
        headers.insert("content-version", HeaderValue::from_static(API_VERSION));
        headers.insert(header::CONTENT_LANGUAGE, HeaderValue::from_static("en"));
        response
    }
}

impl From<ContentError> for Problem {
    fn from(error: ContentError) -> Self {
        match error {
            ContentError::Internal(error) => {
                tracing::error!(
                    error = format!("{error:#}"),
                    "a Swift registry request failed"
                );
                Problem::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "the server failed; see its log",
                )
            }
            other => Problem::new(other.status(), other.to_string()),
        }
    }
}

impl From<astrum_blob::Error> for Problem {
    fn from(error: astrum_blob::Error) -> Self {
        Problem::from(ContentError::from(error))
    }
}

impl From<sea_orm::DbErr> for Problem {
    fn from(error: sea_orm::DbErr) -> Self {
        Problem::from(ContentError::from(error))
    }
}

type Answer = Result<Response, Problem>;

fn check_accept(request: &ContentRequest) -> Result<(), Problem> {
    let Some(accept) = request.header("accept") else {
        return Ok(());
    };
    let lower = accept.to_ascii_lowercase();
    let Some(at) = lower.find("application/vnd.swift.registry") else {
        return Ok(());
    };
    let rest = &lower[at + "application/vnd.swift.registry".len()..];
    let Some(version) = rest.strip_prefix(".v") else {
        return Ok(());
    };
    let version: String = version
        .chars()
        .take_while(|c| !matches!(c, '+' | ';' | ',' | ' '))
        .collect();
    if version == API_VERSION {
        return Ok(());
    }
    match !version.is_empty() && version.bytes().all(|c| c.is_ascii_digit()) {
        true => Err(Problem::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported API version",
        )),
        false => Err(Problem::new(StatusCode::BAD_REQUEST, "invalid API version")),
    }
}

fn with_version(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert("content-version", HeaderValue::from_static(API_VERSION));
    response
}

fn json_answer(
    request: &ContentRequest,
    status: StatusCode,
    value: &Value,
    links: &[String],
) -> Response {
    let body = value.to_string();
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .header("content-version", API_VERSION)
        .header(header::CONTENT_LENGTH, body.len());
    if !links.is_empty() {
        builder = builder.header(header::LINK, links.join(", "));
    }
    builder
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn link(url: &str, rel: &str) -> String {
    format!("<{url}>; rel=\"{rel}\"")
}

fn sort_descending(versions: &mut [String]) {
    versions.sort_by(
        |a, b| match (semver::Version::parse(a), semver::Version::parse(b)) {
            (Ok(a), Ok(b)) => b.cmp(&a),
            _ => b.cmp(a),
        },
    );
}

fn base(ctx: &Ctx) -> String {
    ctx.repository_url()
}

fn attributes(component: &Component) -> &Value {
    &component.attributes[ID]
}

fn write_policy(ctx: &Ctx) -> WritePolicy {
    ctx.repository
        .attributes
        .storage
        .as_ref()
        .and_then(|storage| storage.write_policy)
        .unwrap_or(WritePolicy::Allow)
}

async fn releases_of(ctx: &Ctx, package: &Package) -> Result<Vec<Component>, Problem> {
    let mut found: Vec<Component> = component::Entity::find()
        .filter(component::Column::Repository.eq(ctx.name()))
        .filter(component::Column::Format.eq(ID))
        .filter(
            Expr::expr(Func::lower(Expr::col(component::Column::Namespace)))
                .eq(package.scope.to_lowercase()),
        )
        .filter(
            Expr::expr(Func::lower(Expr::col(component::Column::Name)))
                .eq(package.name.to_lowercase()),
        )
        .all(ctx.db())
        .await?
        .into_iter()
        .filter(|found| attributes(found)["checksum"].is_string())
        .collect();
    found.sort_by_key(|found| found.id);
    Ok(found)
}

fn canonical(package: &Package, releases: &[Component]) -> Package {
    match releases.first() {
        Some(first) => Package {
            scope: first.namespace.clone(),
            name: first.name.clone(),
            version: package.version.clone(),
        },
        None => package.clone(),
    }
}

fn release_document(package: &Package, component: &Component) -> Value {
    let swift = attributes(component);
    let mut archive = json!({"name": "source-archive", "type": "application/zip"});
    if let Some(checksum) = swift["checksum"].as_str() {
        archive["checksum"] = Value::String(checksum.to_owned());
    }
    if swift["signing"].is_object() {
        archive["signing"] = swift["signing"].clone();
    }
    let mut document = json!({
        "id": package.identifier(),
        "version": component.version,
        "resources": [archive],
        "metadata": if swift["metadata"].is_object() { swift["metadata"].clone() } else { json!({}) },
    });
    if let Some(published) = swift["publishedAt"].as_str() {
        document["publishedAt"] = Value::String(published.to_owned());
    }
    document
}

fn release_links(base: &str, package: &Package, versions: &[String]) -> Vec<String> {
    let mut sorted = versions.to_vec();
    sort_descending(&mut sorted);
    let mut links = Vec::new();
    let Some(latest) = sorted.first() else {
        return links;
    };
    links.push(link(
        &format!("{base}{}", package.at(latest).release_path()),
        "latest-version",
    ));
    if let Some(index) = sorted
        .iter()
        .position(|version| Some(version) == package.version.as_ref())
    {
        if index > 0 {
            links.push(link(
                &format!("{base}{}", package.at(&sorted[index - 1]).release_path()),
                "successor-version",
            ));
        }
        if index + 1 < sorted.len() {
            links.push(link(
                &format!("{base}{}", package.at(&sorted[index + 1]).release_path()),
                "predecessor-version",
            ));
        }
    }
    links
}

fn digest(sha256: &str) -> Option<String> {
    hex::decode(sha256)
        .ok()
        .filter(|bytes| bytes.len() == 32)
        .map(|bytes| format!("sha-256={}", STANDARD.encode(bytes)))
}

async fn find_release(
    ctx: &Ctx,
    package: &Package,
) -> Result<(Package, Component, Vec<String>), Problem> {
    let releases = releases_of(ctx, package).await?;
    let canonical = canonical(package, &releases);
    let version = package.version.clone().unwrap_or_default();
    let versions: Vec<String> = releases.iter().map(|found| found.version.clone()).collect();
    let found = releases
        .into_iter()
        .find(|found| found.version == version)
        .ok_or_else(|| {
            Problem::not_found(format!(
                "release {version} of {} not found",
                package.identifier()
            ))
        })?;
    Ok((canonical, found, versions))
}

impl Swift {
    async fn member_first<T, F, Fut>(&self, ctx: &Ctx, visit: F) -> Result<Option<T>, Problem>
    where
        F: Fn(Ctx) -> Fut,
        Fut: std::future::Future<Output = Result<T, Problem>>,
    {
        for member in group::members(ctx).await? {
            let member = ctx.member(member);
            match visit(member).await {
                Ok(found) => return Ok(Some(found)),
                Err(problem) if problem.status == StatusCode::NOT_FOUND => {}
                Err(problem) => {
                    tracing::info!(
                        group = ctx.name(),
                        detail = problem.detail,
                        "a group member failed; trying the next"
                    );
                }
            }
        }
        Ok(None)
    }

    async fn releases(&self, ctx: &Ctx, request: &ContentRequest, package: &Package) -> Answer {
        let mut merged: BTreeMap<String, ()> = BTreeMap::new();
        let mut canonical_package = None;
        let leaves = match ctx.kind() {
            RepositoryType::Group => group::members(ctx)
                .await?
                .into_iter()
                .map(|member| ctx.member(member))
                .collect(),
            _ => vec![ctx.clone()],
        };
        for leaf in leaves {
            let releases = releases_of(&leaf, package).await?;
            if releases.is_empty() {
                continue;
            }
            canonical_package.get_or_insert_with(|| canonical(package, &releases));
            for found in releases {
                merged.insert(found.version, ());
            }
        }
        let Some(canonical) = canonical_package else {
            return Err(Problem::not_found(format!(
                "package {} not found",
                package.identifier()
            )));
        };
        let base = base(ctx);
        let mut versions: Vec<String> = merged.into_keys().collect();
        sort_descending(&mut versions);
        let mut releases = Map::new();
        for version in &versions {
            releases.insert(
                version.clone(),
                json!({"url": format!("{base}{}", canonical.at(version).release_path())}),
            );
        }
        let links: Vec<String> = versions
            .first()
            .map(|latest| {
                link(
                    &format!("{base}{}", canonical.at(latest).release_path()),
                    "latest-version",
                )
            })
            .into_iter()
            .collect();
        Ok(json_answer(
            request,
            StatusCode::OK,
            &json!({"releases": releases}),
            &links,
        ))
    }

    async fn release(&self, ctx: &Ctx, request: &ContentRequest, package: &Package) -> Answer {
        let (canonical, found, versions) = match ctx.kind() {
            RepositoryType::Group => self
                .member_first(
                    ctx,
                    |member| async move { find_release(&member, package).await },
                )
                .await?
                .ok_or_else(|| {
                    Problem::not_found(format!(
                        "release {} of {} not found",
                        package.version.clone().unwrap_or_default(),
                        package.identifier()
                    ))
                })?,
            _ => find_release(ctx, package).await?,
        };
        let document = release_document(&canonical, &found);
        let links = release_links(&base(ctx), &canonical, &versions);
        Ok(json_answer(request, StatusCode::OK, &document, &links))
    }

    async fn leaf_archive(
        ctx: &Ctx,
        package: &Package,
    ) -> Result<(Package, Component, Asset), Problem> {
        let (canonical, found, _) = find_release(ctx, package).await?;
        let asset = store::find(ctx.db(), ctx.name(), &canonical.archive_path())
            .await?
            .ok_or_else(|| {
                Problem::not_found(format!(
                    "source archive of {} not found",
                    canonical.identifier()
                ))
            })?;
        Ok((canonical, found, asset))
    }

    async fn archive(&self, ctx: &Ctx, request: &ContentRequest, package: &Package) -> Answer {
        let (canonical, found, asset, owner) = match ctx.kind() {
            RepositoryType::Group => self
                .member_first(ctx, |member| async move {
                    Self::leaf_archive(&member, package)
                        .await
                        .map(|(canonical, found, asset)| (canonical, found, asset, member))
                })
                .await?
                .ok_or_else(|| {
                    Problem::not_found(format!(
                        "source archive of {} not found",
                        package.identifier()
                    ))
                })?,
            _ => {
                let (canonical, found, asset) = Self::leaf_archive(ctx, package).await?;
                (canonical, found, asset, ctx.clone())
            }
        };
        let file = format!(
            "attachment; filename=\"{}-{}.zip\"",
            canonical.name,
            canonical.version.clone().unwrap_or_default()
        );
        let mut response = serve(
            &owner,
            &asset,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await?;
        let headers = response.headers_mut();
        let swift = attributes(&found);
        let mut extra: Vec<(&str, String)> = vec![
            ("content-disposition", file),
            ("cache-control", "public, immutable".to_owned()),
            ("content-version", API_VERSION.to_owned()),
        ];
        if let Some(digest) = swift["checksum"].as_str().and_then(digest) {
            extra.push(("digest", digest));
        }
        if let (Some(format), Some(signature)) = (
            swift["signing"]["signatureFormat"].as_str(),
            swift["signing"]["signatureBase64Encoded"].as_str(),
        ) {
            extra.push(("x-swift-package-signature-format", format.to_owned()));
            extra.push(("x-swift-package-signature", signature.to_owned()));
        }
        for (name, value) in extra {
            if let Ok(value) = HeaderValue::from_str(&value) {
                headers.insert(name, value);
            }
        }
        Ok(response)
    }

    async fn leaf_manifest(
        ctx: &Ctx,
        package: &Package,
        file: &str,
        swift_version: Option<&str>,
    ) -> Result<Result<(Package, Asset, BTreeMap<String, String>), String>, Problem> {
        let (canonical, _, _) = find_release(ctx, package).await?;
        let unqualified = store::find(
            ctx.db(),
            ctx.name(),
            &canonical.manifest_path(archive::MANIFEST),
        )
        .await?
        .ok_or_else(|| {
            Problem::not_found(format!("manifest of {} not found", canonical.identifier()))
        })?;
        let alternates: BTreeMap<String, String> = unqualified.attributes[ID]["alternates"]
            .as_object()
            .map(|found| {
                found
                    .iter()
                    .map(|(file, version)| {
                        (
                            file.clone(),
                            version.as_str().unwrap_or_default().to_owned(),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        let mut wanted = file.to_owned();
        if file == archive::MANIFEST
            && let Some(swift) = swift_version
        {
            match archive::resolve(alternates.keys(), swift) {
                Some(resolved) => wanted = resolved,
                None => return Ok(Err(canonical.manifest_path(archive::MANIFEST))),
            }
        }
        let asset = match wanted == archive::MANIFEST {
            true => unqualified,
            false => store::find(ctx.db(), ctx.name(), &canonical.manifest_path(&wanted))
                .await?
                .ok_or_else(|| {
                    Problem::not_found(format!(
                        "manifest {file} of {} not found",
                        canonical.identifier()
                    ))
                })?,
        };
        Ok(Ok((canonical, asset, alternates)))
    }

    async fn manifest(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        package: &Package,
        file: &str,
    ) -> Answer {
        let swift_version = request.query.as_deref().and_then(|query| {
            url::form_urlencoded::parse(query.as_bytes())
                .find(|(name, _)| name == "swift-version")
                .map(|(_, value)| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        });
        let found = match ctx.kind() {
            RepositoryType::Group => self
                .member_first(ctx, |member| {
                    let swift_version = swift_version.clone();
                    async move {
                        Self::leaf_manifest(&member, package, file, swift_version.as_deref())
                            .await
                            .map(|found| {
                                found.map(|(canonical, asset, alternates)| {
                                    (canonical, asset, alternates, member)
                                })
                            })
                    }
                })
                .await?
                .ok_or_else(|| {
                    Problem::not_found(format!("manifest of {} not found", package.identifier()))
                })?,
            _ => Self::leaf_manifest(ctx, package, file, swift_version.as_deref())
                .await?
                .map(|(canonical, asset, alternates)| (canonical, asset, alternates, ctx.clone())),
        };
        let (canonical, asset, alternates, owner) = match found {
            Ok(found) => found,
            Err(unqualified) => {
                let response = Response::builder()
                    .status(StatusCode::SEE_OTHER)
                    .header(header::LOCATION, format!("{}{unqualified}", base(ctx)))
                    .header("content-version", API_VERSION)
                    .body(Body::empty())
                    .map_err(|error| Problem::from(ContentError::internal(error)))?;
                return Ok(response);
            }
        };
        let filename = asset
            .path
            .rsplit('/')
            .next()
            .unwrap_or(archive::MANIFEST)
            .to_owned();
        let mut response = serve(
            &owner,
            &asset,
            &request.method,
            &request.headers,
            Serve {
                content_type: Some("text/x-swift".to_owned()),
                ..Serve::download()
            },
        )
        .await?;
        let headers = response.headers_mut();
        if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
            headers.insert(header::CONTENT_DISPOSITION, value);
        }
        headers.insert(
            header::CACHE_CONTROL,
            HeaderValue::from_static("public, immutable"),
        );
        if filename == archive::MANIFEST && !alternates.is_empty() {
            let base = base(ctx);
            let links: Vec<String> = alternates
                .iter()
                .filter_map(|(file, tools)| {
                    let swift = archive::variant_version(file)?;
                    let tools = if tools.is_empty() { swift } else { tools.as_str() };
                    Some(format!(
                        "<{base}{}?swift-version={swift}>; rel=\"alternate\"; filename=\"{file}\"; swift-tools-version=\"{tools}\"",
                        canonical.manifest_path(archive::MANIFEST)
                    ))
                })
                .collect();
            if let Ok(value) = HeaderValue::from_str(&links.join(", ")) {
                headers.insert(header::LINK, value);
            }
        }
        Ok(with_version(response))
    }

    async fn identifiers(&self, ctx: &Ctx, request: &ContentRequest) -> Answer {
        let url = request
            .query
            .as_deref()
            .and_then(|query| {
                url::form_urlencoded::parse(query.as_bytes())
                    .find(|(name, _)| name == "url")
                    .map(|(_, value)| value.trim().to_owned())
            })
            .filter(|url| !url.is_empty())
            .ok_or_else(|| {
                Problem::new(
                    StatusCode::BAD_REQUEST,
                    "the url query parameter is required",
                )
            })?;
        let wanted = normalize_url(&url);
        let leaves = match ctx.kind() {
            RepositoryType::Group => group::members(ctx)
                .await?
                .into_iter()
                .map(|member| ctx.member(member))
                .collect(),
            _ => vec![ctx.clone()],
        };
        let mut found = BTreeSet::new();
        for leaf in leaves {
            let components = component::Entity::find()
                .filter(component::Column::Repository.eq(leaf.name()))
                .filter(component::Column::Format.eq(ID))
                .filter(Expr::cust(
                    "jsonb_typeof(attributes->'swift'->'repositoryURLs') = 'array'",
                ))
                .all(leaf.db())
                .await?;
            for component in components {
                let matches = attributes(&component)["repositoryURLs"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                    .any(|candidate| normalize_url(candidate) == wanted);
                if matches {
                    found.insert(format!("{}.{}", component.namespace, component.name));
                }
            }
        }
        if found.is_empty() {
            return Err(Problem::not_found(format!(
                "no package identifiers are registered for {url}"
            )));
        }
        Ok(json_answer(
            request,
            StatusCode::OK,
            &json!({"identifiers": found}),
            &[],
        ))
    }

    async fn publish(&self, ctx: &Ctx, request: ContentRequest, package: Package) -> Answer {
        let target = match ctx.kind() {
            RepositoryType::Hosted => ctx.clone(),
            RepositoryType::Group => match group::writable_member(ctx).await? {
                Some(member) => {
                    let member = ctx.member(member);
                    ctx.require(View::Add, &package.archive_path())?;
                    member
                }
                None => {
                    return Err(Problem::new(
                        StatusCode::METHOD_NOT_ALLOWED,
                        "publishing isn't supported: the group has no writable member",
                    ));
                }
            },
            RepositoryType::Proxy => {
                return Err(Problem::new(
                    StatusCode::METHOD_NOT_ALLOWED,
                    "publishing isn't supported: proxy repositories are read-only",
                ));
            }
        };
        if write_policy(&target) == WritePolicy::Deny {
            return Err(Problem::new(
                StatusCode::METHOD_NOT_ALLOWED,
                format!(
                    "publishing isn't supported: repository {} is read-only",
                    target.name()
                ),
            ));
        }
        let existing = releases_of(&target, &package).await?;
        let canonical = canonical(&package, &existing);
        if ctx.kind() == RepositoryType::Hosted {
            ctx.require(View::Add, &canonical.archive_path())?;
        }
        let version = package.version.clone().unwrap_or_default();
        if existing.iter().any(|found| found.version == version)
            && write_policy(&target) != WritePolicy::Allow
        {
            return Err(Problem::new(
                StatusCode::CONFLICT,
                format!("a release with version {version} already exists"),
            ));
        }
        let content_type = request
            .header("content-type")
            .unwrap_or_default()
            .to_owned();
        let signature_format = request
            .header("x-swift-package-signature-format")
            .map(str::to_owned)
            .filter(|value| !value.trim().is_empty());
        let boundary = multer::parse_boundary(&content_type).map_err(|_| {
            Problem::new(
                StatusCode::BAD_REQUEST,
                "a package release must be published as multipart/form-data",
            )
        })?;
        let mut multipart =
            multer::Multipart::new(request.take_body().into_data_stream(), boundary);
        let mut written = None;
        let mut archive_signature = None;
        let mut metadata = None;
        let mut metadata_signature = None;
        let failed = |written: &Option<astrum_blob::Written>, problem: Problem| {
            if let Some(written) = written {
                store::discard(target.content.clone(), &written.store, written.id.uuid());
            }
            problem
        };
        loop {
            let field = match multipart.next_field().await {
                Ok(Some(field)) => field,
                Ok(None) => break,
                Err(error) => {
                    return Err(failed(
                        &written,
                        Problem::new(
                            StatusCode::BAD_REQUEST,
                            format!("the form cannot be read: {error}"),
                        ),
                    ));
                }
            };
            let base64 = field
                .headers()
                .get("content-transfer-encoding")
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.trim().eq_ignore_ascii_case("base64"));
            let name = field.name().unwrap_or_default().to_owned();
            match name.as_str() {
                "source-archive" => {
                    if written.is_some() {
                        return Err(failed(
                            &written,
                            Problem::new(
                                StatusCode::BAD_REQUEST,
                                "more than one source-archive section",
                            ),
                        ));
                    }
                    let stream: ContentStream = if base64 {
                        let bytes = field.bytes().await.map_err(|error| {
                            Problem::new(StatusCode::BAD_REQUEST, error.to_string())
                        })?;
                        let cleaned: Vec<u8> = bytes
                            .iter()
                            .copied()
                            .filter(|c| !c.is_ascii_whitespace())
                            .collect();
                        let decoded = STANDARD.decode(cleaned).map_err(|_| {
                            Problem::new(
                                StatusCode::BAD_REQUEST,
                                "the source archive is not valid base64",
                            )
                        })?;
                        astrum_blob::content_of(decoded)
                    } else {
                        field.map_err(io::Error::other).boxed()
                    };
                    written = Some(store::write_blob(&target, stream).await?);
                }
                "source-archive-signature" | "metadata" | "metadata-signature" => {
                    let bytes = field.bytes().await.map_err(|error| {
                        failed(
                            &written,
                            Problem::new(StatusCode::BAD_REQUEST, error.to_string()),
                        )
                    })?;
                    if bytes.len() > MAX_PART {
                        return Err(failed(
                            &written,
                            Problem::new(
                                StatusCode::PAYLOAD_TOO_LARGE,
                                format!("the {name} section is larger than {MAX_PART} bytes"),
                            ),
                        ));
                    }
                    let bytes = match base64 {
                        true => STANDARD
                            .decode(
                                bytes
                                    .iter()
                                    .copied()
                                    .filter(|c| !c.is_ascii_whitespace())
                                    .collect::<Vec<u8>>(),
                            )
                            .map_err(|_| {
                                failed(
                                    &written,
                                    Problem::new(
                                        StatusCode::BAD_REQUEST,
                                        format!("the {name} section is not valid base64"),
                                    ),
                                )
                            })?,
                        false => bytes.to_vec(),
                    };
                    match name.as_str() {
                        "source-archive-signature" => archive_signature = Some(bytes),
                        "metadata" => metadata = Some(bytes),
                        _ => metadata_signature = Some(bytes),
                    }
                }
                _ => {
                    let _ = field.bytes().await;
                }
            }
        }
        let Some(written) = written else {
            return Err(Problem::unprocessable(
                "the request has no source-archive section",
            ));
        };
        let present =
            |value: &Option<Vec<u8>>| value.as_ref().is_some_and(|bytes| !bytes.is_empty());
        let checks = || -> Result<Value, Problem> {
            if (present(&archive_signature) || present(&metadata_signature))
                && signature_format.is_none()
            {
                return Err(Problem::new(
                    StatusCode::BAD_REQUEST,
                    "a signed release needs the X-Swift-Package-Signature-Format header",
                ));
            }
            if present(&metadata_signature) && !present(&archive_signature) {
                return Err(Problem::unprocessable(
                    "the metadata is signed but the source archive is not",
                ));
            }
            parse_metadata(metadata.as_deref())
        };
        let metadata = match checks() {
            Ok(metadata) => metadata,
            Err(problem) => return Err(failed(&Some(written), problem)),
        };
        let data = match read_written(&target, &written).await {
            Ok(data) => data,
            Err(problem) => return Err(failed(&Some(written), problem)),
        };
        let source = match tokio::task::spawn_blocking(move || archive::read(&data))
            .await
            .map_err(|error| Problem::from(ContentError::internal(error)))?
        {
            Ok(source) => source,
            Err(message) => return Err(failed(&Some(written), Problem::unprocessable(message))),
        };
        let package = Package {
            version: Some(version.clone()),
            ..canonical
        };
        let signing = |signature: &Option<Vec<u8>>| {
            signature
                .as_ref()
                .filter(|bytes| !bytes.is_empty())
                .map(|bytes| {
                    json!({
                        "signatureBase64Encoded": STANDARD.encode(bytes),
                        "signatureFormat": signature_format.clone().unwrap_or_default(),
                    })
                })
        };
        let mut swift = json!({
            "checksum": written.digests.sha256,
            "publishedAt": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
            "metadata": metadata,
        });
        if let Some(description) = metadata["description"].as_str() {
            swift["description"] = Value::String(description.to_owned());
        }
        let urls: Vec<String> = metadata["repositoryURLs"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(|url| url.trim().to_owned())
            .filter(|url| !url.is_empty())
            .collect();
        if !urls.is_empty() {
            swift["repositoryURLs"] = json!(urls);
        }
        if let Some(signing) = signing(&archive_signature) {
            swift["signing"] = signing;
        }
        if let Some(signing) = signing(&metadata_signature) {
            swift["metadataSigning"] = signing;
        }
        let key = ComponentKey {
            namespace: package.scope.clone(),
            name: package.name.clone(),
            version: version.clone(),
            kind: String::new(),
            attributes: json!({ ID: swift }),
        };
        let mut put = PutAsset::hosted(&target, package.archive_path())
            .component(key.clone())
            .kind(ARCHIVE)
            .content_type("application/zip")
            .attributes(json!({}));
        put.write_policy = Some(WritePolicy::Allow);
        store::adopt(&target, put, &written).await?;
        let tools = source.tools_versions();
        let alternates: Map<String, Value> = tools
            .iter()
            .filter(|(file, _)| file.as_str() != archive::MANIFEST)
            .map(|(file, version)| (file.clone(), Value::String(version.clone())))
            .collect();
        let mut kept = BTreeSet::new();
        for (file, text) in &source.manifests {
            let path = package.manifest_path(file);
            kept.insert(path.clone());
            let mut attributes =
                json!({"toolsVersion": tools.get(file).cloned().unwrap_or_default()});
            if file == archive::MANIFEST {
                attributes["alternates"] = Value::Object(alternates.clone());
            }
            let mut put = PutAsset::hosted(&target, path)
                .component(key.clone())
                .kind(MANIFEST_KIND)
                .content_type("text/x-swift")
                .attributes(json!({ ID: attributes }));
            put.write_policy = Some(WritePolicy::Allow);
            put.check_content_type = false;
            store::put(
                &target,
                put,
                astrum_blob::content_of(text.clone().into_bytes()),
            )
            .await?;
        }
        let stale = asset::Entity::find()
            .filter(asset::Column::Repository.eq(target.name()))
            .filter(asset::Column::Kind.eq(MANIFEST_KIND))
            .filter(asset::Column::Path.starts_with(format!("{}/", package.release_path())))
            .all(target.db())
            .await?;
        for found in stale {
            if !kept.contains(&found.path) {
                store::delete_asset(&target.content, found).await?;
            }
        }
        tracing::info!(
            repository = target.name(),
            package = package.identifier(),
            version,
            "published a Swift package release"
        );
        let response = Response::builder()
            .status(StatusCode::CREATED)
            .header("content-version", API_VERSION)
            .header(
                header::LOCATION,
                format!("{}{}", base(ctx), package.release_path()),
            )
            .body(Body::empty())
            .map_err(|error| Problem::from(ContentError::internal(error)))?;
        Ok(response)
    }

    async fn answer(&self, ctx: &Ctx, request: ContentRequest) -> Answer {
        check_accept(&request)?;
        let route = paths::route(&request.path);
        match (&request.method, route) {
            (&Method::POST, Route::Login) => match &ctx.subject {
                Some(subject) if !subject.anonymous => {
                    Ok(with_version(StatusCode::OK.into_response()))
                }
                _ => Err(Problem::from(ContentError::Unauthorized)),
            },
            (&Method::GET | &Method::HEAD, Route::Availability) => {
                Ok(with_version(StatusCode::OK.into_response()))
            }
            (&Method::GET | &Method::HEAD, Route::Identifiers) => {
                ctx.require(View::Read, &request.path)?;
                self.identifiers(ctx, &request).await
            }
            (&Method::GET | &Method::HEAD, Route::Releases(package)) => {
                ctx.require(View::Read, &package.package_path())?;
                self.releases(ctx, &request, &package).await
            }
            (&Method::GET | &Method::HEAD, Route::Release(package)) => {
                ctx.require(View::Read, &package.release_path())?;
                self.release(ctx, &request, &package).await
            }
            (&Method::GET | &Method::HEAD, Route::Archive(package)) => {
                ctx.require(View::Read, &package.archive_path())?;
                self.archive(ctx, &request, &package).await
            }
            (&Method::GET | &Method::HEAD, Route::Manifest(package, file)) => {
                ctx.require(View::Read, &package.manifest_path(&file))?;
                self.manifest(ctx, &request, &package, &file).await
            }
            (&Method::PUT, Route::Release(package)) if !request.path.ends_with(".json") => {
                self.publish(ctx, request, package).await
            }
            (&Method::DELETE, _) if ctx.kind() != RepositoryType::Group => {
                ctx.require(View::Delete, &request.path)?;
                match store::delete(ctx, &request.path).await? {
                    true => Ok(with_version(StatusCode::NO_CONTENT.into_response())),
                    false => Err(Problem::not_found(format!("{} not found", request.path))),
                }
            }
            (_, Route::Other) => Err(Problem::not_found(format!(
                "{} is not a Swift registry path",
                request.path
            ))),
            (method, _) => Err(Problem::new(
                StatusCode::METHOD_NOT_ALLOWED,
                format!("{method} is not allowed here"),
            )),
        }
    }
}

fn normalize_url(url: &str) -> String {
    let lower = url.trim().to_lowercase();
    let lower = lower.strip_suffix('/').unwrap_or(&lower);
    lower.strip_suffix(".git").unwrap_or(lower).to_owned()
}

fn parse_metadata(bytes: Option<&[u8]>) -> Result<Value, Problem> {
    let Some(bytes) = bytes.filter(|bytes| !bytes.is_empty()) else {
        return Ok(json!({}));
    };
    let invalid = |detail: &str| {
        Problem::unprocessable(format!(
            "invalid JSON provided for release metadata: {detail}"
        ))
    };
    let value: Value = serde_json::from_slice(bytes)
        .map_err(|_| Problem::unprocessable("invalid JSON provided for release metadata"))?;
    let Some(object) = value.as_object() else {
        return Err(invalid("expected an object"));
    };
    for field in [
        "description",
        "licenseURL",
        "readmeURL",
        "originalPublicationTime",
    ] {
        if object
            .get(field)
            .is_some_and(|value| !value.is_null() && !value.is_string())
        {
            return Err(invalid(&format!("{field} must be a string")));
        }
    }
    if let Some(time) = object
        .get("originalPublicationTime")
        .and_then(Value::as_str)
        && chrono::DateTime::parse_from_rfc3339(time).is_err()
    {
        return Err(invalid(
            "originalPublicationTime must be an ISO 8601 date-time",
        ));
    }
    if let Some(urls) = object
        .get("repositoryURLs")
        .filter(|value| !value.is_null())
        && !urls
            .as_array()
            .is_some_and(|urls| urls.iter().all(Value::is_string))
    {
        return Err(invalid("repositoryURLs must be an array of strings"));
    }
    if let Some(author) = object.get("author").filter(|value| !value.is_null()) {
        let named = author["name"]
            .as_str()
            .is_some_and(|name| !name.trim().is_empty());
        if !author.is_object() || !named {
            return Err(invalid("author.name is required"));
        }
    }
    Ok(value)
}

async fn read_written(ctx: &Ctx, written: &astrum_blob::Written) -> Result<Vec<u8>, Problem> {
    if written.size > MAX_ARCHIVE {
        return Err(Problem::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "the source archive is too large",
        ));
    }
    let store = ctx.content.blob_stores().get(&written.store).await?;
    let content = store
        .read(written.id)
        .await
        .map_err(ContentError::from)?
        .ok_or_else(|| {
            Problem::from(ContentError::internal(io::Error::other(
                "a blob that was just written is missing",
            )))
        })?;
    let mut data = Vec::with_capacity(written.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        data.extend_from_slice(&chunk);
    }
    Ok(data)
}

#[async_trait]
impl Format for Swift {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() == RepositoryType::Proxy {
            return Err(ContentError::Unavailable(
                "Swift proxy repositories are not served by this server yet".to_owned(),
            ));
        }
        match self.answer(&ctx, request).await {
            Ok(response) => Ok(response),
            Err(problem) if problem.status == StatusCode::UNAUTHORIZED => {
                Err(ContentError::Unauthorized)
            }
            Err(problem) => Ok(problem.reply()),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => Ok(group::first(ctx, |member| async move {
                store::find(member.db(), member.name(), path).await
            })
            .await?
            .map(|(_, asset)| asset)),
            _ => store::find(ctx.db(), ctx.name(), path).await,
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
    fn metadata_is_checked_like_the_registry_specification_asks() {
        assert_eq!(parse_metadata(None).unwrap(), json!({}));
        assert!(parse_metadata(Some(b"{\"description\": 1}")).is_err());
        assert!(parse_metadata(Some(b"[]")).is_err());
        assert!(parse_metadata(Some(b"{\"author\": {}}")).is_err());
        assert!(parse_metadata(Some(b"{\"originalPublicationTime\": \"yesterday\"}")).is_err());
        let fine = br#"{"description":"A list","repositoryURLs":["https://github.com/acme/List.git"],"author":{"name":"Jane"}}"#;
        assert_eq!(parse_metadata(Some(fine)).unwrap()["description"], "A list");
    }

    #[test]
    fn versions_sort_newest_first_and_digests_are_base64() {
        let mut versions = vec![
            "1.0.0".to_owned(),
            "1.10.0".to_owned(),
            "1.2.0-beta".to_owned(),
            "1.2.0".to_owned(),
        ];
        sort_descending(&mut versions);
        assert_eq!(versions, ["1.10.0", "1.2.0", "1.2.0-beta", "1.0.0"]);
        assert_eq!(
            digest(&"00".repeat(32)).as_deref(),
            Some("sha-256=AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
        );
        assert_eq!(
            normalize_url("https://GitHub.com/acme/List.git/"),
            "https://github.com/acme/list"
        );
    }
}
