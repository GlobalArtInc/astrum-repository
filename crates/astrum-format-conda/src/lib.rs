mod group;
pub mod hosted;
pub mod package;
pub mod paths;
pub mod raw;
pub mod repodata;
pub mod task;
pub mod testing;
pub mod version;

use astrum_content::format::SearchTarget;
use astrum_content::proxy;
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Freshness, SearchKey, Serve, UploadField,
    UploadForm, UploadSpec, View, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use axum::response::Response;
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::paths::{FileName, IndexFile, PackageType, Route};

pub const ID: &str = "conda";
pub const PACKAGE: &str = "package";
pub const METADATA: &str = "metadata";

/// Index files are checked again on every use, which costs a 304 when nothing changed: conda and
/// mamba otherwise keep a copy without `max-age` for as long as they like.
const INDEX_CACHE: &str = "no-cache, max-age=0";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "conda.subdir",
        target: SearchTarget::Asset,
        pointer: &["conda", "subdir"],
    },
    SearchKey {
        param: "conda.build",
        target: SearchTarget::Asset,
        pointer: &["conda", "build"],
    },
    SearchKey {
        param: "conda.license",
        target: SearchTarget::Asset,
        pointer: &["conda", "license"],
    },
];

#[derive(Default)]
pub struct Conda;

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn directory_page(ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let channel = format!("{}{}", ctx.repository_url(), path.trim_end_matches('/'));
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a conda channel. Install from it with <code>conda install -c {} &lt;package&gt;</code>; packages live in subdirectories such as <code>noarch/</code> and <code>linux-64/</code>, each with its <code>repodata.json</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&channel),
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn generated(request: &ContentRequest, content_type: &str, body: Vec<u8>) -> Response {
    let etag = format!("\"{}\"", hex::encode(&Sha256::digest(&body)[..16]));
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::CACHE_CONTROL, INDEX_CACHE);
    if unchanged {
        return builder
            .status(StatusCode::NOT_MODIFIED)
            .body(Body::empty())
            .expect("an empty response is valid");
    }
    builder
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, body.len())
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .expect("a generated response is valid")
}

fn metadata_type(file: &str) -> Option<&'static str> {
    if file.ends_with(".json") {
        Some("application/json")
    } else if file.ends_with(".bz2") {
        Some("application/x-bzip2")
    } else if file.ends_with(".zst") {
        Some("application/zstd")
    } else {
        None
    }
}

fn empty_hosted(route: &Route) -> Option<(Vec<u8>, &'static str)> {
    match route {
        Route::Index { subdir, file, .. } => {
            let json = repodata::Repodata::empty(subdir).to_bytes();
            let bytes = match file {
                IndexFile::Json | IndexFile::Current => json,
                IndexFile::Bz2 => repodata::bz2(&json),
                IndexFile::Zst => repodata::zst(&json),
            };
            Some((bytes, file.content_type()))
        }
        Route::Channeldata { .. } => Some((repodata::empty_channeldata(), "application/json")),
        _ => None,
    }
}

impl Conda {
    async fn proxied(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        let route = paths::route(path);
        let freshness = match route {
            Route::Index { .. } | Route::Channeldata { .. } => Freshness::Metadata,
            _ => Freshness::Content,
        };
        proxy::fetch(ctx, path, None, freshness, || match &route {
            Route::Package {
                channel,
                subdir,
                file,
            } => PutAsset::cached(path.to_owned())
                .kind(PACKAGE)
                .content_type(file.kind.content_type())
                .component(ComponentKey {
                    namespace: channel.clone(),
                    name: file.name.clone(),
                    version: file.version.clone(),
                    kind: String::new(),
                    attributes: json!({}),
                })
                .attributes(json!({ ID: {
                    "channel": channel,
                    "subdir": subdir,
                    "build": file.build,
                    "packageType": file.kind.id(),
                }})),
            Route::Index { .. } | Route::Channeldata { .. } => {
                let put = PutAsset::cached(path.to_owned()).kind(METADATA);
                match metadata_type(paths::file_of(path)) {
                    Some(content_type) => put.content_type(content_type),
                    None => put,
                }
            }
            _ => PutAsset::cached(path.to_owned()).kind(METADATA),
        })
        .await
    }

