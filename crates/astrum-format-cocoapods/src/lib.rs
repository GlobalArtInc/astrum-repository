pub mod podspec;

use std::sync::Mutex;

use astrum_content::proxy::{self, Origin};
use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{
    Asset, BrowseLayout, ContentError, ContentRequest, Ctx, Format, Freshness, Serve, View, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderName, Method, StatusCode, header};
use axum::response::Response;
use futures_util::TryStreamExt;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

pub const ID: &str = "cocoapods";
pub const METADATA: &str = "metadata";
pub const PODSPEC: &str = "podspec";
pub const POD: &str = "pod";
pub const VERSION_FILE: &str = "/CocoaPods-version.yml";
const JSON_TYPE: &str = "application/json";
const MAX_PODSPEC: u64 = 8 * 1024 * 1024;
const MAX_VERSION_FILE: u64 = 64 * 1024;

#[derive(Default)]
pub struct CocoaPods;

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Index,
    Podspec {
        pod: String,
        version: String,
    },
    Archive {
        pod: String,
        version: String,
        file: String,
    },
    Directory,
    Other,
}

fn is_index(name: &str) -> bool {
    if matches!(
        name,
        "CocoaPods-version.yml"
            | "all_pods.txt"
            | "deprecated_podspecs.txt"
            | "all_pods_versions.txt"
    ) {
        return true;
    }
    name.strip_prefix("all_pods_versions_")
        .and_then(|rest| rest.strip_suffix(".txt"))
        .is_some_and(|prefixes| prefixes.split('_').all(podspec::valid_prefix))
}

fn route(path: &str) -> Route {
    if path.ends_with('/') {
        return Route::Directory;
    }
    let parts: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    match parts.as_slice() {
        [name] if is_index(name) => Route::Index,
        ["pods", pod, version, file]
            if podspec::valid_name(pod)
                && podspec::valid_version(version)
                && podspec::valid_archive_name(file) =>
        {
            Route::Archive {
                pod: (*pod).to_owned(),
                version: (*version).to_owned(),
                file: (*file).to_owned(),
            }
        }
        ["Specs", rest @ ..] if rest.len() >= 3 => {
            let (prefixes, tail) = rest.split_at(rest.len() - 3);
            let [pod, version, file] = tail else {
                return Route::Other;
            };
            if prefixes.iter().all(|prefix| podspec::valid_prefix(prefix))
                && podspec::valid_name(pod)
                && podspec::valid_version(version)
                && *file == format!("{pod}.podspec.json")
            {
                Route::Podspec {
                    pod: (*pod).to_owned(),
                    version: (*version).to_owned(),
                }
            } else {
                Route::Other
            }
        }
        _ => Route::Other,
    }
}

fn component(pod: &str, version: &str) -> ComponentKey {
    ComponentKey {
        namespace: String::new(),
        name: pod.to_owned(),
        version: version.to_owned(),
        kind: String::new(),
        attributes: json!({}),
    }
}

