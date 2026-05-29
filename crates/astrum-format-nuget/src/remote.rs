use std::collections::HashMap;
use std::io::Read;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use astrum_content::store::{self, ComponentKey, PutAsset};
use astrum_content::{Asset, ContentError, Ctx, Freshness, proxy};
use astrum_db::entity::component;
use astrum_repo::config::NugetVersion;
use axum::http::StatusCode;
use futures_util::TryStreamExt;
use sea_orm::{ActiveValue, EntityTrait};
use serde_json::{Value, json};

use crate::hosted;
use crate::metadata::parse_nuspec;
use crate::route::package_path;
use crate::urls::Urls;
use crate::{ID, PACKAGE_KIND, ZIP};

pub const INDEX_PATH: &str = "/.nuget/upstream-index.json";
const MAX_DOCUMENT: usize = 64 * 1024 * 1024;
const MAX_QUERIES: usize = 2_000;
const DEFAULT_QUERY_SECONDS: i32 = 3600;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    V2,
    V3,
}

pub fn mode(ctx: &Ctx) -> Mode {
    match ctx
        .repository
        .attributes
        .nuget_proxy
        .as_ref()
        .and_then(|nuget| nuget.nuget_version)
    {
        Some(NugetVersion::V2) => Mode::V2,
        _ => Mode::V3,
    }
}

fn remote_base(ctx: &Ctx) -> Result<String, ContentError> {
    remote_of_repository(&ctx.repository)
}

fn remote_of_repository(
    repository: &astrum_repo::config::Repository,
) -> Result<String, ContentError> {
    repository
        .attributes
        .proxy
        .as_ref()
        .and_then(|proxy| proxy.remote_url.clone())
        .filter(|remote| !remote.trim().is_empty())
        .ok_or_else(|| {
            ContentError::internal(anyhow::anyhow!(
                "the proxy {} has no remote URL",
                repository.name
            ))
        })
}

fn index_url(ctx: &Ctx) -> Result<url::Url, ContentError> {
    index_url_of(&ctx.repository)
}

pub(crate) fn index_url_of(
    repository: &astrum_repo::config::Repository,
) -> Result<url::Url, ContentError> {
    let remote = remote_of_repository(repository)?;
    let remote = remote.trim();
    let joined = match remote.to_ascii_lowercase().ends_with(".json") {
        true => remote.to_owned(),
        false => format!("{}/index.json", remote.trim_end_matches('/')),
    };
    url::Url::parse(&joined)
        .map_err(|error| ContentError::BadGateway(format!("the remote URL is not valid: {error}")))
}

