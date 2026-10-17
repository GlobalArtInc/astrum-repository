mod paths;
mod revision;

use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use astrum_content::format::SearchTarget;
use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Freshness, SearchKey, Serve, View, serve,
};
use astrum_db::entity::component;
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use futures_util::TryStreamExt;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde_json::{Value, json};

use paths::{DEFAULT_REVISION, RepoType, Request, is_commit};
use revision::{RepoFile, Revision};

pub const ID: &str = "huggingface";
pub const REVISION: &str = "REVISION";
pub const FILE: &str = "FILE";
const MAX_REVISION: u64 = 64 * 1024 * 1024;
const MAX_FORM: usize = 1024 * 1024;

const SEGMENT: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}')
    .add(b'%')
    .add(b'/')
    .add(b'|')
    .add(b'\\')
    .add(b'^')
    .add(b'[')
    .add(b']');

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "huggingface.revision",
        target: SearchTarget::Component,
        pointer: &["huggingface", "revisions"],
    },
    SearchKey {
        param: "huggingface.file",
        target: SearchTarget::Asset,
        pointer: &["huggingface", "file"],
    },
];

#[derive(Default)]
pub struct HuggingFace {
    parsed: Mutex<HashMap<String, Arc<Revision>>>,
}

struct Failure {
    status: StatusCode,
    code: Option<&'static str>,
    message: String,
}

impl Failure {
    fn new(status: StatusCode, code: Option<&'static str>, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    fn reply(self) -> Response {
        let message: String = self
            .message
            .chars()
            .map(|c| if (' '..'\u{7f}').contains(&c) { c } else { ' ' })
            .take(500)
            .collect();
        let body = json!({ "error": message }).to_string();
        let mut response = (self.status, body).into_response();
        let headers = response.headers_mut();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        if let Ok(value) = HeaderValue::from_str(&message) {
            headers.insert("x-error-message", value);
        }
        if let Some(code) = self.code {
            headers.insert("x-error-code", HeaderValue::from_static(code));
        }
        response
    }
}

impl From<ContentError> for Failure {
    fn from(error: ContentError) -> Self {
        Failure::new(error.status(), None, error.to_string())
    }
}

fn json_response(request: &ContentRequest, value: &Value) -> Response {
    let body = value.to_string();
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CONTENT_LENGTH, body.len())
        .body(match request.method {
            Method::HEAD => Body::empty(),
            _ => Body::from(body),
        })
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|segment| utf8_percent_encode(segment, SEGMENT).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

fn remote(ctx: &Ctx) -> Result<url::Url, ContentError> {
    let base = ctx
        .repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.as_deref())
        .ok_or_else(|| ContentError::internal(anyhow::anyhow!("the proxy has no remote URL")))?;
    let base = match base.ends_with('/') {
        true => base.to_owned(),
        false => format!("{base}/"),
    };
    url::Url::parse(&base)
        .map_err(|error| ContentError::BadGateway(format!("the remote URL is not valid: {error}")))
}

fn join(ctx: &Ctx, relative: &str) -> Result<url::Url, ContentError> {
    remote(ctx)?
        .join(relative)
        .map_err(|error| ContentError::BadGateway(error.to_string()))
}

