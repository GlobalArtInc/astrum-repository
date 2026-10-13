mod archive;
mod group;
mod hosted;
pub mod paths;
mod pgp;
mod proxy;
mod reply;
mod routes;
mod sums;
mod tokens;
pub mod versions;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use astrum_content::format::SearchTarget;
use astrum_content::store;
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, SearchKey, Serve, UploadField, UploadForm,
    UploadSpec, View, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};

use paths::{Endpoint, FileKind, Module, Provider};

pub use routes::routes;

pub const ID: &str = "terraform";
pub const REALM: &str = "TerraformToken";
pub const MODULE_ARCHIVE: &str = "MODULE_ARCHIVE";
pub const METADATA: &str = "METADATA";
pub const KIND_MODULE: &str = "module";
pub const KIND_PROVIDER: &str = "provider";
pub const DEFAULT_PROTOCOLS: &[&str] = &["5.0"];

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "terraform.namespace",
        target: SearchTarget::Component,
        pointer: &["terraform", "namespace"],
    },
    SearchKey {
        param: "terraform.name",
        target: SearchTarget::Component,
        pointer: &["terraform", "name"],
    },
    SearchKey {
        param: "terraform.provider",
        target: SearchTarget::Component,
        pointer: &["terraform", "provider"],
    },
    SearchKey {
        param: "terraform.kind",
        target: SearchTarget::Component,
        pointer: &["terraform", "kind"],
    },
];

#[derive(Default)]
pub struct Terraform {
    secret: tokens::Secret,
    locks: Mutex<HashMap<String, Arc<tokio::sync::Mutex<()>>>>,
}

pub enum Download {
    Location(String),
    Missing(StatusCode, String),
}

pub enum Document {
    Found(Value),
    Missing(StatusCode, String),
}

impl Document {
    fn missing(message: impl Into<String>) -> Self {
        Document::Missing(StatusCode::NOT_FOUND, message.into())
    }

    fn reply(self, request: &ContentRequest) -> Result<Response, ContentError> {
        match self {
            Document::Found(value) => reply::json(request, StatusCode::OK, &value),
            Document::Missing(status, message) => reply::error(request, status, &message),
        }
    }
}

pub fn requires_authentication(ctx: &Ctx) -> bool {
    ctx.repository
        .attributes
        .terraform
        .as_ref()
        .and_then(|terraform| terraform.require_authentication)
        .unwrap_or(false)
}

fn anonymous(ctx: &Ctx) -> bool {
    ctx.subject.as_ref().is_none_or(|subject| subject.anonymous)
}

fn allow(ctx: &Ctx) -> &'static str {
    match ctx.kind() {
        RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
        RepositoryType::Proxy => "GET,HEAD,DELETE",
        RepositoryType::Group => "GET,HEAD",
    }
}

pub fn discovery() -> Value {
    json!({
        "modules.v1": "../v1/modules/",
        "providers.v1": "../v1/providers/",
    })
}

pub fn module_versions_document(versions: &[String]) -> Value {
    json!({
        "modules": [{
            "versions": versions.iter().map(|version| json!({"version": version})).collect::<Vec<_>>(),
        }],
    })
}

pub fn mirror_index_document(versions: &[String]) -> Value {
    let entries: serde_json::Map<String, Value> = versions
        .iter()
        .map(|version| (version.clone(), json!({})))
        .collect();
    json!({ "versions": entries })
}

impl Terraform {
    pub(crate) fn lock(&self, key: &str) -> Arc<tokio::sync::Mutex<()>> {
        let mut locks = self.locks.lock().expect("the lock map is never poisoned");
        if locks.len() > 4096 {
            locks.retain(|_, lock| Arc::strong_count(lock) > 1);
        }
        locks.entry(key.to_owned()).or_default().clone()
    }

    pub(crate) async fn tokens(&self, ctx: &Ctx) -> Result<tokens::Tokens, ContentError> {
        self.secret.tokens(ctx.db()).await
    }

    async fn check_read(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        endpoint: &Endpoint,
    ) -> Result<(), ContentError> {
        if matches!(
            endpoint,
            Endpoint::ModuleArchive(..) | Endpoint::ProviderFile(..)
        ) && self
            .tokens(ctx)
            .await?
            .valid(ctx.name(), &request.path, request.query.as_deref())
        {
            return Ok(());
        }
        if requires_authentication(ctx) && anonymous(ctx) {
            return Err(ContentError::Unauthorized);
        }
        ctx.require(View::Read, &request.path)
    }