fn v2_root(ctx: &Ctx) -> Result<String, ContentError> {
    Ok(format!(
        "{}/",
        remote_base(ctx)?.trim().trim_end_matches('/')
    ))
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Upstream {
    pub package_base: Option<String>,
    pub registration_base: Option<String>,
    pub registration_bases: Vec<String>,
    pub search: Option<String>,
    pub autocomplete: Option<String>,
}

fn slash(url: &str) -> String {
    format!("{}/", url.trim_end_matches('/'))
}

fn pick(resources: &[(String, String)], preferences: &[&str]) -> Option<String> {
    preferences.iter().find_map(|wanted| {
        resources
            .iter()
            .find(|(kind, _)| kind.eq_ignore_ascii_case(wanted))
            .map(|(_, id)| id.clone())
    })
}

pub fn read_index(value: &Value) -> Upstream {
    let resources: Vec<(String, String)> = value
        .get("resources")
        .and_then(Value::as_array)
        .map(|resources| {
            resources
                .iter()
                .filter_map(|resource| {
                    let id = resource.get("@id")?.as_str()?.to_owned();
                    let kinds = match resource.get("@type")? {
                        Value::String(kind) => vec![kind.clone()],
                        Value::Array(kinds) => kinds
                            .iter()
                            .filter_map(|kind| kind.as_str().map(str::to_owned))
                            .collect(),
                        _ => Vec::new(),
                    };
                    Some(kinds.into_iter().map(move |kind| (kind, id.clone())))
                })
                .flatten()
                .collect()
        })
        .unwrap_or_default();
    let mut registration_bases: Vec<String> = resources
        .iter()
        .filter(|(kind, _)| kind.starts_with("RegistrationsBaseUrl"))
        .map(|(_, id)| slash(id))
        .collect();
    registration_bases.sort();
    registration_bases.dedup();
    Upstream {
        package_base: pick(&resources, &["PackageBaseAddress/3.0.0"]).map(|base| slash(&base)),
        registration_base: pick(
            &resources,
            &[
                "RegistrationsBaseUrl/3.6.0",
                "RegistrationsBaseUrl/Versioned",
                "RegistrationsBaseUrl/3.4.0",
                "RegistrationsBaseUrl/3.0.0-rc",
                "RegistrationsBaseUrl/3.0.0-beta",
                "RegistrationsBaseUrl",
            ],
        )
        .map(|base| slash(&base)),
        registration_bases,
        search: pick(
            &resources,
            &[
                "SearchQueryService/3.5.0",
                "SearchQueryService/3.0.0-rc",
                "SearchQueryService/3.0.0-beta",
                "SearchQueryService",
            ],
        ),
        autocomplete: pick(
            &resources,
            &[
                "SearchAutocompleteService/3.5.0",
                "SearchAutocompleteService/3.0.0-rc",
                "SearchAutocompleteService/3.0.0-beta",
                "SearchAutocompleteService",
            ],
        ),
    }
}

impl Upstream {
    pub fn prefixes(&self, urls: &Urls) -> Vec<(String, String)> {
        let mut prefixes: Vec<(String, String)> = self
            .registration_bases
            .iter()
            .map(|base| (base.clone(), urls.registration_base()))
            .collect();
        if let Some(base) = &self.package_base {
            prefixes.push((base.clone(), urls.content_base()));
        }
        prefixes.sort_by_key(|(from, _)| std::cmp::Reverse(from.len()));
        prefixes
    }
}

pub fn rewrite(value: &mut Value, prefixes: &[(String, String)]) {
    match value {
        Value::String(text) => {
            if let Some((from, to)) = prefixes.iter().find(|(from, _)| text.starts_with(from)) {
                *text = format!("{to}{}", &text[from.len()..]);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|item| rewrite(item, prefixes)),
        Value::Object(fields) => fields
            .values_mut()
            .for_each(|field| rewrite(field, prefixes)),
        _ => {}
    }
}

type Queries = HashMap<(String, String), (Instant, Arc<Vec<u8>>)>;

#[derive(Default)]
pub struct Cache {
    indexes: Mutex<HashMap<String, (uuid::Uuid, Arc<Upstream>)>>,
    queries: Mutex<Queries>,
}

impl Cache {
    fn query(&self, key: &(String, String), age: Duration) -> Option<Arc<Vec<u8>>> {
        let queries = self
            .queries
            .lock()
            .expect("the query cache lock is never poisoned");
        queries
            .get(key)
            .filter(|(at, _)| at.elapsed() < age)
            .map(|(_, body)| body.clone())
    }

    fn remember(&self, key: (String, String), body: Arc<Vec<u8>>) {
        let mut queries = self
            .queries
            .lock()
            .expect("the query cache lock is never poisoned");
        if queries.len() >= MAX_QUERIES {
            queries.clear();
        }
        queries.insert(key, (Instant::now(), body));
    }

    pub fn forget(&self, repository: &str) {
        self.indexes
            .lock()
            .expect("the index cache lock is never poisoned")
            .remove(repository);
        self.queries
            .lock()
            .expect("the query cache lock is never poisoned")
            .retain(|(name, _), _| name != repository);
    }
}

pub async fn read_asset(ctx: &Ctx, asset: &Asset) -> Result<Vec<u8>, ContentError> {
    let store = ctx.content.blob_stores().get(&asset.blob_store).await?;
    let content = store.read(asset.blob_id.into()).await?.ok_or_else(|| {
        ContentError::internal(anyhow::anyhow!(
            "the blob of {}{} is missing",
            asset.repository,
            asset.path
        ))
    })?;
    let mut stream = content.stream;
    let mut collected = Vec::with_capacity(asset.size.max(0) as usize);
    while let Some(chunk) = stream.try_next().await.map_err(ContentError::internal)? {
        if collected.len() + chunk.len() > MAX_DOCUMENT {
            return Err(ContentError::internal(anyhow::anyhow!(
                "the document {}{} is too large",
                asset.repository,
                asset.path
            )));
        }
        collected.extend_from_slice(&chunk);
    }
    Ok(inflate(collected))
}

pub fn inflate(bytes: Vec<u8>) -> Vec<u8> {
    if bytes.starts_with(&[0x1f, 0x8b]) {
        let mut inflated = Vec::new();
        if flate2::read::GzDecoder::new(bytes.as_slice())
            .take(MAX_DOCUMENT as u64)
            .read_to_end(&mut inflated)
            .is_ok()
        {
            return inflated;
        }
    }
    bytes
}

fn parse_json(bytes: &[u8]) -> Result<Value, ContentError> {
    serde_json::from_slice(bytes).map_err(|error| {
        ContentError::BadGateway(format!("the remote answered with broken JSON: {error}"))
    })
}

async fn cached(ctx: &Ctx, path: &str, url: url::Url) -> Result<Option<Asset>, ContentError> {
    proxy::fetch_from(
        ctx,
        path,
        Freshness::Metadata,
        || async move { Ok(Some(proxy::Origin::new(url))) },
        || PutAsset::cached(path).content_type("application/json"),
    )
    .await
}

fn remote_of(asset: &Asset) -> Option<&str> {
    asset
        .attributes
        .pointer("/nuget/remote")
        .and_then(Value::as_str)
}

pub async fn upstream(ctx: &Ctx, cache: &Cache) -> Result<Arc<Upstream>, ContentError> {
    let url = index_url(ctx)?;
    if let Some(known) = store::find(ctx.db(), ctx.name(), INDEX_PATH).await?
        && remote_of(&known) != Some(url.as_str())
    {
        store::delete(ctx, INDEX_PATH).await?;
    }
    let remote = url.to_string();
    let asset = proxy::fetch_from(
        ctx,
        INDEX_PATH,
        Freshness::Metadata,
        || async move { Ok(Some(proxy::Origin::new(url))) },
        || {
            PutAsset::cached(INDEX_PATH)
                .content_type("application/json")
                .attributes(json!({ ID: { "remote": remote } }))
        },
    )
    .await?
    .ok_or_else(|| ContentError::BadGateway("the remote has no service index".to_owned()))?;
    if let Some((blob, known)) = cache
        .indexes
        .lock()
        .expect("the index cache lock is never poisoned")
        .get(ctx.name())
        && *blob == asset.blob_id
    {
        return Ok(known.clone());
    }
    let value = parse_json(&read_asset(ctx, &asset).await?)?;
    let found = Arc::new(read_index(&value));
    if found.package_base.is_none() && found.registration_base.is_none() {
        return Err(ContentError::BadGateway(
            "the remote's service index names no package or registration resources".to_owned(),
        ));
    }
    cache
        .indexes
        .lock()
        .expect("the index cache lock is never poisoned")
        .insert(ctx.name().to_owned(), (asset.blob_id, found.clone()));
    Ok(found)
}

pub async fn document(ctx: &Ctx, path: &str, remote: &str) -> Result<Option<Value>, ContentError> {
    let url = url::Url::parse(remote).map_err(|error| {
        ContentError::BadGateway(format!("the remote URL is not valid: {error}"))
    })?;
    match cached(ctx, path, url).await? {
        Some(asset) => Ok(Some(parse_json(&read_asset(ctx, &asset).await?)?)),
        None => Ok(None),
    }
}

pub async fn registration(
    ctx: &Ctx,
    cache: &Cache,
    rest: &str,
    urls: &Urls,
) -> Result<Option<Value>, ContentError> {
    let upstream = upstream(ctx, cache).await?;
    let Some(base) = &upstream.registration_base else {
        return Ok(None);
    };
    let path = format!("/v3/registration/{rest}");
    let Some(mut value) = document(ctx, &path, &format!("{base}{rest}")).await? else {
        return Ok(None);
    };
    rewrite(&mut value, &upstream.prefixes(urls));
    Ok(Some(value))
}

pub async fn versions(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
) -> Result<Option<Vec<String>>, ContentError> {
    let upstream = upstream(ctx, cache).await?;
    let Some(base) = &upstream.package_base else {
        return Ok(None);
    };
    let id = id.to_lowercase();
    let found = document(
        ctx,
        &format!("/v3/content/{id}/index.json"),
        &format!("{base}{id}/index.json"),
    )
    .await?;
    Ok(found.map(|value| {
        value
            .get("versions")
            .and_then(Value::as_array)
            .map(|versions| {
                versions
                    .iter()
                    .filter_map(|version| version.as_str().map(str::to_lowercase))
                    .collect()
            })
            .unwrap_or_default()
    }))
}

pub async fn nuspec(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    version: &str,
) -> Result<Option<Asset>, ContentError> {
    let upstream = upstream(ctx, cache).await?;
    let Some(base) = &upstream.package_base else {
        return Ok(None);
    };
    let id = id.to_lowercase();
    let version = version.to_lowercase();
    let url = url::Url::parse(&format!("{base}{id}/{version}/{id}.nuspec"))
        .map_err(|error| ContentError::BadGateway(error.to_string()))?;
    let path = format!("/v3/content/{id}/{version}/{id}.nuspec");
    proxy::fetch_from(
        ctx,
        &path,
        Freshness::Metadata,
        || async move { Ok(Some(proxy::Origin::new(url))) },
        || PutAsset::cached(path.clone()).content_type("application/xml"),
    )
    .await
}

pub async fn package(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    version: &str,
) -> Result<Option<Asset>, ContentError> {
    let path = package_path(id, version);
    let lower_id = id.to_lowercase();
    let lower_version = version.to_lowercase();
    let url = match mode(ctx) {
        Mode::V3 => {
            let existing = store::find(ctx.db(), ctx.name(), &path).await?;
            let upstream = match upstream(ctx, cache).await {
                Ok(upstream) => upstream,
                Err(error) => {
                    return match existing {
                        Some(asset) => Ok(Some(asset)),
                        None => Err(error),
                    };
                }
            };
            let Some(base) = &upstream.package_base else {
                return Ok(existing);
            };
            format!("{base}{lower_id}/{lower_version}/{lower_id}.{lower_version}.nupkg")
        }
        Mode::V2 => format!("{}package/{id}/{version}", v2_root(ctx)?),
    };
    let url = url::Url::parse(&url).map_err(|error| ContentError::BadGateway(error.to_string()))?;
    let found = proxy::fetch_from(
        ctx,
        &path,
        Freshness::Content,
        || async move { Ok(Some(proxy::Origin::new(url))) },
        || {
            PutAsset::cached(path.clone())
                .component(ComponentKey {
                    namespace: String::new(),
                    name: lower_id.clone(),
                    version: lower_version.clone(),
                    kind: PACKAGE_KIND.to_owned(),
                    attributes: json!({}),
                })
                .kind(PACKAGE_KIND)
                .content_type(ZIP)
                .attributes(json!({ ID: {} }))
        },
    )
    .await?;
    if let Some(asset) = &found
        && let Err(error) = enrich(ctx, asset).await
    {
        tracing::info!(repository = ctx.name(), path = asset.path, %error, "the cached package keeps no metadata");
    }
    Ok(found)
}

async fn enrich(ctx: &Ctx, asset: &Asset) -> Result<(), ContentError> {
    let Some(component_id) = asset.component_id else {
        return Ok(());
    };
    let Some(found) = store::component_by_id(ctx.db(), component_id).await? else {
        return Ok(());
    };
    if found.attributes.get(ID).is_some() {
        return Ok(());
    }
    let bytes = hosted::nuspec_of(ctx, asset).await?;
    let metadata =
        parse_nuspec(&bytes).map_err(|error| ContentError::BadGateway(error.to_string()))?;
    component::Entity::update(component::ActiveModel {
        id: ActiveValue::Unchanged(component_id),
        name: ActiveValue::Set(metadata.id.clone()),
        version: ActiveValue::Set(metadata.normalized_version.clone()),
        attributes: ActiveValue::Set(hosted::attributes(&metadata)),
        updated_at: ActiveValue::Set(chrono::Utc::now().fixed_offset()),
        ..Default::default()
    })
    .exec(ctx.db())
    .await?;
    Ok(())
}

fn query_age(ctx: &Ctx) -> Duration {
    let seconds = ctx
        .repository
        .attributes
        .nuget_proxy
        .as_ref()
        .and_then(|nuget| nuget.query_cache_item_max_age)
        .unwrap_or(DEFAULT_QUERY_SECONDS)
        .max(0);
    Duration::from_secs(seconds as u64)
}

pub async fn forward(
    ctx: &Ctx,
    cache: &Cache,
    url: url::Url,
) -> Result<Option<Arc<Vec<u8>>>, ContentError> {
    let key = (ctx.name().to_owned(), url.to_string());
    let age = query_age(ctx);
    if !age.is_zero()
        && let Some(body) = cache.query(&key, age)
    {
        return Ok(Some(body));
    }
    let remote = proxy::request(ctx, url, &[]).await?;
    match remote.status {
        StatusCode::OK => {
            let body = Arc::new(inflate(remote.bytes(MAX_DOCUMENT).await?));
            if !age.is_zero() {
                cache.remember(key, body.clone());
            }
            Ok(Some(body))
        }
        StatusCode::NOT_FOUND | StatusCode::GONE => Ok(None),
        status => Err(ContentError::BadGateway(format!(
            "the remote answered {status}"
        ))),
    }
}

pub async fn search(
    ctx: &Ctx,
    cache: &Cache,
    autocomplete: bool,
    query: Option<&str>,
    urls: &Urls,
) -> Result<Option<Value>, ContentError> {
    let upstream = upstream(ctx, cache).await?;
    let service = match autocomplete {
        true => &upstream.autocomplete,
        false => &upstream.search,
    };
    let Some(service) = service else {
        return Ok(None);
    };
    let mut url =
        url::Url::parse(service).map_err(|error| ContentError::BadGateway(error.to_string()))?;
    url.set_query(query.filter(|query| !query.is_empty()));
    let Some(body) = forward(ctx, cache, url).await? else {
        return Ok(None);
    };
    let mut value = parse_json(&body)?;
    rewrite(&mut value, &upstream.prefixes(urls));
    Ok(Some(value))
}

pub fn v2_url(ctx: &Ctx, path: &str, query: Option<&str>) -> Result<url::Url, ContentError> {
    let mut joined = format!("{}{}", v2_root(ctx)?, path.trim_start_matches('/'));
    if let Some(query) = query.filter(|query| !query.is_empty()) {
        joined.push('?');
        joined.push_str(query);
    }
    url::Url::parse(&joined).map_err(|error| ContentError::BadGateway(error.to_string()))
}

pub fn local_next(ctx: &Ctx, next: &str, urls: &Urls) -> Option<String> {
    let root = url::Url::parse(&v2_root(ctx).ok()?).ok()?;
    let found = url::Url::parse(next).or_else(|_| root.join(next)).ok()?;
    let rest = found.path().strip_prefix(root.path())?;
    let mut local = format!("{}{rest}", urls.root());
    if let Some(query) = found.query() {
        local.push('?');
        local.push_str(query);
    }
    Some(local)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nuget_org() -> Value {
        json!({"version": "3.0.0", "resources": [
            {"@id": "https://api.nuget.org/v3-flatcontainer/", "@type": "PackageBaseAddress/3.0.0"},
            {"@id": "https://api.nuget.org/v3/registration5-semver1/", "@type": "RegistrationsBaseUrl"},
            {"@id": "https://api.nuget.org/v3/registration5-gz-semver1/", "@type": "RegistrationsBaseUrl/3.4.0"},
            {"@id": "https://api.nuget.org/v3/registration5-gz-semver2/", "@type": "RegistrationsBaseUrl/3.6.0"},
            {"@id": "https://azuresearch-usnc.nuget.org/query", "@type": "SearchQueryService"},
            {"@id": "https://azuresearch-usnc.nuget.org/query", "@type": "SearchQueryService/3.5.0"},
            {"@id": "https://azuresearch-usnc.nuget.org/autocomplete", "@type": ["SearchAutocompleteService", "SearchAutocompleteService/3.5.0"]}
        ]})
    }

    #[test]
    fn the_service_index_names_the_preferred_resources() {
        let upstream = read_index(&nuget_org());
        assert_eq!(
            upstream.package_base.as_deref(),
            Some("https://api.nuget.org/v3-flatcontainer/")
        );
        assert_eq!(
            upstream.registration_base.as_deref(),
            Some("https://api.nuget.org/v3/registration5-gz-semver2/")
        );
        assert_eq!(upstream.registration_bases.len(), 3);
        assert_eq!(
            upstream.search.as_deref(),
            Some("https://azuresearch-usnc.nuget.org/query")
        );
        assert_eq!(
            upstream.autocomplete.as_deref(),
            Some("https://azuresearch-usnc.nuget.org/autocomplete")
        );
        assert_eq!(read_index(&json!({})), Upstream::default());
    }

    #[test]
    fn remote_urls_are_rewritten_to_the_proxy() {
        let upstream = read_index(&nuget_org());
        let urls = Urls::new("http://h/repository/p");
        let mut value = json!({
            "@id": "https://api.nuget.org/v3/registration5-gz-semver2/a/index.json",
            "items": [{
                "packageContent": "https://api.nuget.org/v3-flatcontainer/a/1.0.0/a.1.0.0.nupkg",
                "registration": "https://api.nuget.org/v3/registration5-semver1/a/index.json",
                "catalogEntry": {"@id": "https://api.nuget.org/v3/catalog0/data/a.json", "count": 3}
            }]
        });
        rewrite(&mut value, &upstream.prefixes(&urls));
        assert_eq!(
            value["@id"],
            "http://h/repository/p/v3/registration/a/index.json"
        );
        assert_eq!(
            value["items"][0]["packageContent"],
            "http://h/repository/p/v3/content/a/1.0.0/a.1.0.0.nupkg"
        );
        assert_eq!(
            value["items"][0]["registration"],
            "http://h/repository/p/v3/registration/a/index.json"
        );
        assert_eq!(
            value["items"][0]["catalogEntry"]["@id"],
            "https://api.nuget.org/v3/catalog0/data/a.json"
        );
    }

    #[test]
    fn gzip_documents_are_inflated() {
        use std::io::Write;
        let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        encoder.write_all(b"{\"a\":1}").unwrap();
        let packed = encoder.finish().unwrap();
        assert_eq!(inflate(packed), b"{\"a\":1}");
        assert_eq!(inflate(b"plain".to_vec()), b"plain");
    }
}
