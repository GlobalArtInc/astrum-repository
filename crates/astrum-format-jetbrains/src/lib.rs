pub mod builds;
mod hosted;
mod markup;
pub mod plugin;
mod query;
mod remote;
pub mod xml;

use std::collections::BTreeMap;

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
use futures_util::TryStreamExt;
use serde_json::json;

use crate::hosted::{Claim, Refusal};
use crate::plugin::{artifact_path, valid_id, valid_version};
use crate::query::{Params, Query, channel_of};
use crate::xml::Entry;

pub const ID: &str = "jetbrains";
pub const ARTIFACT: &str = "plugin";
pub const ZIP_TYPE: &str = "application/zip";
const XML_TYPE: &str = "text/xml;charset=UTF-8";
const JSON_TYPE: &str = "application/json";
const MAX_FIELD: u64 = 64 * 1024;

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "jetbrains.vendor",
        target: SearchTarget::Component,
        pointer: &["jetbrains", "plugin", "vendor", "name"],
    },
    SearchKey {
        param: "jetbrains.category",
        target: SearchTarget::Component,
        pointer: &["jetbrains", "plugin", "category"],
    },
    SearchKey {
        param: "jetbrains.channel",
        target: SearchTarget::Component,
        pointer: &["jetbrains", "channel"],
    },
];

const KEYWORDS: &[&[&str]] = &[
    &["jetbrains", "plugin", "name"],
    &["jetbrains", "plugin", "vendor", "name"],
];

