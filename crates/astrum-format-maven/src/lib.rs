mod hosted;
pub mod metadata;
pub mod path;
pub mod pom;
mod respond;
mod upload;
pub mod version;
mod xml;

use astrum_content::format::SearchTarget;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Freshness, SearchKey, Serve, UploadForm,
    UploadSpec, View, group, proxy, serve,
};
use astrum_repo::RepositoryType;
use astrum_repo::config::{ContentDisposition, LayoutPolicy, VersionPolicy, WritePolicy};
use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, Utc};
use serde_json::json;

use crate::path::{Coordinates, Kind, MavenPath};
use crate::respond::{Document, Found};

pub const ID: &str = "maven2";
pub(crate) const DOCUMENT_LIMIT: usize = 16 * 1024 * 1024;
const HOSTED_ALLOW: &str = "GET,HEAD,PUT,DELETE";
const READ_ALLOW: &str = "GET,HEAD";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "maven.groupId",
        target: SearchTarget::Asset,
        pointer: &[ID, "groupId"],
    },
    SearchKey {
        param: "maven.artifactId",
        target: SearchTarget::Asset,
        pointer: &[ID, "artifactId"],
    },
    SearchKey {
        param: "maven.baseVersion",
        target: SearchTarget::Asset,
        pointer: &[ID, "baseVersion"],
    },
    SearchKey {
        param: "maven.classifier",
        target: SearchTarget::Asset,
        pointer: &[ID, "classifier"],
    },
    SearchKey {
        param: "maven.extension",
        target: SearchTarget::Asset,
        pointer: &[ID, "extension"],
    },
];

pub struct Maven;

pub(crate) fn component_key(coordinates: &Coordinates) -> ComponentKey {
    ComponentKey {
        namespace: coordinates.group_id.clone(),
        name: coordinates.artifact_id.clone(),
        version: coordinates.version.clone(),
        kind: String::new(),
        attributes: json!({ ID: {
            "groupId": coordinates.group_id,
            "artifactId": coordinates.artifact_id,
            "version": coordinates.version,
            "baseVersion": coordinates.base_version,
        }}),
    }
}

pub(crate) fn describe(parsed: &MavenPath, put: PutAsset) -> PutAsset {
    let put = put.kind(parsed.kind.asset_kind());
    match &parsed.kind {
        Kind::Artifact(coordinates) | Kind::Signature(coordinates) => {
            let put = put
                .component(component_key(coordinates))
                .attributes(coordinates.attributes());
            match coordinates.extension.as_str() {
                "pom" => put.content_type(respond::XML),
                "module" => put.content_type(respond::JSON),
                _ => put,
            }
        }
        Kind::Metadata | Kind::ArchetypeCatalog => put.content_type(respond::XML),
        Kind::Index | Kind::Other => put,
    }
}

fn version_policy(ctx: &Ctx) -> Option<VersionPolicy> {
    ctx.repository
        .attributes
        .maven
        .as_ref()
        .and_then(|maven| maven.version_policy)
}

fn strict(ctx: &Ctx) -> bool {
    ctx.repository
        .attributes
        .maven
        .as_ref()
        .and_then(|maven| maven.layout_policy)
        .unwrap_or(LayoutPolicy::Strict)
        == LayoutPolicy::Strict
}

fn disposition(ctx: &Ctx) -> &'static str {
    match ctx
        .repository
        .attributes
        .maven
        .as_ref()
        .and_then(|maven| maven.content_disposition)
    {
        Some(ContentDisposition::Inline) => "inline",
        _ => "attachment",
    }
}