async fn read(ctx: &Ctx, asset: &Asset) -> Result<Vec<u8>, ContentError> {
    if asset.size.max(0) as u64 > MAX_REVISION {
        return Err(ContentError::BadGateway(
            "the revision information is too large".to_owned(),
        ));
    }
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let Some(content) = store.read(asset.blob_id.into()).await? else {
        return Err(ContentError::not_found(asset.path.clone()));
    };
    let mut body = Vec::with_capacity(content.size as usize);
    let mut stream = content.stream;
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

fn query_flag(request: &ContentRequest, name: &str) -> bool {
    request.query.as_deref().is_some_and(|query| {
        url::form_urlencoded::parse(query.as_bytes())
            .any(|(key, value)| key == name && (value == "1" || value.eq_ignore_ascii_case("true")))
    })
}

impl HuggingFace {
    fn parsed(&self, asset: &Asset) -> Option<Arc<Revision>> {
        self.parsed
            .lock()
            .expect("the revision cache lock is never poisoned")
            .get(&format!(
                "{}{}@{}",
                asset.repository, asset.path, asset.blob_id
            ))
            .cloned()
    }

    fn remember(&self, asset: &Asset, revision: Arc<Revision>) {
        let mut parsed = self
            .parsed
            .lock()
            .expect("the revision cache lock is never poisoned");
        if parsed.len() > 64 {
            parsed.clear();
        }
        parsed.insert(
            format!("{}{}@{}", asset.repository, asset.path, asset.blob_id),
            revision,
        );
    }

    async fn revision(
        &self,
        ctx: &Ctx,
        kind: RepoType,
        repo: &str,
        name: &str,
    ) -> Result<Arc<Revision>, Failure> {
        let path = paths::revision_path(kind, repo, name);
        let immutable = is_commit(name);
        let cached = match immutable {
            true => store::find(ctx.db(), ctx.name(), &path).await?,
            false => None,
        };
        let asset = match cached {
            Some(asset) => asset,
            None => {
                let url = join(
                    ctx,
                    &format!(
                        "api/{}/{repo}/revision/{}?blobs=true",
                        kind.plural(),
                        utf8_percent_encode(name, SEGMENT)
                    ),
                )?;
                let found = proxy::fetch_from(
                    ctx,
                    &path,
                    Freshness::Metadata,
                    || async { Ok(Some(Origin::new(url))) },
                    || {
                        PutAsset::cached(&path)
                            .kind(REVISION)
                            .content_type("application/json")
                    },
                )
                .await?;
                found.ok_or_else(|| {
                    Failure::new(
                        StatusCode::NOT_FOUND,
                        Some("RepoNotFound"),
                        format!(
                            "{} {repo} has no revision {name} on the remote",
                            kind.kind()
                        ),
                    )
                })?
            }
        };
        if let Some(parsed) = self.parsed(&asset) {
            return Ok(parsed);
        }
        let parsed = Arc::new(
            Revision::parse(read(ctx, &asset).await?)
                .map_err(|message| Failure::new(StatusCode::BAD_GATEWAY, None, message))?,
        );
        self.remember(&asset, parsed.clone());
        if !immutable && let Some(sha) = parsed.sha.as_deref().filter(|sha| is_commit(sha)) {
            let commit_path = paths::revision_path(kind, repo, sha);
            if store::find(ctx.db(), ctx.name(), &commit_path)
                .await?
                .is_none()
            {
                let put = PutAsset::cached(&commit_path)
                    .kind(REVISION)
                    .content_type("application/json");
                store::put(ctx, put, astrum_blob::content_of(parsed.raw.clone())).await?;
            }
        }
        Ok(parsed)
    }

    async fn commit_of(
        &self,
        ctx: &Ctx,
        kind: RepoType,
        repo: &str,
        name: &str,
    ) -> Result<String, Failure> {
        if is_commit(name) {
            return Ok(name.to_owned());
        }
        self.revision(ctx, kind, repo, name)
            .await?
            .sha
            .clone()
            .filter(|sha| is_commit(sha))
            .ok_or_else(|| {
                Failure::new(
                    StatusCode::BAD_GATEWAY,
                    None,
                    format!("The remote answered no commit for {repo} at {name}"),
                )
            })
    }

    async fn revisions_of(
        ctx: &Ctx,
        kind: RepoType,
        repo: &str,
        commit: &str,
        name: &str,
    ) -> Result<Vec<String>, ContentError> {
        let mut revisions = BTreeSet::new();
        if name != commit {
            revisions.insert(name.to_owned());
        }
        let known = component::Entity::find()
            .filter(component::Column::Repository.eq(ctx.name()))
            .filter(component::Column::Namespace.eq(kind.plural()))
            .filter(component::Column::Name.eq(repo))
            .filter(component::Column::Version.eq(commit))
            .one(ctx.db())
            .await?;
        if let Some(known) = known {
            for revision in known.attributes[ID]["revisions"]
                .as_array()
                .into_iter()
                .flatten()
            {
                if let Some(revision) = revision.as_str() {
                    revisions.insert(revision.to_owned());
                }
            }
        }
        Ok(revisions.into_iter().collect())
    }

    #[allow(clippy::too_many_arguments)]
    async fn file(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        kind: RepoType,
        repo: &str,
        name: &str,
        file: &str,
    ) -> Result<Response, Failure> {
        let commit = self.commit_of(ctx, kind, repo, name).await?;
        let at_commit = self.revision(ctx, kind, repo, &commit).await?;
        let listed = at_commit.file(file).cloned();
        if listed.is_none() && at_commit.lists_files {
            return Err(Failure::new(
                StatusCode::NOT_FOUND,
                Some("EntryNotFound"),
                format!("{file} does not exist on {repo} at {commit}"),
            ));
        }
        let path = paths::file_path(kind, repo, &commit, file);
        ctx.require(View::Read, &path)?;
        let cached = store::find(ctx.db(), ctx.name(), &path).await?;
        if cached.is_none()
            && request.method == Method::HEAD
            && let Some(listed) = &listed
            && let Some(etag) = listed.etag()
            && listed.content_size() >= 0
        {
            return Ok(file_head(&commit, etag, listed.content_size()));
        }
        let asset = match cached {
            Some(asset) => asset,
            None => {
                self.fetch_file(ctx, kind, repo, name, &commit, file, listed.as_ref())
                    .await?
            }
        };
        let etag = listed
            .as_ref()
            .and_then(RepoFile::etag)
            .map(str::to_owned)
            .or_else(|| asset.attributes[ID]["etag"].as_str().map(str::to_owned))
            .unwrap_or_else(|| asset.sha1.clone());
        let mut response = serve(
            ctx,
            &asset,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await
        .map_err(Failure::from)?;
        let headers = response.headers_mut();
        let quoted = format!("\"{etag}\"");
        for (name, value) in [
            ("x-repo-commit", commit.clone()),
            ("x-linked-etag", quoted.clone()),
            ("etag", quoted),
            ("x-linked-size", asset.size.to_string()),
        ] {
            if let Ok(value) = HeaderValue::from_str(&value) {
                headers.insert(name, value);
            }
        }
        Ok(response)
    }

    #[allow(clippy::too_many_arguments)]
    async fn fetch_file(
        &self,
        ctx: &Ctx,
        kind: RepoType,
        repo: &str,
        name: &str,
        commit: &str,
        file: &str,
        listed: Option<&RepoFile>,
    ) -> Result<Asset, Failure> {
        let path = paths::file_path(kind, repo, commit, file);
        let url = join(
            ctx,
            &format!(
                "{}{repo}/resolve/{commit}/{}",
                kind.url_prefix(),
                encode_path(file)
            ),
        )?;
        let revisions = Self::revisions_of(ctx, kind, repo, commit, name).await?;
        let identity = json!({
            "repoId": repo,
            "repoType": kind.kind(),
            "commit": commit,
            "revisions": revisions,
        });
        let mut attributes = identity.clone();
        attributes["file"] = Value::String(file.to_owned());
        attributes["lfs"] = Value::Bool(listed.is_some_and(|listed| listed.lfs_sha256.is_some()));
        if let Some(etag) = listed.and_then(RepoFile::etag) {
            attributes["etag"] = Value::String(etag.to_owned());
        }
        let found = proxy::fetch_from(
            ctx,
            &path,
            Freshness::Content,
            || async { Ok(Some(Origin::new(url))) },
            || {
                PutAsset::cached(&path)
                    .kind(FILE)
                    .component(ComponentKey {
                        namespace: kind.plural().to_owned(),
                        name: repo.to_owned(),
                        version: commit.to_owned(),
                        kind: kind.kind().to_owned(),
                        attributes: json!({ ID: identity }),
                    })
                    .attributes(json!({ ID: attributes }))
            },
        )
        .await?
        .ok_or_else(|| {
            Failure::new(
                StatusCode::NOT_FOUND,
                Some("EntryNotFound"),
                format!("{file} is not available on the remote for {repo} at {commit}"),
            )
        })?;
        if let Some(listed) = listed {
            let wrong_hash = listed
                .lfs_sha256
                .as_deref()
                .is_some_and(|sha256| !found.sha256.eq_ignore_ascii_case(sha256));
            let size = listed.content_size();
            let wrong_size = size >= 0 && size != found.size;
            if wrong_hash || wrong_size {
                store::delete(ctx, &path).await?;
                return Err(Failure::new(
                    StatusCode::BAD_GATEWAY,
                    None,
                    format!("{file} of {repo} at {commit} does not match what the Hub announced"),
                ));
            }
        }
        Ok(found)
    }

    async fn requested_paths(request: &ContentRequest) -> Vec<String> {
        let mut paths: Vec<String> = request
            .query
            .as_deref()
            .map(|query| {
                url::form_urlencoded::parse(query.as_bytes())
                    .filter(|(name, _)| name == "paths")
                    .map(|(_, value)| value.into_owned())
                    .collect()
            })
            .unwrap_or_default();
        let json = request
            .header("content-type")
            .is_some_and(|value| value.starts_with("application/json"));
        let Ok(body) = request.bytes(MAX_FORM).await else {
            return paths;
        };
        if json {
            if let Ok(value) = serde_json::from_slice::<Value>(&body) {
                paths.extend(
                    value["paths"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter_map(|path| path.as_str().map(str::to_owned)),
                );
            }
        } else {
            paths.extend(
                url::form_urlencoded::parse(&body)
                    .filter(|(name, _)| name == "paths")
                    .map(|(_, value)| value.into_owned()),
            );
        }
        paths
    }

    async fn answer(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, Failure> {
        let Some(parsed) = paths::parse(&request.path) else {
            return Err(Failure::new(
                StatusCode::NOT_FOUND,
                None,
                format!("{} is not a Hugging Face Hub path", request.path),
            ));
        };
        match (&request.method, parsed) {
            (
                &Method::GET | &Method::HEAD,
                Request::Info {
                    kind,
                    repo,
                    revision,
                },
            ) => {
                ctx.require(View::Read, &request.path)?;
                let name = revision.unwrap_or_else(|| DEFAULT_REVISION.to_owned());
                let revision = self.revision(ctx, kind, &repo, &name).await?;
                let value: Value = serde_json::from_slice(&revision.raw).unwrap_or_default();
                Ok(json_response(request, &value))
            }
            (
                &Method::GET | &Method::HEAD,
                Request::Tree {
                    kind,
                    repo,
                    revision,
                    directory,
                },
            ) => {
                ctx.require(View::Read, &request.path)?;
                let found = self.revision(ctx, kind, &repo, &revision).await?;
                let tree = found
                    .tree(&directory, query_flag(request, "recursive"))
                    .ok_or_else(|| {
                        Failure::new(
                            StatusCode::NOT_FOUND,
                            Some("EntryNotFound"),
                            format!("{directory} is not a directory of {repo} at {revision}"),
                        )
                    })?;
                Ok(json_response(request, &tree))
            }
            (
                &Method::GET | &Method::POST,
                Request::PathsInfo {
                    kind,
                    repo,
                    revision,
                },
            ) => {
                ctx.require(View::Read, &request.path)?;
                let wanted = Self::requested_paths(request).await;
                let found = self.revision(ctx, kind, &repo, &revision).await?;
                Ok(json_response(request, &found.paths_info(&wanted)))
            }
            (
                &Method::GET | &Method::HEAD,
                Request::File {
                    kind,
                    repo,
                    revision,
                    file,
                },
            ) => self.file(ctx, request, kind, &repo, &revision, &file).await,
            (&Method::GET | &Method::HEAD, Request::Stored) => {
                ctx.require(View::Read, &request.path)?;
                let asset = store::find(ctx.db(), ctx.name(), &request.path)
                    .await?
                    .ok_or_else(|| Failure::new(StatusCode::NOT_FOUND, None, "Not found"))?;
                serve(
                    ctx,
                    &asset,
                    &request.method,
                    &request.headers,
                    Serve::download(),
                )
                .await
                .map_err(Failure::from)
            }
            (method, _) => Err(Failure::from(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: "GET,HEAD,DELETE",
            })),
        }
    }
}

fn file_head(commit: &str, etag: &str, size: i64) -> Response {
    let quoted = format!("\"{etag}\"");
    Response::builder()
        .status(StatusCode::OK)
        .header("x-repo-commit", commit)
        .header(header::ETAG, &quoted)
        .header("x-linked-etag", &quoted)
        .header("x-linked-size", size)
        .header(header::CONTENT_LENGTH, size)
        .header(header::ACCEPT_RANGES, "bytes")
        .body(Body::empty())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

#[async_trait]
impl Format for HuggingFace {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Proxy {
            return Err(ContentError::not_found(
                "Hugging Face repositories are proxies only",
            ));
        }
        if request.method == Method::DELETE {
            ctx.require(View::Delete, &request.path)?;
            return match store::delete(&ctx, &request.path).await? {
                true => Ok(StatusCode::NO_CONTENT.into_response()),
                false => Err(ContentError::not_found(request.path.clone())),
            };
        }
        match self.answer(&ctx, &request).await {
            Ok(response) => Ok(response),
            Err(failure)
                if matches!(
                    failure.status,
                    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN
                ) && failure.code.is_none() =>
            {
                Err(match failure.status {
                    StatusCode::UNAUTHORIZED => ContentError::Unauthorized,
                    _ => ContentError::Forbidden,
                })
            }
            Err(failure) => Ok(failure.reply()),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        store::find(ctx.db(), ctx.name(), path).await
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        &[&["huggingface", "revisions"]]
    }
}
