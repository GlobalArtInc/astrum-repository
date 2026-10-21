pub mod archive;
pub mod db;
mod files;
mod group;
mod hosted;
pub mod package;
pub mod paths;
mod remote;
pub mod signing;
pub mod task;
pub mod testing;
pub mod version;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use astrum_content::format::SearchTarget;
use astrum_content::store;
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, SearchKey, Serve, UploadField,
    UploadForm, UploadSpec, View, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use uuid::Uuid;

use crate::group::GroupDatabase;
use crate::paths::{Database, KEY_SUFFIX, Route};
use crate::signing::{KEY_TYPE, SIGNATURE_TYPE, SigningKey};

pub const ID: &str = "pacman";
pub const PACKAGE: &str = "package";
pub const SIGNATURE: &str = "signature";
pub const DATABASE: &str = "database";
pub const OTHER: &str = "other";
pub const DATABASE_TYPE: &str = "application/gzip";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "pacman.architecture",
        target: SearchTarget::Asset,
        pointer: &["pacman", "architecture"],
    },
    SearchKey {
        param: "pacman.base",
        target: SearchTarget::Asset,
        pointer: &["pacman", "base"],
    },
    SearchKey {
        param: "pacman.license",
        target: SearchTarget::Asset,
        pointer: &["pacman", "license"],
    },
];

const KEYWORDS: &[&[&str]] = &[&["pacman", "description"]];

type SignatureCache = HashMap<(String, String, String), (Uuid, String, Vec<u8>)>;

#[derive(Default)]
pub struct Pacman {
    keys: Arc<signing::Keys>,
    signatures: Mutex<SignatureCache>,
}

enum Signature {
    Made(Vec<u8>),
    Stored(Box<(Ctx, Asset)>),
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

pub fn key_name(repository: &str) -> String {
    format!("{repository}{KEY_SUFFIX}")
}

fn directory_page(ctx: &Ctx, key: Option<&SigningKey>) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let url = escape(&ctx.repository_url());
    let key = match key {
        Some(key) => {
            let file = escape(&key_name(ctx.name()));
            format!(
                "<p>Its databases are signed with the key {fingerprint}: add <a href=\"{url}/{file}\">{file}</a> with <code>pacman-key --add {file}</code> and <code>pacman-key --lsign-key {fingerprint}</code>.</p>\n    ",
                fingerprint = key.fingerprint,
            )
        }
        None => String::new(),
    };
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a pacman package repository. Add a section to <code>/etc/pacman.conf</code> with <code>Server = {url}/$arch</code> for packages stored in folders named after their architecture, or <code>Server = {url}/$repo/os/$arch</code> for the layout of an Arch Linux mirror.</p>\n    {key}<p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn bytes_response(
    request: &ContentRequest,
    content_type: &str,
    body: Vec<u8>,
) -> Result<Response, ContentError> {
    let length = body.len();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, length)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .map_err(ContentError::internal)
}

fn allow(ctx: &Ctx) -> &'static str {
    match ctx.kind() {
        RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
        RepositoryType::Proxy => "GET,HEAD,DELETE",
        RepositoryType::Group => "GET,HEAD",
    }
}

async fn serve_found(
    ctx: &Ctx,
    request: &ContentRequest,
    found: Option<Asset>,
) -> Result<Response, ContentError> {
    let found = found.ok_or_else(|| ContentError::not_found(request.path.clone()))?;
    let options = Serve {
        count_download: found.kind == PACKAGE,
        ..Serve::default()
    };
    serve(ctx, &found, &request.method, &request.headers, options).await
}

