use std::collections::{BTreeMap, HashSet};

use astrum_content::{Asset, ContentError, Ctx, group, store};
use astrum_repo::RepositoryType;
use serde_json::Value;

use crate::hosted;
use crate::odata::{self, Entry, Request};
use crate::registration;
use crate::remote::{self, Cache, Mode};
use crate::route::{Feed, symbols_path};
use crate::search::{self, Found, Query};
use crate::urls::Urls;
use crate::version::{self, Version};

const MAX_MERGED_PAGES: usize = 64;

async fn members(ctx: &Ctx) -> Result<Vec<Ctx>, ContentError> {
    Ok(group::members(ctx)
        .await?
        .into_iter()
        .map(|member| ctx.member(member))
        .collect())
}

fn skipped(ctx: &Ctx, member: &Ctx, error: &ContentError) {
    tracing::info!(group = ctx.name(), member = member.name(), %error, "a group member failed; using the others");
}

fn leaf_mode(ctx: &Ctx) -> Option<Mode> {
    match ctx.kind() {
        RepositoryType::Proxy => Some(remote::mode(ctx)),
        _ => None,
    }
}

async fn leaf_versions(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
) -> Result<Option<Vec<String>>, ContentError> {
    match leaf_mode(ctx) {
        None => {
            let found = hosted::versions(ctx, id).await?;
            Ok((!found.is_empty()).then(|| {
                found
                    .iter()
                    .map(|stored| stored.metadata.lower_version())
                    .collect()
            }))
        }
        Some(Mode::V3) => remote::versions(ctx, cache, id).await,
        Some(Mode::V2) => {
            let request = Request::parse(
                Feed::FindPackagesById,
                false,
                &[
                    ("id".to_owned(), format!("'{id}'")),
                    ("semVerLevel".to_owned(), "2.0.0".to_owned()),
                ],
            );
            let entries = v2_leaf(ctx, cache, &request, "FindPackagesById()", None)
                .await?
                .map(|found| found.entries)
                .unwrap_or_default();
            Ok((!entries.is_empty()).then(|| {
                entries
                    .iter()
                    .map(|entry| entry.version().to_lowercase())
                    .collect()
            }))
        }
    }
}

pub async fn versions(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
) -> Result<Option<Vec<String>>, ContentError> {
    if ctx.kind() != RepositoryType::Group {
        return leaf_versions(ctx, cache, id).await;
    }
    let mut merged: Vec<String> = Vec::new();
    let mut any = false;
    for member in members(ctx).await? {
        match leaf_versions(&member, cache, id).await {
            Ok(Some(found)) => {
                any = true;
                for version in found {
                    if !merged.iter().any(|known| version::same(known, &version)) {
                        merged.push(version);
                    }
                }
            }
            Ok(None) => {}
            Err(error) => skipped(ctx, &member, &error),
        }
    }
    merged.sort_by(|left, right| version::compare_text(left, right));
    Ok(any.then_some(merged))
}

async fn leaf_registration_index(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    urls: &Urls,
) -> Result<Option<Value>, ContentError> {
    match leaf_mode(ctx) {
        None => {
            let items: Vec<Value> = hosted::versions(ctx, id)
                .await?
                .iter()
                .map(|stored| registration::item(stored, urls))
                .collect();
            Ok(registration::index(id, items, urls))
        }
        Some(Mode::V3) => {
            remote::registration(
                ctx,
                cache,
                &format!("{}/index.json", id.to_lowercase()),
                urls,
            )
            .await
        }
        Some(Mode::V2) => Ok(None),
    }
}

async fn leaf_registration_page(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    lower: &str,
    upper: &str,
    urls: &Urls,
) -> Result<Option<Value>, ContentError> {
    match leaf_mode(ctx) {
        Some(Mode::V3) => {
            remote::registration(
                ctx,
                cache,
                &format!("{}/page/{lower}/{upper}.json", id.to_lowercase()),
                urls,
            )
            .await
        }
        _ => Ok(None),
    }
}

async fn inline_items(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    index: &Value,
    urls: &Urls,
) -> Result<Vec<Value>, ContentError> {
    let mut items = Vec::new();
    let pages = index
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    for page in pages.iter().take(MAX_MERGED_PAGES) {
        match page.get("items").and_then(Value::as_array) {
            Some(inlined) => items.extend(inlined.iter().cloned()),
            None => {
                let (Some(lower), Some(upper)) = (
                    page.get("lower").and_then(Value::as_str),
                    page.get("upper").and_then(Value::as_str),
                ) else {
                    continue;
                };
                if let Some(fetched) =
                    leaf_registration_page(ctx, cache, id, lower, upper, urls).await?
                    && let Some(inlined) = fetched.get("items").and_then(Value::as_array)
                {
                    items.extend(inlined.iter().cloned());
                }
            }
        }
    }
    Ok(items)
}