fn refused(ctx: &Ctx, parsed: &MavenPath) -> Option<String> {
    let policy = version_policy(ctx)?;
    let (snapshot, what) = match &parsed.kind {
        Kind::Artifact(coordinates) | Kind::Signature(coordinates) => (
            coordinates.is_snapshot(),
            format!("version: {}", coordinates.version),
        ),
        Kind::Metadata if parsed.in_snapshot_directory() => (
            true,
            format!("metadata in path: {}", parsed.path.trim_start_matches('/')),
        ),
        _ => return None,
    };
    let allowed = match policy {
        VersionPolicy::Release => !snapshot,
        VersionPolicy::Snapshot => snapshot,
        VersionPolicy::Mixed => true,
    };
    (!allowed).then(|| {
        let name = match policy {
            VersionPolicy::Release => "RELEASE",
            VersionPolicy::Snapshot => "SNAPSHOT",
            VersionPolicy::Mixed => "MIXED",
        };
        format!("Repository version policy: {name} does not allow {what}")
    })
}

fn freshness(kind: &Kind) -> Freshness {
    match kind {
        Kind::Metadata | Kind::ArchetypeCatalog | Kind::Index => Freshness::Metadata,
        Kind::Artifact(coordinates) | Kind::Signature(coordinates)
            if version::timestamped(&coordinates.version).is_none()
                && coordinates.is_snapshot() =>
        {
            Freshness::Metadata
        }
        _ => Freshness::Content,
    }
}

fn not_found(path: &str) -> ContentError {
    ContentError::not_found(path.trim_start_matches('/'))
}

fn not_allowed(method: &Method, allow: &'static str) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow,
    }
}

fn base_of(parsed: &MavenPath) -> MavenPath {
    MavenPath {
        path: parsed.base.clone(),
        hash: None,
        base: parsed.base.clone(),
        kind: parsed.kind.clone(),
    }
}