#[derive(Default)]
pub struct Jetbrains {
    remote: remote::Remote,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Route {
    Root,
    UpdateFile,
    List(Option<String>),
    Manager,
    VersionDownload,
    Upload,
    Artifact(String, String, &'static str),
    Archive,
    Other,
}

fn artifact_route(id: &str, version: &str, file: &str) -> Option<Route> {
    if !valid_id(id) || !valid_version(version) {
        return None;
    }
    ["zip", "jar"].into_iter().find_map(|extension| {
        (file == format!("{id}-{version}.{extension}"))
            .then(|| Route::Artifact(id.to_owned(), version.to_owned(), extension))
    })
}

fn route(path: &str) -> Route {
    let trimmed = path.trim_matches('/');
    if trimmed.is_empty() {
        return Route::Root;
    }
    let directory = path.ends_with('/');
    let parts: Vec<&str> = trimmed.split('/').collect();
    let found = match parts.as_slice() {
        ["updatePlugins.xml"] if !directory => Some(Route::UpdateFile),
        ["plugins", "list"] => Some(Route::List(None)),
        ["plugins", channel, "list"] => Some(Route::List(Some(channel.to_ascii_lowercase()))),
        ["pluginManager"] => Some(Route::Manager),
        ["plugin", "download"] => Some(Route::VersionDownload),
        ["api", "updates", "upload"] if !directory => Some(Route::Upload),
        [id, version, file] if !directory => artifact_route(id, version, file),
        _ => None,
    };
    if let Some(found) = found {
        return found;
    }
    let lower = trimmed.to_ascii_lowercase();
    match !directory && (lower.ends_with(".zip") || lower.ends_with(".jar")) {
        true => Route::Archive,
        false => Route::Other,
    }
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

fn visible(ctx: &Ctx, id: &str, version: &str) -> bool {
    ["zip", "jar"]
        .into_iter()
        .any(|extension| ctx.allowed(View::Read, &artifact_path(id, version, extension)))
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
        .header(header::CONTENT_TYPE, JSON_TYPE)
        .body(Body::from(value.to_string()))
        .expect("a JSON answer is valid")
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
        "<!DOCTYPE html>\n<html>\n  <head><title>{name}</title></head>\n  <body>\n    <h1>{name}</h1>\n    <p>This is a repository of plugins for IntelliJ Platform IDEs. Add <code>{url}/updatePlugins.xml</code> in Settings | Plugins | Manage Plugin Repositories, or start the IDE with <code>idea.plugins.host={url}</code> to use the repository instead of JetBrains Marketplace.</p>\n    <p>The files of this repository are listed on its <a href=\"{}/service/rest/repository/browse/{name}/\">HTML index</a>.</p>\n  </body>\n</html>\n",
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
        .map(|c| match c.is_ascii_graphic() || c == ' ' {
            true if c != '"' && c != '\\' => c,
            _ => '_',
        })
        .collect();
    if let Ok(value) = HeaderValue::from_str(&format!("attachment; filename=\"{name}\"")) {
        response
            .headers_mut()
            .insert(header::CONTENT_DISPOSITION, value);
    }
    response
}

impl Jetbrains {
    async fn artifact(
        &self,
        ctx: &Ctx,
        id: &str,
        version: &str,
        extension: Option<&str>,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted | RepositoryType::Proxy => {
                let extensions: Vec<&str> = match extension {
                    Some(extension) => vec![extension],
                    None => vec!["zip", "jar"],
                };
                for extension in extensions {
                    let path = artifact_path(id, version, extension);
                    if let Some(found) = store::find(ctx.db(), ctx.name(), &path).await? {
                        return Ok(Some((ctx.clone(), found)));
                    }
                }
                match ctx.kind() {
                    RepositoryType::Proxy => Ok(self
                        .remote
                        .version(ctx, id, version)
                        .await?
                        .map(|found| (ctx.clone(), found))),
                    _ => Ok(None),
                }
            }
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.artifact(&member, id, version, extension)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn latest(
        &self,
        ctx: &Ctx,
        id: &str,
        query: &Query,
    ) -> Result<Option<(Ctx, Asset)>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let stored = hosted::stored(ctx, Some(id)).await?;
                Ok(hosted::select(stored, query)
                    .into_iter()
                    .next()
                    .map(|chosen| (ctx.clone(), chosen.asset)))
            }
            RepositoryType::Proxy => Ok(self
                .remote
                .managed(ctx, id, query)
                .await?
                .map(|found| (ctx.clone(), found))),
            RepositoryType::Group => {
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.latest(&member, id, query)).await {
                        Ok(Some(found)) => return Ok(Some(found)),
                        Ok(None) => {}
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(None)
            }
        }
    }

    async fn documented_entries(
        &self,
        ctx: &Ctx,
        query: &Query,
        base: &str,
    ) -> Result<Vec<Entry>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let stored = hosted::stored(ctx, None).await?;
                Ok(hosted::select(stored, query)
                    .iter()
                    .map(|chosen| xml::documented_entry(&chosen.item(base)))
                    .collect())
            }
            RepositoryType::Proxy => Ok(Vec::new()),
            RepositoryType::Group => {
                let mut lists = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.documented_entries(&member, query, base)).await {
                        Ok(entries) => lists.push(entries),
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(xml::merge(lists))
            }
        }
    }

    async fn legacy_entries(&self, ctx: &Ctx, query: &Query) -> Result<Vec<Entry>, ContentError> {
        match ctx.kind() {
            RepositoryType::Hosted => {
                let stored = hosted::stored(ctx, None).await?;
                let base = ctx.repository_url();
                Ok(hosted::select(stored, query)
                    .iter()
                    .map(|chosen| xml::legacy_entry(&chosen.item(&base)))
                    .collect())
            }
            RepositoryType::Proxy => {
                let Some(asset) = self.remote.listing(ctx, query).await? else {
                    return Ok(Vec::new());
                };
                let bytes = remote::read_list(ctx, &asset).await?;
                xml::parse_entries(&bytes).map_err(|message| {
                    ContentError::BadGateway(format!(
                        "the remote sent a bad plugin list: {message}"
                    ))
                })
            }
            RepositoryType::Group => {
                let mut lists = Vec::new();
                for member in group::members(ctx).await? {
                    let member = ctx.member(member);
                    match Box::pin(self.legacy_entries(&member, query)).await {
                        Ok(entries) => lists.push(entries),
                        Err(error) => log_member_failure(ctx, &member, &error),
                    }
                }
                Ok(xml::merge(lists))
            }
        }
    }

    async fn update_file(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        require_reader(ctx)?;
        if ctx.kind() == RepositoryType::Proxy {
            return Err(ContentError::not_found(
                "a proxy repository answers the requests of idea.plugins.host; it has no updatePlugins.xml",
            ));
        }
        let query = Query::new(&Params::of(request.query.as_deref()), None);
        let mut entries = self
            .documented_entries(ctx, &query, &ctx.repository_url())
            .await?;
        if !matches!(ctx.grant(View::Read), Grant::Whole) {
            entries.retain(|entry| visible(ctx, &entry.id, &entry.version));
        }
        text_response(request, XML_TYPE, xml::documented_document(&entries))
    }

    async fn list(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        channel: Option<&str>,
    ) -> Result<Response, ContentError> {
        require_reader(ctx)?;
        let query = Query::new(&Params::of(request.query.as_deref()), channel);
        if ctx.kind() == RepositoryType::Proxy && matches!(ctx.grant(View::Read), Grant::Whole) {
            let asset = self
                .remote
                .listing(ctx, &query)
                .await?
                .ok_or_else(|| ContentError::not_found("the remote has no such plugins"))?;
            return serve(
                ctx,
                &asset,
                &request.method,
                &request.headers,
                Serve::default(),
            )
            .await;
        }
        let mut entries = self.legacy_entries(ctx, &query).await?;
        if !matches!(ctx.grant(View::Read), Grant::Whole) {
            entries.retain(|entry| visible(ctx, &entry.id, &entry.version));
        }
        text_response(request, XML_TYPE, xml::legacy_document(&entries))
    }

    async fn send(
        &self,
        owner: &Ctx,
        ctx: &Ctx,
        asset: &Asset,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &asset.path)?;
        let response = serve(
            owner,
            asset,
            &request.method,
            &request.headers,
            Serve::download().disposition("attachment"),
        )
        .await?;
        Ok(attachment(response, asset))
    }

    async fn manager(&self, ctx: &Ctx, request: &ContentRequest) -> Result<Response, ContentError> {
        require_reader(ctx)?;
        let params = Params::of(request.query.as_deref());
        if params
            .get("action")
            .is_some_and(|action| action != "download")
        {
            return Err(ContentError::BadRequest(
                "only action=download is served".to_owned(),
            ));
        }
        let id = params.get("id").ok_or_else(|| {
            ContentError::BadRequest("the id of the plugin is missing".to_owned())
        })?;
        let query = Query::new(&params, None);
        let (owner, asset) = self.latest(ctx, id, &query).await?.ok_or_else(|| {
            ContentError::not_found(format!(
                "Can't find update for plugin \"{id}\" compatible with build \"{}\"",
                query.build.as_deref().unwrap_or_default()
            ))
        })?;
        self.send(&owner, ctx, &asset, request).await
    }

    async fn by_version(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        require_reader(ctx)?;
        let params = Params::of(request.query.as_deref());
        let (Some(id), Some(version)) = (params.get("pluginId"), params.get("version")) else {
            return Err(ContentError::BadRequest(
                "pluginId and version are required".to_owned(),
            ));
        };
        let (owner, asset) = self
            .artifact(ctx, id, version, None)
            .await?
            .ok_or_else(|| ContentError::not_found(format!("Can't find update {id} {version}")))?;
        self.send(&owner, ctx, &asset, request).await
    }

    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        id: &str,
        version: &str,
        extension: &str,
    ) -> Result<Response, ContentError> {
        ctx.require(View::Read, &request.path)?;
        let (owner, asset) = self
            .artifact(ctx, id, version, Some(extension))
            .await?
            .ok_or_else(|| ContentError::not_found(request.path.clone()))?;
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
                "{} is a {} repository; publish plugins to a hosted repository",
                ctx.name(),
                ctx.kind()
            )));
        }
        if !hosted::may_write(ctx) {
            return Err(denied(ctx));
        }
        Ok(())
    }

    async fn publish(&self, ctx: &Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        self.require_hosted(ctx)?;
        let content_type = request
            .header("content-type")
            .unwrap_or_default()
            .to_owned();
        let boundary = multer::parse_boundary(&content_type).map_err(|_| {
            ContentError::BadRequest(
                "send the plugin as a multipart/form-data request with a file part".to_owned(),
            )
        })?;
        let limits = multer::Constraints::new().size_limit(
            multer::SizeLimit::new()
                .per_field(MAX_FIELD)
                .for_field("file", u64::MAX),
        );
        let mut multipart = multer::Multipart::with_constraints(
            request.take_body().into_data_stream(),
            boundary,
            limits,
        );
        let bad = |error: multer::Error| {
            ContentError::BadRequest(format!("the upload form cannot be read: {error}"))
        };
        let mut written = None;
        let mut fields: BTreeMap<String, String> = BTreeMap::new();
        while let Some(part) = multipart.next_field().await.map_err(bad)? {
            let name = part.name().unwrap_or_default().to_owned();
            if name == "file" && written.is_none() {
                let body: astrum_blob::ContentStream =
                    Box::pin(part.map_err(std::io::Error::other));
                written = Some(store::write_blob(ctx, body).await?);
            } else {
                let text = part.text().await.map_err(bad)?;
                fields.insert(name, text);
            }
        }
        let written = written
            .ok_or_else(|| ContentError::BadRequest("the upload has no file part".to_owned()))?;
        let claim = Claim {
            xml_id: fields
                .get("xmlId")
                .or_else(|| fields.get("pluginId"))
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty()),
            channel: channel_of(fields.get("channel").map(String::as_str)),
            path: None,
        };
        let (found, stored) = hosted::store_plugin(ctx, written, &claim)
            .await
            .map_err(Refusal::into_error)?;
        Ok(json_response(
            StatusCode::OK,
            &json!({
                "pluginXmlId": found.plugin.id,
                "version": found.plugin.version,
                "channel": claim.channel,
                "url": format!("{}{}", ctx.repository_url(), stored.path),
            }),
        ))
    }

    async fn put(
        &self,
        ctx: &Ctx,
        request: ContentRequest,
        expected: Option<String>,
    ) -> Result<Response, ContentError> {
        self.require_hosted(ctx)?;
        let channel = channel_of(Params::of(request.query.as_deref()).get("channel"));
        let written = store::write_blob(ctx, request.into_stream()).await?;
        let claim = Claim {
            xml_id: None,
            channel,
            path: expected,
        };
        let (found, stored) = hosted::store_plugin(ctx, written, &claim)
            .await
            .map_err(Refusal::into_error)?;
        let location = format!("{}{}", ctx.repository_url(), stored.path);
        let mut response = json_response(
            StatusCode::CREATED,
            &json!({
                "pluginXmlId": found.plugin.id,
                "version": found.plugin.version,
                "channel": claim.channel,
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
                RepositoryType::Hosted => "GET,HEAD,POST,PUT,DELETE",
                RepositoryType::Proxy => "GET,HEAD,DELETE",
                RepositoryType::Group => "GET,HEAD",
            },
        }
    }
}

