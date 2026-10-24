pub mod addon;
mod hosted;
mod remote;
pub mod updates;
pub mod version;

use astrum_content::format::{BrowseLayout, SearchKey, SearchTarget};
use astrum_content::store;
use astrum_content::{
    Asset, ContentError, ContentRequest, Ctx, Format, Grant, Serve, UploadField, UploadForm,
    UploadSpec, View, group, serve,
};
use astrum_repo::RepositoryType;
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde_json::{Value, json};
use url::form_urlencoded;

use crate::addon::{addon_path, file_name, valid_id, valid_key, valid_version};
use crate::hosted::Refusal;
use crate::updates::Updates;

pub const ID: &str = "firefox";
pub const ADDON: &str = "addon";
pub const METADATA: &str = "metadata";
pub const XPI_TYPE: &str = "application/x-xpinstall";
const JSON_TYPE: &str = "application/json";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "firefox.name",
        target: SearchTarget::Component,
        pointer: &["firefox", "name"],
    },
    SearchKey {
        param: "firefox.signed",
        target: SearchTarget::Component,
        pointer: &["firefox", "signed"],
    },
    SearchKey {
        param: "firefox.minVersion",
        target: SearchTarget::Component,
        pointer: &["firefox", "minVersion"],
    },
];

const KEYWORDS: &[&[&str]] = &[&["firefox", "name"]];

#[derive(Default)]
pub struct Firefox {
    remote: remote::Remote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Root,
    Updates,
    VersionCheck,
    AddonUpdates(String),
    Latest(String),
    VersionFile(String, String),
    Artifact(String, String),
    Archive,
    Other,
}

fn route(path: &str) -> Route {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Route::Root;
    }
    let directory = path.ends_with('/');
    let parts: Vec<&str> = trimmed.split('/').collect();
    let found = match (directory, parts.as_slice()) {
        (false, ["updates.json"]) => Some(Route::Updates),
        (false, ["update", "VersionCheck.php"]) => Some(Route::VersionCheck),
        (false, [key, "updates.json"]) if valid_key(key) => {
            Some(Route::AddonUpdates((*key).to_owned()))
        }
        (false, [key, "latest.xpi"]) if valid_key(key) => Some(Route::Latest((*key).to_owned())),
        (false, [key, file]) if valid_key(key) => file
            .strip_suffix(".xpi")
            .filter(|version| valid_version(version))
            .map(|version| Route::VersionFile((*key).to_owned(), version.to_owned())),
        (false, [id, version, file])
            if valid_id(id) && valid_version(version) && *file == file_name(id, version) =>
        {
            Some(Route::Artifact((*id).to_owned(), (*version).to_owned()))
        }
        _ => None,
    };
    found.unwrap_or_else(
        || match !directory && trimmed.to_ascii_lowercase().ends_with(".xpi") {
            true => Route::Archive,
            false => Route::Other,
        },
    )
}

fn parameter(query: Option<&str>, name: &str) -> Option<String> {
    form_urlencoded::parse(query.unwrap_or_default().as_bytes())
        .find(|(key, value)| key == name && !value.trim().is_empty())
        .map(|(_, value)| value.trim().to_owned())
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

fn json_response(status: StatusCode, value: &Value) -> Response {
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, JSON_TYPE)
        .body(Body::from(value.to_string()))
        .expect("a JSON answer is valid")
}

fn manifest_response(request: &ContentRequest, value: &Value) -> Result<Response, ContentError> {
    let body = value.to_string();
    let length = body.len();
    let body = match request.method {
        Method::HEAD => Body::empty(),
        _ => Body::from(body),
    };
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, JSON_TYPE)
        .header(header::CONTENT_LENGTH, length)
        .header(header::CACHE_CONTROL, "no-cache")
        .body(body)
        .map_err(ContentError::internal)
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a repository of Firefox add-ons. Install one through the <code>ExtensionSettings</code> policy with <code>{url}/&lt;add-on id&gt;/latest.xpi</code> as its <code>install_url</code>. Add-ons find their updates at <code>{url}/&lt;add-on id&gt;/updates.json</code>, and the preference <code>extensions.update.url</code> can point at <code>{url}/update/VersionCheck.php?id=%ITEM_ID%</code>.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
        escape(&ctx.base_url),
    );
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
        .body(Body::from(body))
        .map_err(ContentError::internal)
}

