pub mod crx;
pub mod gupdate;
mod hosted;
pub mod manifest;
mod remote;

use std::collections::BTreeMap;

use astrum_content::format::{BrowseLayout, SearchKey, SearchTarget};
use astrum_content::store;
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Serve, UploadField, UploadForm, UploadSpec,
    View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use serde_json::json;

use crate::crx::{artifact_path, compare_versions, valid_id, valid_version};
use crate::gupdate::{Answer, Mode};
use crate::hosted::Refusal;

pub use crate::remote::DEFAULT_PRODVERSION;

pub const ID: &str = "chrome";
pub const ARTIFACT: &str = "extension";
pub const CRX_TYPE: &str = "application/x-chrome-extension";
pub const UPDATES: &str = "updates.xml";
const XML_TYPE: &str = "text/xml; charset=UTF-8";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "chrome.name",
        target: SearchTarget::Component,
        pointer: &["chrome", "name"],
    },
    SearchKey {
        param: "chrome.manifestVersion",
        target: SearchTarget::Component,
        pointer: &["chrome", "manifestVersion"],
    },
];

const KEYWORDS: &[&[&str]] = &[&["chrome", "name"], &["chrome", "description"]];

#[derive(Default)]
pub struct Chrome {
    remote: remote::Remote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Root,
    Updates,
    Latest(String),
    Artifact(String, String),
    Package,
    Other,
}

fn route(path: &str) -> Route {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Route::Root;
    }
    let directory = path.ends_with('/');
    let parts: Vec<&str> = trimmed.split('/').collect();
    let found = match parts.as_slice() {
        _ if directory => None,
        [file] if *file == UPDATES => Some(Route::Updates),
        [id, "latest.crx"] if valid_id(id) => Some(Route::Latest((*id).to_owned())),
        [id, version, file]
            if valid_id(id) && valid_version(version) && *file == format!("{id}-{version}.crx") =>
        {
            Some(Route::Artifact((*id).to_owned(), (*version).to_owned()))
        }
        _ => None,
    };
    match found {
        Some(found) => found,
        None if !directory && trimmed.to_ascii_lowercase().ends_with(".crx") => Route::Package,
        None => Route::Other,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Offered {
    version: String,
    sha256: Option<String>,
    size: Option<u64>,
}

fn denied(ctx: &Ctx) -> ContentError {
    match &ctx.subject {
        Some(subject) if !subject.anonymous => ContentError::Forbidden,
        _ => ContentError::Unauthorized,
    }
}

fn require_reader(ctx: &Ctx) -> Result<(), ContentError> {
    match ctx.grant(View::Read).any() {
        true => Ok(()),
        false => Err(denied(ctx)),
    }
}

fn log_member_failure(ctx: &Ctx, member: &Ctx, error: &ContentError) {
    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; trying the next");
}

fn escape(text: &str) -> String {
    gupdate::escape(text)
}

fn text_response(
    request: &ContentRequest,
    content_type: &'static str,
    body: String,
) -> Result<Response, ContentError> {
    let length = body.len();
    let body = match request.method {
        Method::HEAD => Body::empty(),
        _ => Body::from(body),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, content_type)
        .header(header::CONTENT_LENGTH, length)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(body)
        .map_err(ContentError::internal)
}

fn json_response(status: StatusCode, value: &serde_json::Value) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(value.to_string()))
        .expect("a JSON answer is valid")
}