#[async_trait]
impl Format for Jetbrains {
    fn id(&self) -> &'static str {
        ID
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route(&request.path);
        let method = request.method.clone();
        let kind = ctx.kind();
        let result = match (&method, &route) {
            (&Method::GET | &Method::HEAD, Route::Root) => directory_page(&ctx),
            (&Method::GET | &Method::HEAD, Route::UpdateFile) => {
                self.update_file(&ctx, &request).await
            }
            (&Method::GET | &Method::HEAD, Route::List(channel)) => {
                self.list(&ctx, &request, channel.as_deref()).await
            }
            (&Method::GET | &Method::HEAD, Route::Manager) => self.manager(&ctx, &request).await,
            (&Method::GET | &Method::HEAD, Route::VersionDownload) => {
                self.by_version(&ctx, &request).await
            }
            (&Method::GET | &Method::HEAD, Route::Artifact(id, version, extension)) => {
                self.download(&ctx, &request, id, version, extension).await
            }
            (&Method::GET | &Method::HEAD, Route::Archive | Route::Other) => {
                self.read_other(&ctx, &request).await
            }
            (&Method::POST, Route::Upload) => self.publish(&ctx, request).await,
            (&Method::PUT, Route::Artifact(..)) => {
                let expected = request.path.clone();
                self.put(&ctx, request, Some(expected)).await
            }
            (&Method::PUT, Route::Archive) => self.put(&ctx, request, None).await,
            (&Method::DELETE, Route::Artifact(..) | Route::Archive | Route::Other)
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
            Route::Artifact(id, version, extension) => Ok(self
                .artifact(ctx, &id, &version, Some(extension))
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
            component_fields: vec![
                UploadField::text("channel", true)
                    .describe("The release channel; empty for the default channel"),
            ],
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
        let channel = channel_of(
            form.field(&format!("{ID}.channel"))
                .or_else(|| form.field("channel")),
        );
        let written = store::write_blob(&ctx, UploadForm::stream(file).await?).await?;
        let claim = Claim {
            xml_id: None,
            channel,
            path: None,
        };
        hosted::store_plugin(&ctx, written, &claim)
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
            path: Some("CASE WHEN a.kind = 'plugin' THEN a.path END"),
            generated: &[],
        }
    }
}