impl Pacman {
    async fn database(
        &self,
        ctx: &Ctx,
        wanted: &Database,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(self
                .get(ctx, &wanted.path())
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Proxy => Ok(remote::get(ctx, &wanted.path())
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => Ok(match group::database(self, ctx, wanted).await? {
                Some(GroupDatabase::Passed(member, found)) => Some((*member, found)),
                Some(GroupDatabase::Merged(found)) => Some((ctx.clone(), found)),
                None => None,
            }),
        }
    }

    async fn sign(
        &self,
        signer: &Ctx,
        key: Arc<SigningKey>,
        owner: &Ctx,
        found: &Asset,
    ) -> Result<Vec<u8>, ContentError> {
        let cache_key = (
            signer.name().to_owned(),
            owner.name().to_owned(),
            found.path.clone(),
        );
        let cached = self
            .signatures
            .lock()
            .expect("the signature cache lock is never poisoned")
            .get(&cache_key)
            .filter(|(blob, fingerprint, _)| {
                *blob == found.blob_id && *fingerprint == key.fingerprint
            })
            .map(|(_, _, signature)| signature.clone());
        if let Some(signature) = cached {
            return Ok(signature);
        }
        let file = files::download(owner, &found.blob_store, found.blob_id.into()).await?;
        let signing = key.clone();
        let signature =
            tokio::task::spawn_blocking(move || signing.sign(std::io::BufReader::new(file)))
                .await
                .map_err(ContentError::internal)?
                .map_err(|error| ContentError::internal(std::io::Error::other(error)))?;
        self.signatures
            .lock()
            .expect("the signature cache lock is never poisoned")
            .insert(
                cache_key,
                (found.blob_id, key.fingerprint.clone(), signature.clone()),
            );
        Ok(signature)
    }

    async fn leaf_signature(
        &self,
        ctx: &Ctx,
        wanted: &Database,
        served: Option<(Ctx, Asset)>,
    ) -> Result<Option<Signature>, ContentError> {
        if let Some(key) = self.keys.get(ctx)? {
            let served = match served {
                Some(served) => Some(served),
                None => self.database(ctx, wanted).await?,
            };
            return Ok(match served {
                Some((owner, found)) => {
                    Some(Signature::Made(self.sign(ctx, key, &owner, &found).await?))
                }
                None => None,
            });
        }
        match ctx.kind() {
            RepositoryType::Proxy => Ok(remote::get(ctx, &wanted.signature_path())
                .await?
                .map(|found| Signature::Stored(Box::new((ctx.clone(), found))))),
            _ => Ok(None),
        }
    }

    async fn database_signature(
        &self,
        ctx: &Ctx,
        wanted: &Database,
    ) -> Result<Option<Signature>, ContentError> {
        if ctx.kind() != RepositoryType::Group || self.keys.get(ctx)?.is_some() {
            return self.leaf_signature(ctx, wanted, None).await;
        }
        match group::database(self, ctx, wanted).await? {
            Some(GroupDatabase::Passed(member, found)) => {
                let owner = (*member).clone();
                self.leaf_signature(&member, wanted, Some((owner, found)))
                    .await
            }
            _ => Ok(None),
        }
    }

    async fn key_file(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        file: &str,
    ) -> Result<Response, ContentError> {
        if let Some(key) = self.keys.get(ctx)?
            && file == key_name(ctx.name())
        {
            return bytes_response(request, KEY_TYPE, key.public_key.clone());
        }
        self.plain_file(ctx, request).await
    }

    async fn plain_file(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        match ctx.kind() {
            RepositoryType::Group => match group::file(self, ctx, &request.path).await? {
                Some((member, found)) => serve_found(&member, request, Some(found)).await,
                None => Err(ContentError::not_found(request.path.clone())),
            },
            _ => {
                let found = self.get(ctx, &request.path).await?;
                serve_found(ctx, request, found).await
            }
        }
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        let route = paths::route(path);
        if route == Route::Directory {
            let key = self.keys.get(ctx).ok().flatten();
            return directory_page(ctx, key.as_deref());
        }
        ctx.require(View::Read, path)?;
        match route {
            Route::Database(wanted) => match self.database(ctx, &wanted).await? {
                Some((owner, found)) => serve_found(&owner, request, Some(found)).await,
                None => Err(ContentError::not_found(path.to_owned())),
            },
            Route::DatabaseSignature(wanted) => {
                match self.database_signature(ctx, &wanted).await? {
                    Some(Signature::Made(signature)) => {
                        bytes_response(request, SIGNATURE_TYPE, signature)
                    }
                    Some(Signature::Stored(stored)) => {
                        let (owner, found) = *stored;
                        serve_found(&owner, request, Some(found)).await
                    }
                    None => Err(ContentError::not_found(path.to_owned())),
                }
            }
            Route::Key(file) => self.key_file(ctx, request, &file).await,
            Route::Package(_) | Route::Signature(_) | Route::File | Route::Directory => {
                self.plain_file(ctx, request).await
            }
        }
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        let Some(found) = store::find(ctx.db(), ctx.name(), path).await? else {
            return Err(ContentError::not_found(path.to_owned()));
        };
        match ctx.kind() {
            RepositoryType::Hosted if found.kind == DATABASE => {
                return Err(ContentError::BadRequest(
                    "the databases of a hosted pacman repository are generated; delete packages instead"
                        .to_owned(),
                ));
            }
            RepositoryType::Hosted => hosted::delete(ctx, found).await?,
            _ => store::delete_asset(&ctx.content, found).await?,
        }
        Ok(StatusCode::NO_CONTENT.into_response())
    }

    pub async fn rebuild(&self, ctx: &Ctx) -> Result<u64, ContentError> {
        hosted::rebuild_all(ctx).await
    }
}

#[async_trait]
impl Format for Pacman {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (&request.method, ctx.kind()) {
            (&Method::GET | &Method::HEAD, _) => self.read(&ctx, &request).await,
            (&Method::PUT, RepositoryType::Hosted) if !request.is_directory() => {
                let path = request.path.clone();
                hosted::put(&ctx, &path, request.into_stream()).await?;
                Ok(StatusCode::CREATED.into_response())
            }
            (&Method::DELETE, RepositoryType::Hosted | RepositoryType::Proxy) => {
                self.delete(&ctx, &request.path).await
            }
            (method, _) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: allow(&ctx),
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match (ctx.kind(), paths::route(path)) {
            (RepositoryType::Hosted, Route::Database(wanted)) if wanted.generated() => {
                Ok(hosted::ensure(ctx, &wanted.directory)
                    .await?
                    .map(|generated| generated.of(wanted.variant)))
            }
            (RepositoryType::Hosted, Route::Database(_) | Route::DatabaseSignature(_)) => Ok(None),
            (RepositoryType::Hosted, Route::Package(file) | Route::Signature(file)) => {
                hosted::find_package(ctx, path, &file).await
            }
            (RepositoryType::Hosted, _) => store::find(ctx.db(), ctx.name(), path).await,
            (RepositoryType::Proxy, _) => remote::get(ctx, path).await,
            (RepositoryType::Group, Route::Database(wanted)) => {
                Ok(group::database(self, ctx, &wanted)
                    .await?
                    .map(|found| found.asset().clone()))
            }
            (RepositoryType::Group, Route::DatabaseSignature(_)) => Ok(None),
            (RepositoryType::Group, _) => {
                Ok(group::file(self, ctx, path).await?.map(|(_, found)| found))
            }
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: true,
            component_fields: vec![
                UploadField::text("directory", true)
                    .describe("Directory above the architecture folders, for example custom/os; empty stores packages in x86_64, any and the other architecture folders at the top")
                    .in_group("Component attributes"),
            ],
            asset_fields: vec![UploadField::file("asset").describe(
                "A package (.pkg.tar.zst, .pkg.tar.xz, ...) or its detached signature (.sig)",
            )],
        })
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(
                "only hosted repositories take uploads".to_owned(),
            ));
        }
        hosted::upload(&ctx, &form).await
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        KEYWORDS
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout::default()
    }
}
