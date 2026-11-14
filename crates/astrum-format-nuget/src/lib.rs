mod archive;
mod docs;
mod hosted;
pub mod import;
pub mod metadata;
mod odata;
mod registration;
mod remote;
mod reply;
pub mod route;
mod search;
#[cfg(test)]
mod testing;
mod urls;
pub mod version;
mod xml;

use astrum_content::format::{SearchKey, SearchTarget};
use astrum_content::inventory::{Inventory, LicenseFact, PackageRef};
use astrum_content::store::{self};
use astrum_content::{
    Asset, BrowseLayout, Component, ContentError, ContentRequest, Ctx, Format, Serve, UploadField,
    UploadForm, UploadSpec, View, serve,
};
use astrum_repo::RepositoryType;
use astrum_scan::osv::normalise_name;
use async_trait::async_trait;
use axum::http::Method;
use axum::response::Response;
use percent_encoding::percent_decode_str;
use serde_json::{Value, json};

use crate::docs::File;
use crate::odata::{Page, Request};
use crate::remote::{Cache, Mode};
use crate::route::{FileKind, Route, file_kind, query_pairs};
use crate::search::Query;
use crate::urls::Urls;
use crate::version::Version;

pub const ID: &str = "nuget";
const LICENSE_TEXT_LIMIT: usize = 1024;
const LICENSES_URL: &str = "https://licenses.nuget.org/";
const DEPRECATE_LICENSE_URL: &str = "https://aka.ms/deprecateLicenseUrl";
pub const PACKAGE_KIND: &str = "package";
pub const SYMBOLS_KIND: &str = "symbols";
pub const ZIP: &str = "application/zip";

const SEARCH_KEYS: &[SearchKey] = &[
    SearchKey {
        param: "nuget.id",
        target: SearchTarget::Component,
        pointer: &[ID, "id"],
    },
    SearchKey {
        param: "nuget.tags",
        target: SearchTarget::Component,
        pointer: &[ID, "tags"],
    },
];

#[derive(Default)]
pub struct Nuget {
    cache: Cache,
}

fn not_found() -> ContentError {
    ContentError::not_found("Not Found")
}

fn not_allowed(ctx: &Ctx, method: &Method) -> ContentError {
    ContentError::MethodNotAllowed {
        method: method.to_string(),
        allow: match ctx.kind() {
            RepositoryType::Hosted => "GET,HEAD,PUT,POST,DELETE",
            RepositoryType::Proxy => "GET,HEAD,DELETE",
            RepositoryType::Group => "GET,HEAD",
        },
    }
}

fn speaks_v3(ctx: &Ctx) -> bool {
    ctx.kind() != RepositoryType::Proxy || remote::mode(ctx) == Mode::V3
}

fn speaks_v2(ctx: &Ctx) -> bool {
    ctx.kind() != RepositoryType::Proxy || remote::mode(ctx) == Mode::V2
}

fn service_index(ctx: &Ctx, urls: &Urls) -> Value {
    let mut resources: Vec<(String, &str, &str)> = vec![(
        urls.content_base(),
        "PackageBaseAddress/3.0.0",
        "Package contents: version lists and package files",
    )];
    if ctx.kind() == RepositoryType::Hosted {
        resources.push((
            urls.root(),
            "PackagePublish/2.0.0",
            "Push and delete packages",
        ));
        resources.push((
            urls.root(),
            "SymbolPackagePublish/4.9.0",
            "Push symbol packages",
        ));
    }
    for kind in [
        "RegistrationsBaseUrl",
        "RegistrationsBaseUrl/3.0.0-beta",
        "RegistrationsBaseUrl/3.0.0-rc",
        "RegistrationsBaseUrl/3.4.0",
        "RegistrationsBaseUrl/3.6.0",
        "RegistrationsBaseUrl/Versioned",
    ] {
        resources.push((urls.registration_base(), kind, "Package metadata"));
    }
    for kind in [
        "SearchQueryService",
        "SearchQueryService/3.0.0-beta",
        "SearchQueryService/3.0.0-rc",
        "SearchQueryService/3.5.0",
    ] {
        resources.push((urls.query(), kind, "Package search"));
    }
    for kind in [
        "SearchAutocompleteService",
        "SearchAutocompleteService/3.0.0-beta",
        "SearchAutocompleteService/3.0.0-rc",
        "SearchAutocompleteService/3.5.0",
    ] {
        resources.push((
            urls.autocomplete(),
            kind,
            "Package id and version completion",
        ));
    }
    if ctx.kind() != RepositoryType::Proxy {
        resources.push((urls.root(), "LegacyGallery/2.0.0", "NuGet V2 OData feed"));
    }
    json!({
        "version": "3.0.0",
        "resources": resources
            .into_iter()
            .map(|(id, kind, comment)| json!({"@id": id, "@type": kind, "comment": comment}))
            .collect::<Vec<_>>(),
    })
}