    pub(crate) async fn module_versions(
        &self,
        ctx: &Ctx,
        module: &Module,
    ) -> Result<Option<Vec<String>>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => hosted::module_versions(ctx, module).await,
            RepositoryType::Proxy => proxy::module_versions(ctx, module).await,
            RepositoryType::Group => group::module_versions(self, ctx, module).await,
        }
    }

    pub(crate) async fn module_download(
        &self,
        ctx: &Ctx,
        module: &Module,
    ) -> Result<Download, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => hosted::module_download(self, ctx, module).await,
            RepositoryType::Proxy => proxy::module_download(self, ctx, module).await,
            RepositoryType::Group => group::module_download(self, ctx, module).await,
        }
    }

    pub(crate) async fn provider_versions(
        &self,
        ctx: &Ctx,
        provider: &Provider,
    ) -> Result<Document, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => hosted::provider_versions(ctx, provider).await,
            RepositoryType::Proxy => proxy::provider_versions(ctx, provider).await,
            RepositoryType::Group => group::provider_versions(self, ctx, provider).await,
        }
    }

    pub(crate) async fn provider_download(
        &self,
        ctx: &Ctx,
        provider: &Provider,
        os: &str,
        arch: &str,
    ) -> Result<Document, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => hosted::provider_package(self, ctx, provider, os, arch).await,
            RepositoryType::Proxy => proxy::provider_download(self, ctx, provider, os, arch).await,
            RepositoryType::Group => group::provider_download(self, ctx, provider, os, arch).await,
        }
    }

    pub(crate) async fn mirror_index(
        &self,
        ctx: &Ctx,
        host: &str,
        provider: &Provider,
    ) -> Result<Document, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => hosted::mirror_index(ctx, provider).await,
            RepositoryType::Proxy => proxy::mirror_index(ctx, host, provider).await,
            RepositoryType::Group => group::mirror_index(self, ctx, host, provider).await,
        }
    }

    pub(crate) async fn mirror_version(
        &self,
        ctx: &Ctx,
        host: &str,
        provider: &Provider,
        document: &str,
    ) -> Result<Document, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => hosted::mirror_version(self, ctx, provider, document).await,
            RepositoryType::Proxy => {
                proxy::mirror_version(self, ctx, host, provider, document).await
            }
            RepositoryType::Group => {
                group::mirror_version(self, ctx, host, provider, document).await
            }
        }
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let endpoint = Endpoint::parse(&request.path);
        if endpoint == Endpoint::Discovery {
            return reply::json(request, StatusCode::OK, &discovery());
        }
        if matches!(endpoint, Endpoint::Other | Endpoint::Token) {
            if request.is_directory() {
                return reply::directory(ctx, request);
            }
            return Err(ContentError::not_found(request.path.clone()));
        }
        self.check_read(ctx, request, &endpoint).await?;
        match endpoint {
            Endpoint::ModuleVersions(module) => match self.module_versions(ctx, &module).await? {
                Some(versions) if !versions.is_empty() => reply::json(
                    request,
                    StatusCode::OK,
                    &module_versions_document(&versions),
                ),
                _ => reply::error(
                    request,
                    StatusCode::NOT_FOUND,
                    &format!("Module {} not found", module.address()),
                ),
            },
            Endpoint::ModuleDownload(module) => match self.module_download(ctx, &module).await? {
                Download::Location(location) => reply::location(&location),
                Download::Missing(status, message) => reply::error(request, status, &message),
            },
            Endpoint::ProviderVersions(provider) => {
                self.provider_versions(ctx, &provider).await?.reply(request)
            }
            Endpoint::ProviderDownload(provider, os, arch) => self
                .provider_download(ctx, &provider, &os, &arch)
                .await?
                .reply(request),
            Endpoint::MirrorIndex(host, provider) => self
                .mirror_index(ctx, &host, &provider)
                .await?
                .reply(request),
            Endpoint::MirrorVersion(host, provider) => self
                .mirror_version(ctx, &host, &provider, &request.path)
                .await?
                .reply(request),
            Endpoint::ModuleArchive(..) | Endpoint::ProviderFile(..) => {
                let asset = self
                    .get(ctx, &request.path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
                serve(
                    ctx,
                    &asset,
                    &request.method,
                    &request.headers,
                    Serve::download(),
                )
                .await
            }
            Endpoint::Discovery | Endpoint::Token | Endpoint::Other => {
                Err(ContentError::not_found(request.path.clone()))
            }
        }
    }

    async fn write(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let protocols = request
            .query
            .as_deref()
            .and_then(|query| {
                url::form_urlencoded::parse(query.as_bytes())
                    .find(|(name, _)| name == "protocols")
                    .map(|(_, value)| value.into_owned())
            })
            .map(|value| hosted::protocols(&value))
            .transpose()?;
        match Endpoint::parse(&request.path) {
            Endpoint::ModuleArchive(module, file) => {
                hosted::put_module(self, ctx, &module, Some(&file), request.into_stream()).await?;
                Ok(StatusCode::CREATED.into_response())
            }
            Endpoint::ProviderFile(provider, file, _) => {
                hosted::put_provider_files(
                    self,
                    ctx,
                    &provider,
                    vec![(file, request.into_stream())],
                    protocols,
                )
                .await?;
                Ok(StatusCode::CREATED.into_response())
            }
            Endpoint::Other => Err(ContentError::BadRequest(format!(
                "{} is not a place for Terraform files: upload module packages to v1/modules/<namespace>/<name>/<system>/<version>/<file> and provider release files to v1/providers/<namespace>/<type>/<version>/<file>",
                request.path
            ))),
            _ => Err(ContentError::MethodNotAllowed {
                method: request.method.to_string(),
                allow: "GET,HEAD",
            }),
        }
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        let endpoint = Endpoint::parse(path);
        let deletable = match ctx.kind() {
            RepositoryType::Hosted => matches!(
                endpoint,
                Endpoint::ModuleArchive(..) | Endpoint::ProviderFile(..)
            ),
            _ => !matches!(endpoint, Endpoint::Discovery | Endpoint::Token),
        };
        if !deletable {
            return Err(ContentError::not_found(path.to_owned()));
        }
        ctx.require(View::Delete, path)?;
        if !store::delete(ctx, path).await? {
            return Err(ContentError::not_found(path.to_owned()));
        }
        if ctx.kind() == RepositoryType::Hosted
            && let Endpoint::ProviderFile(provider, _, kind) = endpoint
            && matches!(kind, FileKind::Archive { .. })
        {
            hosted::refresh_checksums(self, ctx, &provider).await?;
        }
        Ok(StatusCode::NO_CONTENT.into_response())
    }
}