    async fn hosted_index(
        &self,
        ctx: &Ctx,
        route: &Route,
        path: &str,
    ) -> Result<Option<Asset>, ContentError> {
        match route {
            Route::Index { channel, .. } | Route::Channeldata { channel } => {
                hosted::refresh(ctx, channel, false).await?;
            }
            _ => {}
        }
        store::find(ctx.db(), ctx.name(), path).await
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        let route = paths::route(path);
        if route == Route::Directory {
            return directory_page(ctx, path);
        }
        ctx.require(View::Read, path)?;
        if route == Route::Unsupported {
            return Err(ContentError::not_found(path.to_owned()));
        }
        let index = matches!(route, Route::Index { .. } | Route::Channeldata { .. });
        let mut response = self.respond(ctx, request, &route).await?;
        if index {
            response.headers_mut().insert(
                header::CACHE_CONTROL,
                header::HeaderValue::from_static(INDEX_CACHE),
            );
        }
        Ok(response)
    }

    async fn respond(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        route: &Route,
    ) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        let download = matches!(route, Route::Package { .. });
        let options = if download {
            Serve::download()
        } else {
            Serve::default()
        };
        match ctx.kind() {
            RepositoryType::Hosted => {
                if let Some(found) = self.hosted_index(ctx, route, path).await? {
                    return serve(ctx, &found, &request.method, &request.headers, options).await;
                }
                match empty_hosted(route) {
                    Some((bytes, content_type)) => Ok(generated(request, content_type, bytes)),
                    None => Err(ContentError::not_found(path.to_owned())),
                }
            }
            RepositoryType::Proxy => {
                let found = self
                    .proxied(ctx, path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
                serve(ctx, &found, &request.method, &request.headers, options).await
            }
            RepositoryType::Group => {
                let answer = match route {
                    Route::Index {
                        channel,
                        subdir,
                        file,
                    } => group::index(self, ctx, channel, subdir, *file).await?,
                    Route::Channeldata { channel } => {
                        group::channeldata(self, ctx, channel).await?
                    }
                    _ => astrum_content::group::first(ctx, |member| async move {
                        self.get(&member, path).await
                    })
                    .await?
                    .map(|(member, found)| group::Answer::Member(Box::new(member), found)),
                };
                match answer {
                    Some(group::Answer::Member(member, found)) => {
                        serve(&member, &found, &request.method, &request.headers, options).await
                    }
                    Some(group::Answer::Merged(found)) => {
                        serve(ctx, &found, &request.method, &request.headers, options).await
                    }
                    Some(group::Answer::Generated(bytes, content_type)) => {
                        Ok(generated(request, content_type, bytes))
                    }
                    None => Err(ContentError::not_found(path.to_owned())),
                }
            }
        }
    }

    async fn put(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.clone();
        let (channel, subdir, file) = match paths::route(&path) {
            Route::Package {
                channel,
                subdir,
                file,
            } => (channel, subdir, file),
            Route::Index { .. } | Route::Channeldata { .. } => {
                return Err(ContentError::BadRequest(
                    "the indexes of a hosted conda repository are generated; upload packages instead"
                        .to_owned(),
                ));
            }
            _ => {
                return Err(ContentError::BadRequest(format!(
                    "{path} is not the place of a conda package: upload <name>-<version>-<build>.conda or .tar.bz2 to <channel>/<subdir>/"
                )));
            }
        };
        if !ctx.allowed(View::Add, &path) && !ctx.allowed(View::Edit, &path) {
            ctx.require(View::Add, &path)?;
        }
        let written = store::write_blob(ctx, request.into_stream()).await?;
        hosted::adopt(
            ctx,
            written,
            hosted::Target::Path {
                channel: &channel,
                subdir: &subdir,
                file: &file,
            },
        )
        .await?;
        Response::builder()
            .status(StatusCode::CREATED)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        let Some(found) = store::find(ctx.db(), ctx.name(), path).await? else {
            return Err(ContentError::not_found(path.to_owned()));
        };
        if ctx.kind() == RepositoryType::Hosted && found.kind != PACKAGE {
            return Err(ContentError::BadRequest(
                "the indexes of a hosted conda repository are generated; delete packages instead"
                    .to_owned(),
            ));
        }
        store::delete_asset(&ctx.content, found).await?;
        if ctx.kind() == RepositoryType::Hosted
            && let Some((channel, _)) = paths::place_of(path)
        {
            hosted::refresh(ctx, &channel, false).await?;
        }
        Response::builder()
            .status(StatusCode::NO_CONTENT)
            .body(Body::empty())
            .map_err(ContentError::internal)
    }