impl Maven {
    async fn find(&self, ctx: &Ctx, parsed: &MavenPath) -> Result<Option<Found>, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => self.find_in_group(ctx, parsed).await,
            _ => self.find_in_member(ctx, parsed).await,
        }
    }

    async fn find_in_member(
        &self,
        ctx: &Ctx,
        parsed: &MavenPath,
    ) -> Result<Option<Found>, ContentError> {
        if refused(ctx, parsed).is_some() {
            return Ok(None);
        }
        match ctx.kind() {
            RepositoryType::Hosted => match &parsed.kind {
                Kind::Metadata => Ok(hosted::metadata(ctx, &parsed.base)
                    .await?
                    .map(Found::Generated)),
                Kind::ArchetypeCatalog => Ok(Some(Found::Generated(hosted::catalog(ctx).await?))),
                _ => Ok(store::find(ctx.db(), ctx.name(), &parsed.base)
                    .await?
                    .map(|asset| Found::Stored(Box::new(asset)))),
            },
            RepositoryType::Proxy => {
                let fetchable = parsed.kind.follows_layout()
                    || matches!(parsed.kind, Kind::ArchetypeCatalog | Kind::Index);
                if strict(ctx) && !fetchable {
                    return Ok(None);
                }
                let found = proxy::fetch(ctx, &parsed.base, None, freshness(&parsed.kind), || {
                    describe(parsed, PutAsset::cached(parsed.base.clone()))
                })
                .await?;
                Ok(found.map(|asset| Found::Stored(Box::new(asset))))
            }
            RepositoryType::Group => Ok(None),
        }
    }

    async fn find_in_group(
        &self,
        ctx: &Ctx,
        parsed: &MavenPath,
    ) -> Result<Option<Found>, ContentError> {
        match &parsed.kind {
            Kind::Metadata => self.merged_metadata(ctx, parsed).await,
            Kind::ArchetypeCatalog => self.merged_catalog(ctx, parsed).await.map(Some),
            _ => Ok(group::first(ctx, |member| async move {
                self.find_in_member(&member, parsed).await
            })
            .await?
            .map(|(_, found)| found)),
        }
    }

    async fn member_documents(
        &self,
        ctx: &Ctx,
        parsed: &MavenPath,
    ) -> Result<Vec<(Vec<u8>, Option<DateTime<Utc>>)>, ContentError> {
        let mut documents = Vec::new();
        for member in group::members(ctx).await? {
            let member = ctx.member(member);
            let found = match self.find_in_member(&member, parsed).await {
                Ok(Some(found)) => found,
                Ok(None) => continue,
                Err(error) => {
                    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; merging the others");
                    continue;
                }
            };
            match found.bytes(&member, DOCUMENT_LIMIT).await {
                Ok(bytes) => documents.push((bytes, found.last_modified())),
                Err(error) => {
                    tracing::info!(group = ctx.name(), member = member.name(), %error, "a member's document cannot be read")
                }
            }
        }
        Ok(documents)
    }

    async fn merged_metadata(
        &self,
        ctx: &Ctx,
        parsed: &MavenPath,
    ) -> Result<Option<Found>, ContentError> {
        let mut last_modified = None;
        let mut parsed_documents = Vec::new();
        for (bytes, modified) in self.member_documents(ctx, parsed).await? {
            match metadata::parse(&bytes) {
                Ok(document) => {
                    parsed_documents.push(document);
                    last_modified = last_modified.max(modified);
                }
                Err(error) => {
                    tracing::info!(group = ctx.name(), path = parsed.base, %error, "a member's metadata is not valid and is left out")
                }
            }
        }
        Ok(metadata::merge(parsed_documents)
            .map(|merged| Found::Generated(Document::xml(metadata::write(&merged), last_modified))))
    }

    async fn merged_catalog(&self, ctx: &Ctx, parsed: &MavenPath) -> Result<Found, ContentError> {
        let mut last_modified = None;
        let mut catalogs = Vec::new();
        for (bytes, modified) in self.member_documents(ctx, parsed).await? {
            if let Ok(catalog) = pom::parse_catalog(&bytes) {
                catalogs.push(catalog);
                last_modified = last_modified.max(modified);
            }
        }
        Ok(Found::Generated(Document::xml(
            pom::write_catalog(&pom::merge_catalogs(catalogs)),
            last_modified,
        )))
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let parsed = path::parse(&request.path);
        ctx.require(View::Read, &parsed.path)?;
        if ctx.kind() != RepositoryType::Group
            && let Some(message) = refused(ctx, &parsed)
        {
            return Err(ContentError::NotFound(message));
        }
        let base = base_of(&parsed);
        let found = self.find(ctx, &base).await?;
        if let Some(hash) = parsed.hash {
            if let Some(found) = found {
                return respond::document(
                    &request.method,
                    &request.headers,
                    Document {
                        bytes: found.digest(hash).into_bytes(),
                        content_type: respond::TEXT,
                        last_modified: found.last_modified(),
                    },
                );
            }
            if ctx.kind() == RepositoryType::Hosted
                && let Some(stored) = store::find(ctx.db(), ctx.name(), &parsed.path).await?
            {
                return serve(
                    ctx,
                    &stored,
                    &request.method,
                    &request.headers,
                    Serve::default(),
                )
                .await;
            }
            return Err(not_found(&parsed.path));
        }
        match found {
            Some(Found::Stored(asset)) => {
                let options = match parsed.kind {
                    Kind::Metadata | Kind::ArchetypeCatalog => Serve::default(),
                    _ => Serve::download().disposition(disposition(ctx)),
                };
                serve(ctx, &asset, &request.method, &request.headers, options).await
            }
            Some(Found::Generated(document)) => {
                respond::document(&request.method, &request.headers, document)
            }
            None => Err(not_found(&parsed.path)),
        }
    }

    async fn write(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let parsed = path::parse(&request.path);
        if parsed.kind == Kind::ArchetypeCatalog {
            return Err(not_allowed(&request.method, READ_ALLOW));
        }
        if let Some(message) = refused(ctx, &parsed) {
            return Err(ContentError::BadRequest(message));
        }
        if strict(ctx) && !parsed.kind.follows_layout() {
            return Err(ContentError::BadRequest(format!(
                "Invalid path for a Maven 2 repository: {}",
                parsed.path
            )));
        }
        let write_policy = ctx
            .repository
            .attributes
            .storage
            .as_ref()
            .and_then(|storage| storage.write_policy);
        if write_policy == Some(WritePolicy::Deny) {
            return Err(ContentError::BadRequest(format!(
                "the repository {} is read-only",
                ctx.name()
            )));
        }
        if let Some(hash) = parsed.hash {
            return hosted::put_checksum(ctx, &parsed, hash, request).await;
        }
        match parsed.kind {
            Kind::Metadata => hosted::put_metadata(ctx, &parsed, request).await,
            _ => hosted::put_asset(ctx, &parsed, request).await,
        }
    }

    async fn delete(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let parsed = path::parse(&request.path);
        ctx.require(View::Delete, &parsed.path)?;
        let hosted = ctx.kind() == RepositoryType::Hosted;
        if hosted && parsed.hash.is_none() && parsed.kind == Kind::ArchetypeCatalog {
            return Err(not_allowed(&request.method, READ_ALLOW));
        }
        if store::delete(ctx, &parsed.path).await? {
            return Ok(StatusCode::NO_CONTENT.into_response());
        }
        let derived = parsed.hash.is_some() || (hosted && parsed.kind == Kind::Metadata);
        let base_exists = match (derived, hosted) {
            (false, _) => false,
            (true, true) => self.find(ctx, &base_of(&parsed)).await?.is_some(),
            (true, false) => store::find(ctx.db(), ctx.name(), &parsed.base)
                .await?
                .is_some(),
        };
        match base_exists {
            true => Ok(StatusCode::NO_CONTENT.into_response()),
            false => Err(not_found(&parsed.path)),
        }
    }

    fn directory(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        if !ctx.allowed(View::Browse, &request.path) {
            ctx.require(View::Read, &request.path)?;
        }
        let name = xml::escape(ctx.name());
        let index = format!(
            "{}/service/rest/repository/browse/{}{}",
            ctx.base_url,
            ctx.name(),
            request.path
        );
        let page = format!(
            "<!DOCTYPE html>\n<html lang=\"en\">\n<head><meta charset=\"utf-8\"><title>{name} - Astrum Repository</title></head>\n<body>\n<h1>{name}</h1>\n<p>Maven repositories are not listed at this address.</p>\n<p>See the <a href=\"{}\">HTML index</a> of this repository instead.</p>\n</body>\n</html>\n",
            xml::escape(&index)
        );
        respond::document(
            &request.method,
            &request.headers,
            Document {
                bytes: page.into_bytes(),
                content_type: respond::HTML,
                last_modified: None,
            },
        )
    }
}