pub mod testing {
    use astrum_content::zip;

    use crate::plugin::DESCRIPTOR;

    pub fn descriptor(id: &str, version: &str, since: &str, until: &str) -> String {
        format!(
            "<idea-plugin>\n  <id>{id}</id>\n  <name>{id} name</name>\n  <version>{version}</version>\n  <vendor email=\"dev@example.test\" url=\"https://example.test/\">Example Vendor</vendor>\n  <idea-version since-build=\"{since}\" until-build=\"{until}\"/>\n  <category>Tools</category>\n  <depends>com.intellij.modules.platform</depends>\n  <description><![CDATA[<p>The {id} plugin.</p>]]></description>\n  <change-notes><![CDATA[Version {version}.]]></change-notes>\n</idea-plugin>\n"
        )
    }

    pub fn jar(descriptor: &str) -> Vec<u8> {
        zip::write(&[
            (DESCRIPTOR, descriptor.as_bytes()),
            ("messages/Bundle.properties", b"title=Example"),
        ])
    }

    pub fn bundle(directory: &str, descriptor: &str) -> Vec<u8> {
        let main = jar(descriptor);
        let library = zip::write(&[("org/example/Library.class", b"\xca\xfe\xba\xbe")]);
        zip::write(&[
            (
                format!("{directory}/lib/{directory}.jar").as_str(),
                main.as_slice(),
            ),
            (
                format!("{directory}/lib/library.jar").as_str(),
                library.as_slice(),
            ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn paths_route_to_the_ide_requests_and_the_plugin_files() {
        assert_eq!(route("/"), Route::Root);
        assert_eq!(route("/updatePlugins.xml"), Route::UpdateFile);
        assert_eq!(route("/plugins/list"), Route::List(None));
        assert_eq!(route("/plugins/list/"), Route::List(None));
        assert_eq!(
            route("/plugins/EAP/list/"),
            Route::List(Some("eap".to_owned()))
        );
        assert_eq!(route("/pluginManager"), Route::Manager);
        assert_eq!(route("/pluginManager/"), Route::Manager);
        assert_eq!(route("/plugin/download"), Route::VersionDownload);
        assert_eq!(route("/api/updates/upload"), Route::Upload);
        assert_eq!(
            route("/org.acme.tool/1.2.3/org.acme.tool-1.2.3.zip"),
            Route::Artifact("org.acme.tool".to_owned(), "1.2.3".to_owned(), "zip")
        );
        assert_eq!(
            route("/PlantUML integration/1.0/PlantUML integration-1.0.jar"),
            Route::Artifact("PlantUML integration".to_owned(), "1.0".to_owned(), "jar")
        );
        assert_eq!(route("/tool-1.0.zip"), Route::Archive);
        assert_eq!(route("/a/b/other.JAR"), Route::Archive);
        assert_eq!(
            route("/org.acme.tool/1.2.3/other-1.2.3.zip"),
            Route::Archive
        );
        assert_eq!(route("/plugins/list/IU-243-abc.xml"), Route::Other);
        assert_eq!(route("/updatePlugins.xml/"), Route::Other);
        assert_eq!(route("/plugins/eap/"), Route::Other);
        assert_eq!(route("/api/updates/upload/"), Route::Other);
    }
}