pub async fn registration_index(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    urls: &Urls,
) -> Result<Option<Value>, ContentError> {
    if ctx.kind() != RepositoryType::Group {
        return leaf_registration_index(ctx, cache, id, urls).await;
    }
    let mut found: Vec<(Ctx, Value)> = Vec::new();
    for member in members(ctx).await? {
        match leaf_registration_index(&member, cache, id, urls).await {
            Ok(Some(index)) => found.push((member, index)),
            Ok(None) => {}
            Err(error) => skipped(ctx, &member, &error),
        }
    }
    if found.len() <= 1 {
        return Ok(found.pop().map(|(_, index)| index));
    }
    let mut merged: Vec<(Version, Value)> = Vec::new();
    for (member, index) in &found {
        for item in inline_items(member, cache, id, index, urls).await? {
            let Some(version) = registration::item_version(&item) else {
                continue;
            };
            if !merged.iter().any(|(known, _)| known.cmp(&version).is_eq()) {
                merged.push((version, item));
            }
        }
    }
    merged.sort_by(|(left, _), (right, _)| left.cmp(right));
    Ok(registration::index(
        id,
        merged.into_iter().map(|(_, item)| item).collect(),
        urls,
    ))
}

pub async fn registration_page(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    lower: &str,
    upper: &str,
    urls: &Urls,
) -> Result<Option<Value>, ContentError> {
    if ctx.kind() != RepositoryType::Group {
        return leaf_registration_page(ctx, cache, id, lower, upper, urls).await;
    }
    for member in members(ctx).await? {
        match leaf_registration_page(&member, cache, id, lower, upper, urls).await {
            Ok(Some(page)) => return Ok(Some(page)),
            Ok(None) => {}
            Err(error) => skipped(ctx, &member, &error),
        }
    }
    Ok(None)
}

async fn leaf_registration_leaf(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    version: &str,
    urls: &Urls,
    catalog: bool,
) -> Result<Option<Value>, ContentError> {
    match leaf_mode(ctx) {
        None => Ok(hosted::one(ctx, id, version)
            .await?
            .map(|stored| match catalog {
                true => registration::catalog_entry(&stored, urls),
                false => registration::leaf(&stored, urls),
            })),
        Some(Mode::V3) if !catalog => {
            remote::registration(
                ctx,
                cache,
                &format!("{}/{}.json", id.to_lowercase(), version.to_lowercase()),
                urls,
            )
            .await
        }
        _ => Ok(None),
    }
}