    /// Writes every index of a hosted repository again.
    pub async fn rebuild(&self, ctx: &Ctx) -> Result<usize, ContentError> {
        let channels = hosted::channels(ctx).await?;
        for channel in &channels {
            hosted::refresh(ctx, channel, true).await?;
        }
        Ok(channels.len())
    }
}

#[async_trait]
impl Format for Conda {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        match (&request.method, ctx.kind()) {
            (&Method::GET | &Method::HEAD, _) => self.read(&ctx, &request).await,
            (&Method::PUT, RepositoryType::Hosted) => self.put(&ctx, request).await,
            (&Method::DELETE, RepositoryType::Hosted | RepositoryType::Proxy) => {
                self.delete(&ctx, &request.path).await
            }
            (method, kind) => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: match kind {
                    RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
                    RepositoryType::Proxy => "GET,HEAD,DELETE",
                    RepositoryType::Group => "GET,HEAD",
                },
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        let route = paths::route(path);
        match ctx.kind() {
            RepositoryType::Hosted => self.hosted_index(ctx, &route, path).await,
            RepositoryType::Proxy => {
                if route == Route::Unsupported {
                    return Ok(None);
                }
                self.proxied(ctx, path).await
            }
            RepositoryType::Group => {
                let answer = match &route {
                    Route::Index {
                        channel,
                        subdir,
                        file,
                    } => group::index(self, ctx, channel, subdir, *file).await?,
                    Route::Channeldata { channel } => {
                        group::channeldata(self, ctx, channel).await?
                    }
                    Route::Unsupported => None,
                    _ => {
                        return Ok(astrum_content::group::first(ctx, |member| async move {
                            self.get(&member, path).await
                        })
                        .await?
                        .map(|(_, found)| found));
                    }
                };
                Ok(match answer {
                    Some(group::Answer::Member(_, found)) | Some(group::Answer::Merged(found)) => {
                        Some(found)
                    }
                    _ => None,
                })
            }
        }
    }

    fn upload_spec(&self) -> Option<UploadSpec> {
        Some(UploadSpec {
            format: ID,
            multiple_upload: false,
            component_fields: vec![
                UploadField::text("channel", true)
                    .describe("Channel path inside the repository, for example main; leave empty for the repository itself")
                    .in_group("Component attributes"),
            ],
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
            .get("conda.asset")
            .ok_or_else(|| ContentError::BadRequest("conda.asset is required".to_owned()))?;
        let channel = form
            .field("conda.channel")
            .unwrap_or("")
            .trim()
            .trim_matches('/')
            .to_owned();
        if !paths::valid_channel(&channel) {
            return Err(ContentError::BadRequest(format!(
                "{channel} is not a valid channel path: use letters, digits, '-', '_' and '.' in segments separated by '/'"
            )));
        }
        let file_name = file
            .filename
            .as_deref()
            .map(|name| name.rsplit(['/', '\\']).next().unwrap_or(name).to_owned());
        if !(ctx.grant(View::Add).any() || ctx.grant(View::Edit).any()) {
            ctx.require(View::Add, &paths::join(&channel, ""))?;
        }
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        hosted::adopt(
            &ctx,
            written,
            hosted::Target::Channel {
                channel: &channel,
                file_name: file_name.as_deref(),
            },
        )
        .await?;
        Ok(())
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}

pub fn file_name(name: &str, version: &str, build: &str, kind: PackageType) -> String {
    format!(
        "{}{}",
        FileName::stem(name, version, build),
        kind.extension()
    )
}