fn found(value: Option<Value>, method: &Method) -> Result<Response, ContentError> {
    value
        .map(|value| reply::json(method, &value))
        .ok_or_else(not_found)
}

fn next_link(urls: &Urls, path: &str, raw: Option<&str>, skip: usize) -> String {
    let mut pairs: Vec<(String, String)> = query_pairs(raw)
        .into_iter()
        .filter(|(key, _)| !key.eq_ignore_ascii_case("$skip"))
        .collect();
    pairs.push(("$skip".to_owned(), skip.to_string()));
    let query = url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish();
    format!("{}{}?{query}", urls.root(), path.trim_start_matches('/'))
}

impl Nuget {
    async fn download(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        id: &str,
        version: &str,
        file: File,
    ) -> Result<Response, ContentError> {
        let (owner, asset) = docs::file(ctx, &self.cache, id, version, file)
            .await?
            .ok_or_else(not_found)?;
        serve(
            &owner,
            &asset,
            &request.method,
            &request.headers,
            Serve::download(),
        )
        .await
    }

    async fn content(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        id: &str,
        version: &str,
        file: &str,
    ) -> Result<Response, ContentError> {
        match file_kind(id, version, file) {
            Some(FileKind::Package) => {
                self.download(ctx, request, id, version, File::Package)
                    .await
            }
            Some(FileKind::Symbols) => {
                self.download(ctx, request, id, version, File::Symbols)
                    .await
            }
            Some(FileKind::Nuspec) => {
                let bytes = docs::nuspec(ctx, &self.cache, id, version)
                    .await?
                    .ok_or_else(not_found)?;
                Ok(reply::body(
                    &request.method,
                    "application/xml",
                    String::from_utf8_lossy(&bytes).into_owned(),
                ))
            }
            None => Err(not_found()),
        }
    }

    async fn feed(
        &self,
        ctx: &Ctx,
        request: &ContentRequest,
        feed: route::Feed,
        count: bool,
        urls: &Urls,
    ) -> Result<Response, ContentError> {
        let raw = request.query.as_deref();
        let pairs = query_pairs(raw);
        let parsed = Request::parse(feed, count, &pairs);
        let path = request.path.trim_start_matches('/');
        let feed_path = path.strip_suffix("/$count").unwrap_or(path);
        if count && ctx.kind() == RepositoryType::Proxy {
            let url = remote::v2_url(ctx, path, raw)?;
            let body = remote::forward(ctx, &self.cache, url)
                .await?
                .ok_or_else(not_found)?;
            return Ok(reply::body(
                &request.method,
                reply::TEXT,
                String::from_utf8_lossy(&body).trim().to_owned(),
            ));
        }
        let result = docs::v2(ctx, &self.cache, &parsed, feed_path, raw)
            .await?
            .ok_or_else(not_found)?;
        if count {
            let start = parsed.skip.min(result.entries.len());
            let end = parsed
                .top
                .map(|top| start.saturating_add(top))
                .unwrap_or(result.entries.len())
                .min(result.entries.len());
            return Ok(reply::body(
                &request.method,
                reply::TEXT,
                (end - start).to_string(),
            ));
        }
        let name = feed_path.split('/').next().unwrap_or("Packages");
        let name = name.trim_end_matches("()");
        let (entries, next) = match result.upstream {
            true => (
                result.entries.as_slice(),
                result
                    .next
                    .as_deref()
                    .and_then(|next| remote::local_next(ctx, next, urls)),
            ),
            false => {
                let (shown, more) = odata::page(&result.entries, &parsed);
                let next = more.then(|| next_link(urls, feed_path, raw, parsed.skip + shown.len()));
                (shown, next)
            }
        };
        let text = odata::feed(
            name,
            &Page {
                entries,
                total: result.total,
                next,
            },
            urls,
        );
        Ok(reply::body(&request.method, reply::FEED, text))
    }