fn directory_page(ctx: &Ctx) -> Result<Response, ContentError> {
    let name = escape(ctx.name());
    let url = escape(&ctx.repository_url());
    let body = format!(
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a repository of extensions for Chromium browsers such as Google Chrome, Microsoft Edge and Brave. Use <code>{url}/{UPDATES}</code> as the update URL of an extension in the <code>ExtensionSettings</code> or <code>ExtensionInstallForcelist</code> policy; <code>{url}/&lt;extension id&gt;/latest.crx</code> downloads the newest version.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

fn attachment(mut response: Response, asset: &Asset) -> Response {
    let name: String = asset
        .path
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        .collect();
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    response
}

fn redirect(ctx: &Ctx, id: &str, version: &str) -> Result<Response, ContentError> {
    let path = artifact_path(id, version);
    ctx.require(View::Read, &path)?;
    Response::builder()
        .status(StatusCode::FOUND)
        .header(header::LOCATION, format!("{}{path}", ctx.repository_url()))
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::empty())
        .map_err(ContentError::internal)
}

impl Chrome {
    async fn offers(
        &self,
        ctx: &Ctx,
        ids: &[String],
        prodversion: Option<&str>,
    ) -> Result<BTreeMap<String, Offered>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => Ok(hosted::newest(ctx, ids)
                .await?
                .into_iter()
                .map(|(id, stored)| {
                    let offered = Offered {
                        version: stored.record.version,
                        sha256: Some(stored.asset.sha256),
                        size: u64::try_from(stored.asset.size).ok(),
                    };
                    (id, offered)
                })
                .collect()),
            RepositoryType::Proxy => {
                let mut found = BTreeMap::new();
                for (id, offer) in self.remote.lookup(ctx, ids, prodversion).await? {
                    let Some((version, codebase)) = offer else {
                        continue;
                    };
                    let cached =
                        store::find(ctx.db(), ctx.name(), &artifact_path(&id, &version)).await?;
                    let offered = match cached {
                        Some(asset) => Offered {
                            version,
                            sha256: Some(asset.sha256),
                            size: u64::try_from(asset.size).ok(),
                        },
                        None => Offered {
                            version,
                            sha256: codebase.sha256,
                            size: codebase.size,
                        },
                    };
                    found.insert(id, offered);
                }
                Ok(found)
            }
            RepositoryType::Group => {
                let mut found = BTreeMap::new();
                let mut remaining = ids.to_vec();
                for member in group::members(ctx).await? {
                    if remaining.is_empty() {
                        break;
                    }
                    let member = ctx.member(member);
                    match Box::pin(self.offers(&member, &remaining, prodversion)).await {
                        Ok(offers) => found.extend(offers),
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                    remaining.retain(|id| !found.contains_key(id));
                }
                Ok(found)
            }
        }
    }

    async fn visible_offers(
        &self,
        ctx: &Ctx,
        ids: &[String],
        prodversion: Option<&str>,
    ) -> Result<BTreeMap<String, Offered>, ContentError> {
        let mut offers = self.offers(ctx, ids, prodversion).await?;
        offers.retain(|id, offered| ctx.allowed(View::Read, &artifact_path(id, &offered.version)));
        Ok(offers)
    }

    async fn artifact(
        &self,
        ctx: &Ctx,
        id: &str,
        version: &str,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                Ok(
                    store::find(ctx.db(), ctx.name(), &artifact_path(id, version))
                        .await?
                        .map(|found| (ctx.clone(), found)),
                )
            }
            RepositoryType::Proxy => Ok(self
                .remote
                .fetch(ctx, id, version)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.artifact(&member, id, version)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn updates(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        require_reader(ctx)?;
        let parsed =
            gupdate::parse_request(request.query.as_deref()).map_err(ContentError::BadRequest)?;
        let ids: Vec<String> = parsed
            .wanted
            .iter()
            .map(|wanted| wanted.id.clone())
            .collect();
        if parsed.mode == Mode::Redirect {
            let first = &ids[..1];
            let offers = self
                .visible_offers(ctx, first, parsed.prodversion.as_deref())
                .await?;
            let offered = offers.get(&first[0]).ok_or_else(|| {
                ContentError::not_found(format!("{} has no version of {}", ctx.name(), first[0]))
            })?;
            return redirect(ctx, &first[0], &offered.version);
        }
        let offers = self
            .visible_offers(ctx, &ids, parsed.prodversion.as_deref())
            .await?;
        let base = ctx.repository_url();
        let answers: Vec<(String, Answer)> = parsed
            .wanted
            .iter()
            .map(|wanted| {
                let answer = match offers.get(&wanted.id) {
                    None => Answer::Unknown,
                    Some(offered)
                        if wanted.version.as_deref().is_some_and(|have| {
                            compare_versions(have, &offered.version).is_ge()
                        }) =>
                    {
                        Answer::NoUpdate
                    }
                    Some(offered) => Answer::Update {
                        codebase: format!("{base}{}", artifact_path(&wanted.id, &offered.version)),
                        version: offered.version.clone(),
                        sha256: offered.sha256.clone(),
                        size: offered.size,
                    },
                };
                (wanted.id.clone(), answer)
            })
            .collect();
        text_response(request, XML_TYPE, gupdate::document(&answers, Utc::now()))
    }

    async fn latest(&self, ctx: &Ctx, id: &str) -> Result<Response, ContentError> {
        require_reader(ctx)?;
        let wanted = [id.to_owned()];
        let offers = self.visible_offers(ctx, &wanted, None).await?;
        let offered = offers.get(id).ok_or_else(|| {
            ContentError::not_found(format!("{} has no version of {id}", ctx.name()))
        })?;
        redirect(ctx, id, &offered.version)
    }

    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        id: &str,
        version: &str,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &request.path)?;
        let (owner, asset) = self
            .artifact(ctx, id, version)
            .await?
            .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
        let response = serve(
            &owner,
            &asset,
            &request.method,
            &request.headers,
            Serve::download().disposition("attachment"),
        )
        .await?;
        Ok(attachment(response, &asset))
    }

    async fn read_other(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        let path = request.path.as_str();
        if request.is_directory() {
            return Err(ContentError::not_found(path.to_owned()));
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

    fn require_hosted(&self, ctx: &Ctx) -> Result<(), ContentError> {
        if ctx.kind() != RepositoryType::Hosted {
            return Err(ContentError::BadRequest(format!(
                "{} is a {} repository; publish extensions to a hosted repository",
                ctx.name(),
                ctx.kind()
            )));
        }
        if !hosted::may_write(ctx) {
            return Err(denied(ctx));
        }
        Ok(())
    }

    async fn put(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        expected: Option<String>,
    ) -> Result<Response, ContentError> {
        self.require_hosted(ctx)?;
        let written = store::write_blob(ctx, request.into_stream()).await?;
        let (package, stored) = hosted::store_package(ctx, written, expected.as_deref())
            .await
            .map_err(Refusal::into_error)?;
        let location = format!("{}{}", ctx.repository_url(), stored.path);
        let mut response = json_response(
            StatusCode::CREATED,
            &json!({
                "id": package.id,
                "name": package.manifest.name,
                "version": package.manifest.version,
                "manifestVersion": package.manifest.manifest_version,
                "url": location,
            }),
        );
        if let Ok(value) = HeaderValue::from_str(&location) {
            response.headers_mut().insert(header::LOCATION, value);
        }
        Ok(response)
    }

    async fn delete(&self, ctx: &Ctx, path: &str) -> Result<Response, ContentError> {
        ctx.require(View::Delete, path)?;
        match store::delete(ctx, path).await? {
            true => Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty())
                .map_err(ContentError::internal),
            false => Err(ContentError::not_found(format!("{path} cannot be found"))),
        }
    }

    fn not_allowed(method: &Method, kind: RepositoryType) -> ContentError {
        ContentError::MethodNotAllowed {
            method: method.to_string(),
            allow: match kind {
                RepositoryType::Hosted => "GET,HEAD,PUT,DELETE",
                RepositoryType::Proxy => "GET,HEAD,DELETE",
                RepositoryType::Group => "GET,HEAD",
            },
        }
    }
}