pub async fn registration_leaf(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    version: &str,
    urls: &Urls,
    catalog: bool,
) -> Result<Option<Value>, ContentError> {
    if ctx.kind() != RepositoryType::Group {
        return leaf_registration_leaf(ctx, cache, id, version, urls, catalog).await;
    }
    for member in members(ctx).await? {
        match leaf_registration_leaf(&member, cache, id, version, urls, catalog).await {
            Ok(Some(found)) => return Ok(Some(found)),
            Ok(None) => {}
            Err(error) => skipped(ctx, &member, &error),
        }
    }
    Ok(None)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum File {
    Package,
    Symbols,
    Nuspec,
}

async fn leaf_file(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    version: &str,
    file: File,
) -> Result<Option<Asset>, ContentError> {
    let normalized = Version::parse(version)
        .map(|parsed| parsed.normalized())
        .unwrap_or_else(|| version.to_owned());
    match (leaf_mode(ctx), file) {
        (None, File::Package | File::Nuspec) => Ok(hosted::one(ctx, id, &normalized)
            .await?
            .and_then(|stored| stored.package)),
        (None, File::Symbols) => Ok(hosted::one(ctx, id, &normalized)
            .await?
            .and_then(|stored| stored.symbols)),
        (Some(_), File::Package | File::Nuspec) => {
            remote::package(ctx, cache, id, &normalized).await
        }
        (Some(_), File::Symbols) => {
            store::find(ctx.db(), ctx.name(), &symbols_path(id, &normalized)).await
        }
    }
}

pub async fn file(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    version: &str,
    file: File,
) -> Result<Option<(Ctx, Asset)>, ContentError> {
    if ctx.kind() != RepositoryType::Group {
        return Ok(leaf_file(ctx, cache, id, version, file)
            .await?
            .map(|asset| (ctx.clone(), asset)));
    }
    for member in members(ctx).await? {
        match leaf_file(&member, cache, id, version, file).await {
            Ok(Some(asset)) => return Ok(Some((member, asset))),
            Ok(None) => {}
            Err(error) => skipped(ctx, &member, &error),
        }
    }
    Ok(None)
}

pub async fn nuspec(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    version: &str,
) -> Result<Option<Vec<u8>>, ContentError> {
    let Some((owner, package)) = file(ctx, cache, id, version, File::Nuspec).await? else {
        return Ok(None);
    };
    match hosted::nuspec_of(&owner, &package).await {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if owner.kind() == RepositoryType::Proxy => {
            tracing::debug!(%error, "asking the remote for the .nuspec instead");
            match remote::nuspec(&owner, cache, id, version).await? {
                Some(asset) => Ok(Some(remote::read_asset(&owner, &asset).await?)),
                None => Ok(None),
            }
        }
        Err(error) => Err(error),
    }
}

async fn leaf_search(
    ctx: &Ctx,
    cache: &Cache,
    query: &Query,
    raw: Option<&str>,
    urls: &Urls,
    autocomplete: bool,
) -> Result<Found, ContentError> {
    match leaf_mode(ctx) {
        None => {
            if autocomplete && let Some(id) = &query.id {
                return Ok(search::autocomplete_versions(
                    hosted::load(ctx, Some(id), &[]).await?,
                    query,
                ));
            }
            let found = hosted::load(ctx, None, &query.free_text()).await?;
            Ok(match autocomplete {
                true => search::autocomplete_ids(found, query),
                false => search::hosted(found, query, urls),
            })
        }
        Some(Mode::V3) => Ok(remote::search(ctx, cache, autocomplete, raw, urls)
            .await?
            .map(|value| Found::from_json(&value))
            .unwrap_or_default()),
        Some(Mode::V2) => Ok(Found::default()),
    }
}

fn identity(item: &Value) -> String {
    match item {
        Value::String(text) => text.to_lowercase(),
        other => other
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_lowercase(),
    }
}

pub async fn search(
    ctx: &Ctx,
    cache: &Cache,
    query: &Query,
    raw: Option<&str>,
    urls: &Urls,
    autocomplete: bool,
) -> Result<Found, ContentError> {
    if ctx.kind() != RepositoryType::Group {
        return leaf_search(ctx, cache, query, raw, urls, autocomplete).await;
    }
    let wide = Query {
        skip: 0,
        take: (query.skip + query.take).min(search::MAX_TAKE),
        ..query.clone()
    };
    let wide_raw = raw.map(|raw| {
        let mut pairs: Vec<(String, String)> = crate::route::query_pairs(Some(raw))
            .into_iter()
            .filter(|(key, _)| {
                !key.eq_ignore_ascii_case("skip") && !key.eq_ignore_ascii_case("take")
            })
            .collect();
        pairs.push(("take".to_owned(), wide.take.to_string()));
        url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(pairs)
            .finish()
    });
    let versions_only = autocomplete && query.id.is_some();
    let mut total = 0;
    let mut seen = HashSet::new();
    let mut data = Vec::new();
    for member in members(ctx).await? {
        match leaf_search(
            &member,
            cache,
            &wide,
            wide_raw.as_deref(),
            urls,
            autocomplete,
        )
        .await
        {
            Ok(found) => {
                let returned = found.data.len();
                let mut fresh = 0;
                for item in found.data {
                    if seen.insert(identity(&item)) {
                        fresh += 1;
                        data.push(item);
                    }
                }
                total += found.total.max(returned).saturating_sub(returned - fresh);
            }
            Err(error) => skipped(ctx, &member, &error),
        }
    }
    if versions_only {
        data.sort_by(|left, right| {
            version::compare_text(
                left.as_str().unwrap_or_default(),
                right.as_str().unwrap_or_default(),
            )
        });
        let total = data.len();
        return Ok(Found { total, data });
    }
    Ok(Found {
        total: total.max(data.len()),
        data: data.into_iter().skip(query.skip).take(query.take).collect(),
    })
}

pub struct V2Found {
    pub entries: Vec<Entry>,
    pub total: usize,
    pub next: Option<String>,
    pub upstream: bool,
}

async fn v2_leaf(
    ctx: &Ctx,
    cache: &Cache,
    request: &Request,
    path: &str,
    raw: Option<&str>,
) -> Result<Option<V2Found>, ContentError> {
    match leaf_mode(ctx) {
        None => {
            let id = match request.feed {
                Feed::FindPackagesById => request.id.clone(),
                _ => None,
            };
            let terms = request
                .search_term
                .as_deref()
                .map(|term| {
                    search::terms(term)
                        .into_iter()
                        .filter_map(|term| match term {
                            search::Term::Any(text) => Some(text),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if request.feed == Feed::FindPackagesById && id.is_none() {
                return Ok(Some(V2Found {
                    entries: Vec::new(),
                    total: 0,
                    next: None,
                    upstream: false,
                }));
            }
            let found = hosted::load(ctx, id.as_deref(), &terms).await?;
            let entries = odata::from_stored(&found);
            Ok(Some(V2Found {
                total: entries.len(),
                entries,
                next: None,
                upstream: false,
            }))
        }
        Some(Mode::V2) => {
            let url = remote::v2_url(ctx, path, raw)?;
            let Some(body) = remote::forward(ctx, cache, url).await? else {
                return Ok(None);
            };
            let parsed = odata::parse_upstream(&body).ok_or_else(|| {
                ContentError::BadGateway("the remote answered with a broken feed".to_owned())
            })?;
            Ok(Some(V2Found {
                total: parsed.entries.len(),
                entries: parsed.entries,
                next: parsed.next,
                upstream: true,
            }))
        }
        Some(Mode::V3) => Ok(None),
    }
}

fn without_paging(raw: Option<&str>, request: &Request) -> String {
    let mut pairs: Vec<(String, String)> = crate::route::query_pairs(raw)
        .into_iter()
        .filter(|(key, _)| !key.eq_ignore_ascii_case("$skip") && !key.eq_ignore_ascii_case("$top"))
        .collect();
    let wanted = request.skip + request.top.unwrap_or(odata::PAGE) + 1;
    pairs.push(("$top".to_owned(), wanted.min(odata::MAX_TOP).to_string()));
    url::form_urlencoded::Serializer::new(String::new())
        .extend_pairs(pairs)
        .finish()
}

pub async fn v2(
    ctx: &Ctx,
    cache: &Cache,
    request: &Request,
    path: &str,
    raw: Option<&str>,
) -> Result<Option<V2Found>, ContentError> {
    if ctx.kind() != RepositoryType::Group {
        let Some(mut found) = v2_leaf(ctx, cache, request, path, raw).await? else {
            return Ok(None);
        };
        if !found.upstream {
            found.entries = request.select(found.entries);
            found.total = found.entries.len();
        }
        return Ok(Some(found));
    }
    let wide = without_paging(raw, request);
    let mut merged: BTreeMap<(String, String), ()> = BTreeMap::new();
    let mut entries = Vec::new();
    let mut any = false;
    for member in members(ctx).await? {
        match v2_leaf(&member, cache, request, path, Some(&wide)).await {
            Ok(Some(found)) => {
                any = true;
                for entry in found.entries {
                    let key = (
                        entry.id().to_lowercase(),
                        Version::parse(entry.version())
                            .map(|version| version.lower())
                            .unwrap_or_else(|| entry.version().to_lowercase()),
                    );
                    if merged.insert(key, ()).is_none() {
                        entries.push(entry);
                    }
                }
            }
            Ok(None) => {}
            Err(error) => skipped(ctx, &member, &error),
        }
    }
    if !any {
        return Ok(None);
    }
    let entries = request.select(entries);
    Ok(Some(V2Found {
        total: entries.len(),
        entries,
        next: None,
        upstream: false,
    }))
}

pub async fn v2_entry(
    ctx: &Ctx,
    cache: &Cache,
    id: &str,
    version: &str,
    path: &str,
) -> Result<Option<Entry>, ContentError> {
    let request = Request::parse(
        Feed::FindPackagesById,
        false,
        &[
            ("id".to_owned(), format!("'{}'", id.replace('\'', "''"))),
            ("semVerLevel".to_owned(), "2.0.0".to_owned()),
        ],
    );
    let leaves = match ctx.kind() {
        RepositoryType::Group => members(ctx).await?,
        _ => vec![ctx.clone()],
    };
    for leaf in leaves {
        let found = match leaf_mode(&leaf) {
            Some(Mode::V2) => v2_leaf(&leaf, cache, &request, path, None).await,
            None => v2_leaf(&leaf, cache, &request, "FindPackagesById()", None)
                .await
                .map(|found| {
                    found.map(|mut found| {
                        found.entries = request.select(found.entries);
                        found
                    })
                }),
            Some(Mode::V3) => Ok(None),
        };
        match found {
            Ok(Some(found)) => {
                if let Some(entry) = found.entries.into_iter().find(|entry| {
                    entry.id().eq_ignore_ascii_case(id) && version::same(entry.version(), version)
                }) {
                    return Ok(Some(entry));
                }
            }
            Ok(None) => {}
            Err(error) if ctx.kind() == RepositoryType::Group => skipped(ctx, &leaf, &error),
            Err(error) => return Err(error),
        }
    }
    Ok(None)
}