    async fn read(
        &self,
        ctx: &Ctx,
        route: Route,
        request: &ContentRequest,
    ) -> Result<Response, ContentError> {
        let urls = Urls::of(ctx);
        let method = &request.method;
        let v3 = speaks_v3(ctx);
        let v2 = speaks_v2(ctx);
        match route {
            Route::ServiceIndex if v3 => Ok(reply::json(method, &service_index(ctx, &urls))),
            Route::Versions { id } if v3 => found(
                docs::versions(ctx, &self.cache, &id)
                    .await?
                    .map(|versions| json!({ "versions": versions })),
                method,
            ),
            Route::Content { id, version, file } if v3 => {
                self.content(ctx, request, &id, &version, &file).await
            }
            Route::RegistrationIndex { id } if v3 => found(
                docs::registration_index(ctx, &self.cache, &id, &urls).await?,
                method,
            ),
            Route::RegistrationPage { id, lower, upper } if v3 => found(
                docs::registration_page(ctx, &self.cache, &id, &lower, &upper, &urls).await?,
                method,
            ),
            Route::RegistrationLeaf { id, version } if v3 => found(
                docs::registration_leaf(ctx, &self.cache, &id, &version, &urls, false).await?,
                method,
            ),
            Route::CatalogLeaf { id, version } if v3 => found(
                docs::registration_leaf(ctx, &self.cache, &id, &version, &urls, true).await?,
                method,
            ),
            Route::Query | Route::Autocomplete if v3 => {
                let autocomplete = route == Route::Autocomplete;
                let query = Query::parse(&query_pairs(request.query.as_deref()));
                let result = docs::search(
                    ctx,
                    &self.cache,
                    &query,
                    request.query.as_deref(),
                    &urls,
                    autocomplete,
                )
                .await?;
                Ok(reply::json(method, &result.to_json()))
            }
            Route::V2Root if v2 => Ok(reply::body(method, reply::XML, odata::service(&urls))),
            Route::V2Metadata if v2 => Ok(reply::body(method, reply::XML, odata::metadata())),
            Route::V2Feed { feed, count } if v2 => {
                self.feed(ctx, request, feed, count, &urls).await
            }
            Route::V2Entry { id, version } if v2 => {
                let entry = docs::v2_entry(
                    ctx,
                    &self.cache,
                    &id,
                    &version,
                    request.path.trim_start_matches('/'),
                )
                .await?
                .ok_or_else(not_found)?;
                Ok(reply::body(
                    method,
                    reply::ENTRY,
                    odata::entry(&entry, &urls),
                ))
            }
            Route::V2Download { id, version } => {
                self.download(ctx, request, &id, &version, File::Package)
                    .await
            }
            Route::Asset { id, version, file } => match file_kind(&id, &version, &file) {
                Some(FileKind::Package) => {
                    self.download(ctx, request, &id, &version, File::Package)
                        .await
                }
                Some(FileKind::Symbols) => {
                    self.download(ctx, request, &id, &version, File::Symbols)
                        .await
                }
                _ => Err(not_found()),
            },
            _ => Err(not_found()),
        }
    }
}