#[async_trait]
impl Format for Maven {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let kind = ctx.kind();
        match request.method {
            Method::GET | Method::HEAD if request.is_directory() => self.directory(&ctx, &request),
            Method::GET | Method::HEAD => self.read(&ctx, &request).await,
            Method::PUT if kind == RepositoryType::Hosted && !request.is_directory() => {
                self.write(&ctx, request).await
            }
            Method::DELETE if kind != RepositoryType::Group && !request.is_directory() => {
                self.delete(&ctx, &request).await
            }
            Method::PUT | Method::DELETE
                if request.is_directory() && kind != RepositoryType::Group =>
            {
                Err(ContentError::BadRequest(
                    "a directory cannot be written or deleted".to_owned(),
                ))
            }
            method => Err(not_allowed(
                &method,
                match kind {
                    RepositoryType::Hosted => HOSTED_ALLOW,
                    RepositoryType::Proxy => "GET,HEAD,DELETE",
                    RepositoryType::Group => READ_ALLOW,
                },
            )),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        let parsed = path::parse(path);
        if parsed.hash.is_some() {
            return Ok(None);
        }
        Ok(match self.find(ctx, &parsed).await? {
            Some(Found::Stored(asset)) => Some(*asset),
            _ => None,
        })
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(upload::spec())
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        upload::upload(&ctx, form).await
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}