#[async_trait]
impl Format for Chrome {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        let method = request.method.clone();
        let kind = ctx.kind();
        let reading = matches!(method, Method::GET | Method::HEAD);
        let result = match (&method, &route) {
            (_, Route::Root) if reading && gupdate::is_update_request(request.query.as_deref()) => {
                self.updates(&ctx, &request).await
            }
            (_, Route::Root) if reading => directory_page(&ctx),
            (_, Route::Updates) if reading => self.updates(&ctx, &request).await,
            (_, Route::Latest(id)) if reading => self.latest(&ctx, id).await,
            (_, Route::Artifact(id, version)) if reading => {
                self.download(&ctx, &request, id, version).await
            }
            (_, Route::Package | Route::Other) if reading => self.read_other(&ctx, &request).await,
            (&Method::PUT, Route::Artifact(..)) => {
                let expected = request.path.clone();
                self.put(&ctx, request, Some(expected)).await
            }
            (&Method::PUT, Route::Package | Route::Latest(_)) => {
                self.put(&ctx, request, None).await
            }
            (&Method::DELETE, Route::Artifact(..) | Route::Package | Route::Other)
                if kind != RepositoryType::Group && !request.is_directory() =>
            {
                let path = request.path.clone();
                self.delete(&ctx, &path).await
            }
            (method, _) => Err(Self::not_allowed(method, kind)),
        };
        result.or_else(|error| Ok(error.into_response()))
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        match route(path) {
            Route::Artifact(id, version) => Ok(self
                .artifact(ctx, &id, &version)
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
            component_fields: Vec::new(),
            asset_fields: vec![UploadField::file("asset")],
        })
    }

    async fn upload(&self, ctx: Ctx, form: UploadForm) -> Result<(), ContentError> {
        self.require_hosted(&ctx)?;
        let file = form
            .files
            .get(&format!("{ID}.asset"))
            .or_else(|| form.files.get("asset"))
            .ok_or_else(|| ContentError::BadRequest(format!("{ID}.asset is required")))?;
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        hosted::store_package(&ctx, written, None)
            .await
            .map(|_| ())
            .map_err(Refusal::into_error)
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }

    fn keyword_attributes(&self) -> &'static [&'static [&'static str]] {
        KEYWORDS
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some("CASE WHEN a.kind = 'extension' THEN a.path END"),
            generated: &[],
        }
    }
}