fn package_inventory(name: &str, version: &str, license: Option<LicenseFact>) -> Option<Inventory> {
    if name.is_empty() {
        return None;
    }
    let name = normalise_name("NuGet", name);
    let version = Version::parse(version)?.lower();
    let purl = format!("pkg:nuget/{name}@{version}");
    Some(Inventory {
        key: purl.clone(),
        packages: vec![PackageRef {
            ecosystem: "NuGet".to_owned(),
            name,
            version,
            purl,
        }],
        license,
        complete: true,
        ..Inventory::default()
    })
}

fn license_fact(attributes: &Value) -> Option<LicenseFact> {
    let stored = attributes.get(ID)?;
    let text = |key: &str| {
        stored
            .get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
    };
    let declared = text("licenseExpression")
        .map(str::to_owned)
        .or_else(|| text("licenseUrl").and_then(declared_of_url))
        .or_else(|| {
            stored
                .get("licenseFile")
                .and_then(Value::as_bool)
                .filter(|file| *file)
                .map(|_| "license file".to_owned())
        })?;
    Some(LicenseFact {
        declared: cap(&declared),
        source: "nuget.nuspec".to_owned(),
    })
}

fn declared_of_url(url: &str) -> Option<String> {
    if url == DEPRECATE_LICENSE_URL {
        return None;
    }
    let Some(expression) = url.strip_prefix(LICENSES_URL) else {
        return Some(url.to_owned());
    };
    let decoded = percent_decode_str(expression).decode_utf8_lossy();
    let decoded = decoded.trim();
    (!decoded.is_empty()).then(|| decoded.to_owned())
}

fn cap(text: &str) -> String {
    let mut end = text.len().min(LICENSE_TEXT_LIMIT);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

#[async_trait]
impl Format for Nuget {
    fn id(&self) -> &'static str {
        ID
    }

    async fn inventory(
        &self,
        _ctx: &Ctx,
        component: &Component,
    ) -> Result<Option<Inventory>, ContentError> {
        Ok(package_inventory(
            &component.name,
            &component.version,
            license_fact(&component.attributes),
        ))
    }

    async fn handle(&self, ctx: Ctx, request: ContentRequest) -> Result<Response, ContentError> {
        let route = route::route(&request.path);
        let method = request.method.clone();
        match (route, &method) {
            (Route::V2Root | Route::Publish, &Method::PUT | &Method::POST) => {
                hosted::push(&ctx, request).await
            }
            (Route::Package { id, version }, &Method::DELETE) => {
                hosted::delete(&ctx, &id, &version).await
            }
            (route, &Method::GET | &Method::HEAD) => {
                ctx.require(View::Read, &request.path)?;
                self.read(&ctx, route, &request).await
            }
            (_, method) => Err(not_allowed(&ctx, method)),
        }
    }

    async fn get(&self, ctx: &Ctx, path: &str) -> Result<Option<Asset>, ContentError> {
        if let Route::Asset { id, version, file } = route::route(path) {
            let wanted = match file_kind(&id, &version, &file) {
                Some(FileKind::Package) => Some(File::Package),
                Some(FileKind::Symbols) => Some(File::Symbols),
                _ => None,
            };
            if let Some(wanted) = wanted {
                return Ok(docs::file(ctx, &self.cache, &id, &version, wanted)
                    .await?
                    .map(|(_, asset)| asset));
            }
        }
        if ctx.kind() == RepositoryType::Group {
            return Ok(None);
        }
        store::find(ctx.db(), ctx.name(), path).await
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
        hosted::upload(&ctx, form).await
    }

    fn browse_layout(&self) -> BrowseLayout {
        BrowseLayout {
            path: Some(
                "CASE WHEN c.id IS NOT NULL THEN '/' || c.name || '/' || c.version || regexp_replace(a.path, '^.*/', '/') END",
            ),
            generated: &[],
        }
    }

    fn search_keys(&self) -> &'static [SearchKey] {
        SEARCH_KEYS
    }
}