impl Firefox {
    async fn collect(
        &self,
        ctx: &Ctx,
        id: Option<&str>,
        base: &str,
    ) -> Result<Updates, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => match id {
                Some(id) if !valid_id(id) => Ok(Updates::new()),
                _ => Ok(hosted::updates(&hosted::stored(ctx, id).await?, base)),
            },
            RepositoryType::Proxy => match id {
                Some(id) => Ok(self
                    .remote
                    .updates(ctx, id, base)
                    .await?
                    .map(|(guid, list)| Updates::from([(guid, list)]))
                    .unwrap_or_default()),
                None => Ok(Updates::new()),
            },
            RepositoryType::Group => {
                let mut lists = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.collect(&member, id, base)).await {
                        Ok(found) => {
                            let known = id.is_some() && found.values().any(|list| !list.is_empty());
                            lists.push(found);
                            if known {
                                break;
                            }
                        }
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(updates::merge(lists))
            }
        }
    }

    async fn manifest(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        id: Option<String>,
    ) -> Result<Response, ContentError> {
        require_reader(ctx)?;
        if ctx.kind() == RepositoryType::Proxy && id.is_none() {
            return Err(ContentError::not_found(
                "a proxy repository answers update manifests for one add-on at a time: ask for <add-on id>/updates.json or updates.json?id=<add-on id>",
            ));
        }
        let mut found = self
            .collect(ctx, id.as_deref(), &ctx.repository_url())
            .await?;
        if !matches!(ctx.grant(View::Read), Grant::Whole) {
            for (id, list) in found.iter_mut() {
                list.retain(|update| ctx.allowed(View::Read, &addon_path(id, &update.version)));
            }
            found.retain(|_, list| !list.is_empty());
        }
        updates::sort(&mut found);
        manifest_response(request, &updates::document(&found))
    }

    async fn version_check(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        let id = parameter(request.query.as_deref(), "id").ok_or_else(|| {
            ContentError::BadRequest("the id of the add-on is missing".to_owned())
        })?;
        self.manifest(ctx, request, Some(id)).await
    }

    async fn latest(&self, ctx: &Ctx, key: &str) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                if !valid_id(key) {
                    return Ok(None);
                }
                Ok(hosted::newest(hosted::stored(ctx, Some(key)).await?)
                    .map(|found| (ctx.clone(), found.asset)))
            }
            RepositoryType::Proxy => Ok(self
                .remote
                .latest(ctx, key)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.latest(&member, key)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn version_file(
        &self,
        ctx: &Ctx,
        key: &str,
        version: &str,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                if !valid_id(key) {
                    return Ok(None);
                }
                Ok(store::find(ctx.db(), ctx.name(), &addon_path(key, version))
                    .await?
                    .map(|found| (ctx.clone(), found)))
            }
            RepositoryType::Proxy => Ok(self
                .remote
                .version(ctx, key, version)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.version_file(&member, key, version)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn send(
        &self,
        owner: &Ctx,
        ctx: &Ctx,
        asset: &Asset,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &asset.path)?;
        serve(
            owner,
            asset,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await
    }

    async fn download_latest(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        key: &str,
    ) -> Result<Response, ContentError> {
        require_reader(ctx)?;
        let (owner, asset) = self.latest(ctx, key).await?.ok_or_else(|| {
            ContentError::not_found(format!("the add-on {key} is not in {}", ctx.name()))
        })?;
        self.send(&owner, ctx, &asset, request).await
    }

    async fn download_version(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        key: &str,
        version: &str,
    ) -> Result<Response, ContentError> {
        match valid_id(key) {
            true => ctx.require(View::Read, &addon_path(key, version))?,
            false => require_reader(ctx)?,
        }
        let (owner, asset) = self.version_file(ctx, key, version).await?.ok_or_else(|| {
            ContentError::not_found(format!(
                "version {version} of the add-on {key} is not in {}",
                ctx.name()
            ))
        })?;
        self.send(&owner, ctx, &asset, request).await
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
                "{} is a {} repository; publish add-ons to a hosted repository",
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
        let outcome = hosted::store_addon(ctx, written, expected.as_deref())
            .await
            .map_err(Refusal::into_error)?;
        let location = format!(
            "{}{}",
            ctx.repository_url(),
            updates::encoded_path(&outcome.asset.path)
        );
        let status = match outcome.created {
            true => StatusCode::CREATED,
            false => StatusCode::OK,
        };
        let mut response = json_response(
            status,
            &json!({
                "id": outcome.addon.id,
                "name": outcome.addon.name,
                "version": outcome.addon.version,
                "signed": outcome.addon.signed,
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
impl Format for Firefox {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        let method = request.method.clone();
        let kind = ctx.kind();
        let result = match (&method, &route) {
            (&Method::GET | &Method::HEAD, Route::Root) => directory_page(&ctx),
            (&Method::GET | &Method::HEAD, Route::Updates) => {
                let id = parameter(request.query.as_deref(), "id");
                self.manifest(&ctx, &request, id).await
            }
            (&Method::GET | &Method::HEAD, Route::VersionCheck) => {
                self.version_check(&ctx, &request).await
            }
            (&Method::GET | &Method::HEAD, Route::AddonUpdates(key)) => {
                self.manifest(&ctx, &request, Some(key.clone())).await
            }
            (&Method::GET | &Method::HEAD, Route::Latest(key)) => {
                self.download_latest(&ctx, &request, key).await
            }
            (&Method::GET | &Method::HEAD, Route::VersionFile(key, version))
            | (&Method::GET | &Method::HEAD, Route::Artifact(key, version)) => {
                self.download_version(&ctx, &request, key, version).await
            }
            (&Method::GET | &Method::HEAD, Route::Archive | Route::Other) => {
                self.read_other(&ctx, &request).await
            }
            (&Method::PUT, Route::Artifact(..)) => {
                let expected = request.path.clone();
                self.put(&ctx, request, Some(expected)).await
            }
            (&Method::PUT, Route::Archive | Route::Latest(_) | Route::VersionFile(..)) => {
                self.put(&ctx, request, None).await
            }
            (&Method::DELETE, _) if kind != RepositoryType::Group && !request.is_directory() => {
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
                .version_file(ctx, &id, &version)
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
        hosted::store_addon(&ctx, written, None)
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
            path: Some("CASE WHEN a.kind = 'addon' THEN a.path END"),
            generated: &[],
        }
    }
}

pub mod testing {
    use astrum_content::zip;
    use serde_json::json;

    pub fn manifest(id: &str, version: &str) -> String {
        json!({
            "manifest_version": 2,
            "name": format!("{id} name"),
            "version": version,
            "description": "An add-on for tests",
            "browser_specific_settings": {"gecko": {"id": id, "strict_min_version": "115.0"}},
            "background": {"scripts": ["background.js"]}
        })
        .to_string()
    }

    pub fn xpi(manifest: &str) -> Vec<u8> {
        zip::write(&[
            ("manifest.json", manifest.as_bytes()),
            ("background.js", b"console.log('hello');"),
        ])
    }

    pub fn signed_xpi(manifest: &str) -> Vec<u8> {
        zip::write(&[
            ("manifest.json", manifest.as_bytes()),
            ("background.js", b"console.log('hello');"),
            ("META-INF/manifest.mf", b"Manifest-Version: 1.0\n"),
            ("META-INF/mozilla.sf", b"Signature-Version: 1.0\n"),
            ("META-INF/mozilla.rsa", b"\x30\x00"),
            ("META-INF/cose.sig", b"\xd8\x62"),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_manifests_and_files() {
        assert_eq!(route("/"), Route::Root);
        assert_eq!(route("/updates.json"), Route::Updates);
        assert_eq!(route("/update/VersionCheck.php"), Route::VersionCheck);
        assert_eq!(
            route("/name@example.org/updates.json"),
            Route::AddonUpdates("name@example.org".to_owned())
        );
        assert_eq!(
            route("/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/latest.xpi"),
            Route::Latest("{ec8030f7-c20a-464f-9b0e-13a3a9e97384}".to_owned())
        );
        assert_eq!(
            route("/ublock-origin/latest.xpi"),
            Route::Latest("ublock-origin".to_owned())
        );
        assert_eq!(
            route("/ublock-origin/1.61.2.xpi"),
            Route::VersionFile("ublock-origin".to_owned(), "1.61.2".to_owned())
        );
        assert_eq!(
            route("/name@example.org/1.2.3/name-1.2.3.xpi"),
            Route::Artifact("name@example.org".to_owned(), "1.2.3".to_owned())
        );
        assert_eq!(
            route(
                "/{ec8030f7-c20a-464f-9b0e-13a3a9e97384}/2.0/ec8030f7-c20a-464f-9b0e-13a3a9e97384-2.0.xpi"
            ),
            Route::Artifact(
                "{ec8030f7-c20a-464f-9b0e-13a3a9e97384}".to_owned(),
                "2.0".to_owned()
            )
        );
        assert_eq!(route("/name@example.org/1.2.3/other.xpi"), Route::Archive);
        assert_eq!(route("/upload/my-addon.XPI"), Route::Archive);
        assert_eq!(route("/addon.xpi"), Route::Archive);
        assert_eq!(route("/-/addon/ublock-origin.json"), Route::Other);
        assert_eq!(route("/updates.json/"), Route::Other);
        assert_eq!(route("/a b/latest.xpi"), Route::Archive);
        assert_eq!(route("/name@example.org/"), Route::Other);
    }

    #[test]
    fn query_parameters_are_decoded() {
        assert_eq!(
            parameter(Some("reqVersion=2&id=%7Bab%7D&version=1"), "id").as_deref(),
            Some("{ab}")
        );
        assert_eq!(parameter(Some("id=&x=1"), "id"), None);
        assert_eq!(parameter(None, "id"), None);
    }
}