pub mod testing {
    use astrum_content::zip;
    use sha2::{Digest, Sha256};

    fn varint(mut value: u64, out: &mut Vec<u8>) {
        while value >= 0x80 {
            out.push((value as u8) | 0x80);
            value >>= 7;
        }
        out.push(value as u8);
    }

    fn field(number: u64, data: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        varint((number << 3) | 2, &mut out);
        varint(data.len() as u64, &mut out);
        out.extend_from_slice(data);
        out
    }

    pub fn id(key: &[u8]) -> String {
        crate::crx::id_of_key(key)
    }

    pub fn header(key: &[u8], rsa_keys: &[&[u8]]) -> Vec<u8> {
        let crx_id = &Sha256::digest(key)[..16];
        let mut header = Vec::new();
        for rsa in rsa_keys {
            let mut proof = field(1, rsa);
            proof.extend(field(2, b"signature bytes"));
            header.extend(field(2, &proof));
        }
        header.extend(field(10000, &field(1, crx_id)));
        header
    }

    pub fn manifest(name: &str, version: &str) -> String {
        format!(
            "{{\n  \"manifest_version\": 3,\n  \"name\": \"{name}\",\n  \"description\": \"The {name} extension.\",\n  \"version\": \"{version}\"\n}}\n"
        )
    }

    pub fn crx(key: &[u8], manifest: &str, files: &[(&str, &[u8])]) -> Vec<u8> {
        let mut entries: Vec<(&str, &[u8])> = vec![("manifest.json", manifest.as_bytes())];
        entries.extend_from_slice(files);
        let archive = zip::write(&entries);
        let header = header(key, &[key]);
        let mut out = b"Cr24".to_vec();
        out.extend_from_slice(&3u32.to_le_bytes());
        out.extend_from_slice(&(header.len() as u32).to_le_bytes());
        out.extend_from_slice(&header);
        out.extend_from_slice(&archive);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ID_A: &str = "abcdefghijklmnopabcdefghijklmnop";

    #[test]
    fn paths_route_to_update_checks_and_packages() {
        assert_eq!(route("/"), Route::Root);
        assert_eq!(route("/updates.xml"), Route::Updates);
        assert_eq!(route("/updates.xml/"), Route::Other);
        assert_eq!(
            route(&format!("/{ID_A}/latest.crx")),
            Route::Latest(ID_A.to_owned())
        );
        assert_eq!(
            route(&format!("/{ID_A}/1.2.3/{ID_A}-1.2.3.crx")),
            Route::Artifact(ID_A.to_owned(), "1.2.3".to_owned())
        );
        assert_eq!(
            route(&format!("/{ID_A}/1.2.3/{ID_A}-1.2.4.crx")),
            Route::Package
        );
        assert_eq!(route("/upload/extension.CRX"), Route::Package);
        assert_eq!(route("/zzzz/latest.crx"), Route::Package);
        assert_eq!(route(&format!("/{ID_A}/updatecheck.json")), Route::Other);
        assert_eq!(route(&format!("/{ID_A}/")), Route::Other);
    }
}