#[async_trait]
impl Format for Terraform {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (&request.method, ctx.kind()) {
            (&Method::GET | &Method::HEAD, _) => self.read(&ctx, &request).await,
            (&Method::DELETE, RepositoryType::Hosted | RepositoryType::Proxy) => {
                self.delete(&ctx, &request.path).await
            }
            (&Method::PUT, RepositoryType::Hosted) => self.write(&ctx, request).await,
            (method, _) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: allow(&ctx),
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => store::find(ctx.db(), ctx.name(), path).await,
            RepositoryType::Proxy => proxy::file(self, ctx, path).await,
            RepositoryType::Group => Ok(astrum_content::group::first(ctx, |member| async move {
                self.get(&member, path).await
            })
            .await?
            .map(|(_, asset)| asset)),
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        let field = |name, optional, group, description| {
            UploadField::text(name, optional)
                .describe(description)
                .in_group(group)
        };
        Some(UploadSpec {
            format: ID,
            multiple_upload: true,
            component_fields: vec![
                field(
                    "uploadType",
                    true,
                    "Coordinates",
                    "module or provider; taken from the coordinates when empty",
                ),
                field(
                    "namespace",
                    false,
                    "Coordinates",
                    "Organization or group of the module or provider",
                ),
                field(
                    "version",
                    false,
                    "Coordinates",
                    "Semantic version, for example 1.2.0",
                ),
                field(
                    "name",
                    true,
                    "Module Coordinates",
                    "Module name, for example vpc",
                ),
                field(
                    "system",
                    true,
                    "Module Coordinates",
                    "Target system of the module, for example aws",
                ),
                field(
                    "type",
                    true,
                    "Provider Coordinates",
                    "Provider type, for example aws",
                ),
                field(
                    "protocols",
                    true,
                    "Provider Coordinates",
                    "Plugin protocol versions of the provider, for example 6.0; 5.0 when empty",
                ),
            ],
            asset_fields: vec![],
        })
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "only hosted repositories take uploads".to_owned(),
            ));
        }
        hosted::upload(self, &ctx, form).await
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}