async fn read(ctx: &Ctx, found: &Asset, limit: u64) -> Result<Vec<u8>, ContentError> {
    if found.size.max(0) as u64 > limit {
        return Err(ContentError::BadGateway(format!(
            "{} is too large",
            found.path
        )));
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a CocoaPods CDN proxy. Add it with <code>pod repo add-cdn {name} {url}/</code> or put <code>source '{url}/'</code> into the Podfile.</p>\n    <p>The files cached by this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn text_response(request: &ContentRequest, body: String, content_type: &str) -> Response {
    let etag = format!(
        "\"{}\"",
        hex::encode(&Sha256::digest(body.as_bytes())[..16])
    );
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
        .expect("a text response is valid")
}

pub fn proxied_location(repository_url: &str, pod: &str, version: &str, url: &url::Url) -> String {
    format!(
        "{repository_url}{}",
        podspec::archive_location(pod, version, &podspec::archive_name(url))
    )
}

pub fn rewrite_podspec(
    bytes: &[u8],
    repository_url: &str,
    pod: &str,
    version: &str,
) -> Option<String> {
    let text = std::str::from_utf8(bytes).ok()?;
    let spec: Value = serde_json::from_str(text).ok()?;
    let source = podspec::http_source(&spec)?;
    let location = proxied_location(repository_url, pod, version, &source.url);
    podspec::rewrite(text, &spec, &location)
}

impl CocoaPods {
    async fn index(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        proxy::fetch(ctx, path, None, Freshness::Metadata, || {
            PutAsset::cached(path).kind(METADATA)
        })
        .await
    }

    async fn prefix_lengths(&self, ctx: &Ctx) -> Vec<usize> {
        let found = match store::find(ctx.db(), ctx.name(), VERSION_FILE).await {
            Ok(Some(found)) => Some(found),
            _ => self.index(ctx, VERSION_FILE).await.ok().flatten(),
        };
        let lengths = match found {
            Some(found) => read(ctx, &found, MAX_VERSION_FILE)
                .await
                .ok()
                .and_then(|bytes| podspec::prefix_lengths(&bytes)),
            None => None,
        };
        lengths.unwrap_or_else(|| podspec::DEFAULT_PREFIX_LENGTHS.to_vec())
    }

    async fn podspec(
        &self,
        ctx: &Ctx,
        path: &str,
        pod: &str,
        version: &str,
    ) -> Result<Option<Asset>, ContentError> {
        proxy::fetch(ctx, path, None, Freshness::Metadata, || {
            PutAsset::cached(path)
                .kind(PODSPEC)
                .content_type(JSON_TYPE)
                .component(component(pod, version))
        })
        .await
    }

    async fn archive(
        &self,
        ctx: &Ctx,
        pod: &str,
        version: &str,
        file: &str,
    ) -> Result<Option<Asset>, ContentError> {
        let path = podspec::archive_path(pod, version, file);
        let original: Mutex<Option<String>> = Mutex::new(None);
        proxy::fetch_from(
            ctx,
            &path,
            Freshness::Content,
            || async {
                let lengths = self.prefix_lengths(ctx).await;
                let spec_path = podspec::podspec_path(pod, version, &lengths);
                let Some(found) = self.podspec(ctx, &spec_path, pod, version).await? else {
                    return Ok(None);
                };
                let bytes = read(ctx, &found, MAX_PODSPEC).await?;
                let Some(source) = serde_json::from_slice::<Value>(&bytes)
                    .ok()
                    .and_then(|spec| podspec::http_source(&spec))
                else {
                    return Ok(None);
                };
                if podspec::archive_name(&source.url) != file {
                    return Ok(None);
                }
                *original.lock().expect("the source lock is never poisoned") =
                    Some(source.url.to_string());
                let mut origin = Origin::new(source.url);
                for (name, value) in source.headers {
                    if let Ok(name) = HeaderName::try_from(name.as_str())
                        && name != header::AUTHORIZATION
                        && name != header::HOST
                    {
                        origin = origin.header(name, value);
                    }
                }
                Ok(Some(origin))
            },
            || {
                let original = original
                    .lock()
                    .expect("the source lock is never poisoned")
                    .clone();
                PutAsset::cached(path.clone())
                    .kind(POD)
                    .component(component(pod, version))
                    .attributes(json!({ ID: { "sourceUrl": original } }))
            },
        )
        .await
    }

    async fn read(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        let route = route(path);
        if route == Route::Directory {
            return directory_page(ctx);
        }
        ctx.require(View::Read, path)?;
        match route {
            Route::Index => {
                let found = self
                    .index(ctx, path)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
                serve(
                    ctx,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::default(),
                )
                .await
            }
            Route::Podspec { pod, version } => {
                let found = self
                    .podspec(ctx, path, &pod, &version)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
                let bytes = read(ctx, &found, MAX_PODSPEC).await?;
                match rewrite_podspec(&bytes, &ctx.repository_url(), &pod, &version) {
                    Some(body) => Ok(text_response(request, body, JSON_TYPE)),
                    None => {
                        serve(
                            ctx,
                            &found,
                            &request.method,
                            &request.headers,
                            Serve::default(),
                        )
                        .await
                    }
                }
            }
            Route::Archive { pod, version, file } => {
                let found = self
                    .archive(ctx, &pod, &version, &file)
                    .await?
                    .ok_or_else(|| ContentError::not_found(path.to_owned()))?;
                serve(
                    ctx,
                    &found,
                    &request.method,
                    &request.headers,
                    Serve::download(),
                )
                .await
            }
            Route::Directory | Route::Other => Err(ContentError::not_found(path.to_owned())),
        }
    }
}

#[async_trait]
impl Format for CocoaPods {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        if ctx.kind() != RepositoryType::Proxy {
            return Err(ContentError::not_found(
                "CocoaPods repositories are proxies only",
            ));
        }
        match request.method {
            Method::GET | Method::HEAD => self.read(&ctx, &request).await,
            Method::DELETE => {
                ctx.require(View::Delete, &request.path)?;
                if !store::delete(&ctx, &request.path).await? {
                    return Err(ContentError::not_found(request.path.clone()));
                }
                Response::builder()
                    .status(StatusCode::NO_CONTENT)
                    .body(Body::empty())
                    .map_err(ContentError::internal)
            }
            ref method => Err(ContentError::MethodNotAllowed {
                method: method.to_string(),
                allow: "GET,HEAD,DELETE",
            }),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        store::find(ctx.db(), ctx.name(), path).await
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NULL THEN a.path ELSE '/' || c.name || '/' || c.version || regexp_replace(a.path, '^.*/', '/') END",
            ),
            generated: &[],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_indexes_podspecs_and_archives() {
        for index in [
            "/CocoaPods-version.yml",
            "/all_pods.txt",
            "/deprecated_podspecs.txt",
            "/all_pods_versions.txt",
            "/all_pods_versions_a_7_5.txt",
            "/all_pods_versions_a7_5.txt",
        ] {
            assert_eq!(route(index), Route::Index, "{index}");
        }
        assert_eq!(
            route("/Specs/a/7/5/AFNetworking/4.0.1/AFNetworking.podspec.json"),
            Route::Podspec {
                pod: "AFNetworking".to_owned(),
                version: "4.0.1".to_owned()
            }
        );
        assert_eq!(
            route("/Specs/Kit/1.0/Kit.podspec.json"),
            Route::Podspec {
                pod: "Kit".to_owned(),
                version: "1.0".to_owned()
            }
        );
        assert_eq!(
            route("/pods/AFNetworking/4.0.1/4.0.1.tar.gz"),
            Route::Archive {
                pod: "AFNetworking".to_owned(),
                version: "4.0.1".to_owned(),
                file: "4.0.1.tar.gz".to_owned()
            }
        );
        assert_eq!(route("/"), Route::Directory);
        assert_eq!(route("/Specs/a/"), Route::Directory);
        for other in [
            "/all_pods_versions_g.txt",
            "/all_pods_versions_.txt",
            "/Specs/a/7/5/AFNetworking/4.0.1/Other.podspec.json",
            "/Specs/A/7/5/AFNetworking/4.0.1/AFNetworking.podspec.json",
            "/Specs/a/7/5/AFNetworking/4.0.1/AFNetworking.podspec",
            "/pods/AFNetworking/4.0.1",
            "/pods/AFNetworking/4.0.1/.hidden",
            "/other.txt",
            "/a/all_pods.txt",
        ] {
            assert_eq!(route(other), Route::Other, "{other}");
        }
    }

    #[test]
    fn http_sources_point_at_the_proxy() {
        let spec = br#"{"name":"Kit","version":"1.0","source":{"http":"https://github.com/acme/kit/releases/download/1.0/Kit.zip","sha256":"ab"}}"#;
        let rewritten =
            rewrite_podspec(spec, "http://astrum/repository/trunk", "Kit", "1.0").unwrap();
        let document: Value = serde_json::from_str(&rewritten).unwrap();
        assert_eq!(
            document["source"],
            json!({"http": "http://astrum/repository/trunk/pods/Kit/1.0/Kit.zip", "sha256": "ab"})
        );
        let git = br#"{"name":"Kit","version":"1.0","source":{"git":"https://github.com/acme/kit.git","tag":"1.0"}}"#;
        assert_eq!(
            rewrite_podspec(git, "http://astrum/repository/trunk", "Kit", "1.0"),
            None
        );
        assert_eq!(
            rewrite_podspec(b"not json", "http://astrum", "Kit", "1.0"),
            None
        );
    }
}