impl Nuget {
    pub fn forget(&self, repository: &str) {
        self.cache.forget(repository);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fact_of(license: &str) -> Option<LicenseFact> {
        let nuspec = format!(
            r#"<package><metadata><id>A</id><version>1.0</version>{license}</metadata></package>"#
        );
        let metadata = metadata::parse_nuspec(nuspec.as_bytes()).unwrap();
        license_fact(&hosted::attributes(&metadata))
    }

    fn declared(text: &str) -> Option<LicenseFact> {
        Some(LicenseFact {
            declared: text.to_owned(),
            source: "nuget.nuspec".to_owned(),
        })
    }

    #[test]
    fn licence_expressions_are_declared_as_they_are() {
        assert_eq!(
            fact_of(r#"<license type="expression">MIT OR Apache-2.0</license>"#),
            declared("MIT OR Apache-2.0")
        );
    }

    #[test]
    fn file_licences_use_their_url_when_there_is_one() {
        assert_eq!(
            fact_of(
                r#"<license type="file">LICENSE.txt</license><licenseUrl>https://example.org/terms</licenseUrl>"#
            ),
            declared("https://example.org/terms")
        );
    }

    #[test]
    fn file_licences_without_a_url_are_declared_as_a_licence_file() {
        assert_eq!(
            fact_of(r#"<license type="file">LICENSE.txt</license>"#),
            declared("license file")
        );
    }

    #[test]
    fn nuget_licence_urls_give_their_expression() {
        assert_eq!(
            fact_of("<licenseUrl>https://licenses.nuget.org/(MIT%20OR%20Apache-2.0)</licenseUrl>"),
            declared("(MIT OR Apache-2.0)")
        );
    }

    #[test]
    fn the_deprecation_placeholder_gives_no_url_fact() {
        assert!(fact_of("<licenseUrl>https://aka.ms/deprecateLicenseUrl</licenseUrl>").is_none());
        assert_eq!(
            fact_of(
                r#"<license type="file">LICENSE.txt</license><licenseUrl>https://aka.ms/deprecateLicenseUrl</licenseUrl>"#
            ),
            declared("license file")
        );
    }

    #[test]
    fn unknown_licence_urls_are_declared_as_they_are() {
        assert_eq!(
            fact_of("<licenseUrl>https://example.org/terms</licenseUrl>"),
            declared("https://example.org/terms")
        );
    }

    #[test]
    fn packages_without_a_licence_have_no_fact() {
        assert!(fact_of("").is_none());
    }

    #[test]
    fn hosted_and_proxy_packages_share_one_key_by_lower_case_id_and_normalised_version() {
        let hosted = package_inventory("Newtonsoft.Json", "13.0.3", None).unwrap();
        assert_eq!(hosted.key, "pkg:nuget/newtonsoft.json@13.0.3");
        assert_eq!(hosted.packages[0].ecosystem, "NuGet");
        assert_eq!(hosted.packages[0].name, "newtonsoft.json");
        let proxy = package_inventory("newtonsoft.json", "13.0.3.0", None).unwrap();
        assert_eq!(proxy.key, hosted.key);
        assert_eq!(proxy.packages[0].name, hosted.packages[0].name);
        assert!(package_inventory("newtonsoft.json", "latest", None).is_none());
        assert!(package_inventory("newtonsoft.json", "", None).is_none());
    }

    #[test]
    fn prerelease_label_case_does_not_split_hosted_and_proxy_keys() {
        let hosted = package_inventory("Contoso.Core", "1.0.0-Beta", None).unwrap();
        let proxy = package_inventory("contoso.core", "1.0.0-beta", None).unwrap();
        assert_eq!(hosted.key, "pkg:nuget/contoso.core@1.0.0-beta");
        assert_eq!(proxy.key, hosted.key);
        assert_eq!(hosted.packages[0].version, "1.0.0-beta");
    }
}
