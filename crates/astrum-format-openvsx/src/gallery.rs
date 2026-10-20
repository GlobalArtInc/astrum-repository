use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::model::{Ext, Release, Urls, time};
use crate::names::{self, UNIVERSAL};
use crate::vsix::asset_type_of;

pub const CONTENT_TYPE: &str = "application/json;api-version=3.0-preview.1";

pub const FLAG_VERSIONS: u64 = 0x1;
pub const FLAG_VERSION_PROPERTIES: u64 = 0x10;
pub const FLAG_LATEST_ONLY: u64 = 0x200;
pub const ALL_FLAGS: u64 = 0x3B7;

const TAG: u64 = 1;
const EXTENSION_ID: u64 = 4;
const CATEGORY: u64 = 5;
const EXTENSION_NAME: u64 = 7;
const SEARCH_TEXT: u64 = 10;
const MAX_PAGE: usize = 1000;
const DEFAULT_PAGE: usize = 50;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Criterion {
    pub kind: u64,
    pub value: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Filter {
    pub criteria: Vec<Criterion>,
    pub page_number: usize,
    pub page_size: usize,
    pub sort_by: u64,
    pub sort_order: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub filters: Vec<Filter>,
    pub flags: u64,
    pub asset_types: Vec<String>,
}

fn number(value: Option<&Value>) -> Option<u64> {
    match value? {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
}

impl Query {
    pub fn parse(value: &Value) -> Result<Query, String> {
        let Some(object) = value.as_object() else {
            return Err("the gallery query must be a JSON object".to_owned());
        };
        let mut filters = Vec::new();
        for filter in object
            .get("filters")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            let criteria = filter
                .get("criteria")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|criterion| {
                    Some(Criterion {
                        kind: number(criterion.get("filterType"))?,
                        value: match criterion.get("value")? {
                            Value::String(text) => text.clone(),
                            other => other.to_string(),
                        },
                    })
                })
                .collect();
            filters.push(Filter {
                criteria,
                page_number: number(filter.get("pageNumber"))
                    .map(|page| page.max(1) as usize)
                    .unwrap_or(1),
                page_size: number(filter.get("pageSize"))
                    .map(|size| (size as usize).clamp(1, MAX_PAGE))
                    .unwrap_or(DEFAULT_PAGE),
                sort_by: number(filter.get("sortBy")).unwrap_or(0),
                sort_order: number(filter.get("sortOrder")).unwrap_or(0),
            });
        }
        if filters.is_empty() {
            filters.push(Filter {
                criteria: Vec::new(),
                page_number: 1,
                page_size: DEFAULT_PAGE,
                sort_by: 0,
                sort_order: 0,
            });
        }
        Ok(Query {
            filters,
            flags: number(object.get("flags")).unwrap_or(0),
            asset_types: object
                .get("assetTypes")
                .and_then(Value::as_array)
                .map(|types| {
                    types
                        .iter()
                        .filter_map(|kind| kind.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    pub fn to_value(&self) -> Value {
        json!({
            "filters": self.filters.iter().map(|filter| json!({
                "criteria": filter.criteria.iter().map(|criterion| json!({"filterType": criterion.kind, "value": criterion.value})).collect::<Vec<_>>(),
                "pageNumber": filter.page_number,
                "pageSize": filter.page_size,
                "sortBy": filter.sort_by,
                "sortOrder": filter.sort_order,
            })).collect::<Vec<_>>(),
            "assetTypes": self.asset_types,
            "flags": self.flags,
        })
    }

    pub fn widened(&self) -> Query {
        let mut wide = self.clone();
        for filter in &mut wide.filters {
            filter.page_size = (filter.page_number * filter.page_size).min(MAX_PAGE);
            filter.page_number = 1;
        }
        wide
    }

    pub fn single(namespace: &str, name: &str, flags: u64) -> Query {
        Query {
            filters: vec![Filter {
                criteria: vec![Criterion {
                    kind: EXTENSION_NAME,
                    value: names::id(namespace, name),
                }],
                page_number: 1,
                page_size: 1,
                sort_by: 0,
                sort_order: 0,
            }],
            flags,
            asset_types: Vec::new(),
        }
    }
}

fn includes_versions(flags: u64) -> bool {
    flags & (FLAG_VERSIONS | FLAG_VERSION_PROPERTIES | FLAG_LATEST_ONLY) != 0
}

fn words(text: &str) -> Vec<String> {
    text.split(|c: char| c.is_whitespace() || c == ',')
        .map(|word| word.trim_matches('"').to_lowercase())
        .filter(|word| !word.is_empty())
        .collect()
}

fn haystack(ext: &Ext<'_>) -> String {
    let mut text = names::id(ext.namespace, ext.name).to_lowercase();
    if let Some(release) = ext.releases.first() {
        for part in [&release.meta.display_name, &release.meta.description]
            .into_iter()
            .flatten()
        {
            text.push(' ');
            text.push_str(&part.to_lowercase());
        }
        for tag in &release.meta.tags {
            text.push(' ');
            text.push_str(&tag.to_lowercase());
        }
    }
    text
}

fn matches(ext: &Ext<'_>, filter: &Filter) -> bool {
    let of = |kind: u64| -> Vec<&str> {
        filter
            .criteria
            .iter()
            .filter(|criterion| criterion.kind == kind)
            .map(|criterion| criterion.value.as_str())
            .collect()
    };
    let wanted_names = of(EXTENSION_NAME);
    let wanted_ids = of(EXTENSION_ID);
    if !wanted_names.is_empty() || !wanted_ids.is_empty() {
        let key = ext.key();
        let uuid = names::extension_uuid(ext.namespace, ext.name);
        let named = wanted_names.iter().any(|name| name.to_lowercase() == key);
        let identified = wanted_ids.iter().any(|id| id.eq_ignore_ascii_case(&uuid));
        if !named && !identified {
            return false;
        }
    }
    let Some(release) = ext.releases.first() else {
        return false;
    };
    let has =
        |list: &[String], wanted: &str| list.iter().any(|item| item.eq_ignore_ascii_case(wanted));
    if !of(CATEGORY)
        .iter()
        .all(|category| has(&release.meta.categories, category))
    {
        return false;
    }
    if !of(TAG).iter().all(|tag| has(&release.meta.tags, tag)) {
        return false;
    }
    let text = haystack(ext);
    of(SEARCH_TEXT)
        .iter()
        .flat_map(|search| words(search))
        .filter(|word| !word.starts_with('@') && !word.contains(':'))
        .all(|word| text.contains(&word))
}

fn relevance(ext: &Ext<'_>, filter: &Filter) -> usize {
    let key = ext.key();
    let name = ext.name.to_lowercase();
    filter
        .criteria
        .iter()
        .filter(|criterion| criterion.kind == SEARCH_TEXT)
        .flat_map(|criterion| words(&criterion.value))
        .map(|word| match word {
            word if word == key || word == name => 3,
            word if name.contains(&word) => 2,
            _ => 1,
        })
        .sum()
}

fn sort(exts: &mut [&Ext<'_>], filter: &Filter) {
    let ascending = filter.sort_order == 1;
    exts.sort_by(|a, b| {
        let ordering = match filter.sort_by {
            1 => a.last_published().cmp(&b.last_published()),
            2 => a
                .display_name()
                .to_lowercase()
                .cmp(&b.display_name().to_lowercase()),
            3 => a.namespace.to_lowercase().cmp(&b.namespace.to_lowercase()),
            5 => a.first_published().cmp(&b.first_published()),
            4 | 6 | 12 => a.downloads().cmp(&b.downloads()),
            _ => {
                return relevance(b, filter)
                    .cmp(&relevance(a, filter))
                    .then_with(|| b.downloads().cmp(&a.downloads()))
                    .then_with(|| a.key().cmp(&b.key()));
            }
        };
        let ordering = match ascending {
            true => ordering,
            false => ordering.reverse(),
        };
        ordering.then_with(|| a.key().cmp(&b.key()))
    });
}

fn property(key: &str, value: impl Into<String>) -> Value {
    json!({"key": key, "value": value.into()})
}

fn properties(release: &Release) -> Value {
    let meta = &release.meta;
    let mut list = vec![
        property(
            "Microsoft.VisualStudio.Services.Branding.Color",
            meta.gallery_color.clone().unwrap_or_default(),
        ),
        property(
            "Microsoft.VisualStudio.Services.Branding.Theme",
            meta.gallery_theme.clone().unwrap_or_default(),
        ),
        property(
            "Microsoft.VisualStudio.Services.Links.Source",
            meta.repository.clone().unwrap_or_default(),
        ),
        property(
            "Microsoft.VisualStudio.Code.SponsorLink",
            meta.sponsor_link.clone().unwrap_or_default(),
        ),
        property(
            "Microsoft.VisualStudio.Code.Engine",
            meta.engines.get("vscode").cloned().unwrap_or_default(),
        ),
        property(
            "Microsoft.VisualStudio.Code.ExtensionDependencies",
            meta.dependencies.join(","),
        ),
        property(
            "Microsoft.VisualStudio.Code.ExtensionPack",
            meta.bundled_extensions.join(","),
        ),
        property(
            "Microsoft.VisualStudio.Code.ExtensionKind",
            meta.extension_kind.join(","),
        ),
        property(
            "Microsoft.VisualStudio.Code.LocalizedLanguages",
            meta.localized_languages.join(","),
        ),
    ];
    if release.is_pre_release() {
        list.push(property("Microsoft.VisualStudio.Code.PreRelease", "true"));
    }
    if meta.web {
        list.push(property("Microsoft.VisualStudio.Code.WebExtension", "true"));
    }
    Value::Array(list)
}

fn version_entry(urls: &Urls, release: &Release, asset_types: &[String]) -> Value {
    let mut files = Vec::new();
    for kind in [
        "manifest",
        "readme",
        "changelog",
        "license",
        "icon",
        "download",
        "vsixmanifest",
    ] {
        let (Some(asset_type), Some(url)) = (asset_type_of(kind), urls.kind(release, kind)) else {
            continue;
        };
        if !asset_types.is_empty()
            && !asset_types
                .iter()
                .any(|wanted| wanted.eq_ignore_ascii_case(asset_type))
        {
            continue;
        }
        files.push(json!({"assetType": asset_type, "source": url}));
    }
    let asset_uri = urls.asset(&release.namespace, &release.name, &release.version);
    json!({
        "version": release.version,
        "targetPlatform": release.platform,
        "flags": "validated",
        "lastUpdated": time(release.published),
        "assetUri": asset_uri,
        "fallbackAssetUri": asset_uri,
        "files": files,
        "properties": properties(release),
    })
}

fn latest_per_platform<'a>(releases: &[&'a Release]) -> Vec<&'a Release> {
    let mut chosen: BTreeMap<&str, &Release> = BTreeMap::new();
    for release in releases {
        let slot = chosen.entry(release.platform.as_str()).or_insert(release);
        if names::compare_versions(&release.version, &slot.version) == Ordering::Greater {
            *slot = release;
        }
    }
    let mut latest: Vec<&Release> = chosen.into_values().collect();
    latest.sort_by(|a, b| crate::model::order(a, b));
    latest
}

pub fn extension(urls: &Urls, ext: &Ext<'_>, flags: u64, asset_types: &[String]) -> Value {
    let first = ext.latest(None).or(ext.releases.first().copied());
    let meta = first
        .map(|release| release.meta.clone())
        .unwrap_or_default();
    let mut object = Map::new();
    object.insert(
        "extensionId".into(),
        json!(names::extension_uuid(ext.namespace, ext.name)),
    );
    object.insert("extensionName".into(), json!(ext.name));
    object.insert("displayName".into(), json!(ext.display_name()));
    object.insert(
        "shortDescription".into(),
        json!(meta.description.unwrap_or_default()),
    );
    object.insert(
        "publisher".into(),
        json!({
            "publisherId": names::namespace_uuid(ext.namespace),
            "publisherName": ext.namespace,
            "displayName": ext.namespace,
            "domain": null,
            "isDomainVerified": false,
        }),
    );
    let versions: Vec<Value> = match (includes_versions(flags), flags & FLAG_LATEST_ONLY != 0) {
        (false, _) => Vec::new(),
        (true, true) => latest_per_platform(&ext.releases)
            .into_iter()
            .map(|release| version_entry(urls, release, asset_types))
            .collect(),
        (true, false) => ext
            .releases
            .iter()
            .map(|release| version_entry(urls, release, asset_types))
            .collect(),
    };
    object.insert("versions".into(), Value::Array(versions));
    object.insert(
        "statistics".into(),
        json!([{"statisticName": "install", "value": ext.downloads() as f64}]),
    );
    object.insert("tags".into(), json!(meta.tags));
    object.insert("categories".into(), json!(meta.categories));
    object.insert("releaseDate".into(), json!(time(ext.first_published())));
    object.insert("publishedDate".into(), json!(time(ext.first_published())));
    object.insert("lastUpdated".into(), json!(time(ext.last_published())));
    object.insert(
        "flags".into(),
        json!(match meta.preview {
            true => "validated, public, preview",
            false => "validated, public",
        }),
    );
    object.insert("deploymentType".into(), json!(0));
    Value::Object(object)
}

fn result(extensions: Vec<Value>, total: usize) -> Value {
    json!({
        "extensions": extensions,
        "pagingToken": null,
        "resultMetadata": [{
            "metadataType": "ResultCount",
            "metadataItems": [{"name": "TotalCount", "count": total}],
        }],
    })
}

pub fn answer(urls: &Urls, exts: &[Ext<'_>], query: &Query) -> Value {
    let results: Vec<Value> = query
        .filters
        .iter()
        .map(|filter| {
            let mut found: Vec<&Ext<'_>> = exts.iter().filter(|ext| matches(ext, filter)).collect();
            sort(&mut found, filter);
            let total = found.len();
            let page: Vec<Value> = found
                .into_iter()
                .skip((filter.page_number - 1) * filter.page_size)
                .take(filter.page_size)
                .map(|ext| extension(urls, ext, query.flags, &query.asset_types))
                .collect();
            result(page, total)
        })
        .collect();
    json!({"results": results})
}

fn identity(extension: &Value) -> Option<String> {
    Some(
        names::id(
            extension.pointer("/publisher/publisherName")?.as_str()?,
            extension.get("extensionName")?.as_str()?,
        )
        .to_lowercase(),
    )
}

fn version_key(entry: &Value) -> (String, String) {
    (
        entry
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        entry
            .get("targetPlatform")
            .and_then(Value::as_str)
            .unwrap_or(UNIVERSAL)
            .to_owned(),
    )
}

fn order_entries(entries: &mut [Value]) {
    entries.sort_by(|a, b| {
        let (a_version, a_platform) = version_key(a);
        let (b_version, b_platform) = version_key(b);
        names::compare_versions(&b_version, &a_version)
            .then_with(|| names::platform_rank(&a_platform).cmp(&names::platform_rank(&b_platform)))
    });
}

fn latest_entries(entries: Vec<Value>) -> Vec<Value> {
    let mut chosen: BTreeMap<String, Value> = BTreeMap::new();
    for entry in entries {
        let (version, platform) = version_key(&entry);
        match chosen.get(&platform) {
            Some(known)
                if names::compare_versions(&version, &version_key(known).0)
                    != Ordering::Greater => {}
            _ => {
                chosen.insert(platform, entry);
            }
        }
    }
    let mut latest: Vec<Value> = chosen.into_values().collect();
    order_entries(&mut latest);
    latest
}

pub fn merge_extensions(found: Vec<Value>, latest_only: bool) -> Vec<Value> {
    let mut order: Vec<String> = Vec::new();
    let mut merged: BTreeMap<String, (Value, Vec<Value>)> = BTreeMap::new();
    for extension in found {
        let Some(key) = identity(&extension) else {
            continue;
        };
        let entries: Vec<Value> = extension
            .get("versions")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        match merged.get_mut(&key) {
            Some((_, known)) => {
                for entry in entries {
                    let wanted = version_key(&entry);
                    if !known.iter().any(|existing| version_key(existing) == wanted) {
                        known.push(entry);
                    }
                }
            }
            None => {
                order.push(key.clone());
                merged.insert(key, (extension, entries));
            }
        }
    }
    order
        .into_iter()
        .filter_map(|key| merged.remove(&key))
        .map(|(mut extension, mut entries)| {
            order_entries(&mut entries);
            if latest_only {
                entries = latest_entries(entries);
            }
            if let Some(object) = extension.as_object_mut() {
                object.insert("versions".into(), Value::Array(entries));
            }
            extension
        })
        .collect()
}

pub fn merge(query: &Query, answers: Vec<Value>) -> Value {
    let latest_only = query.flags & FLAG_LATEST_ONLY != 0;
    let results: Vec<Value> = query
        .filters
        .iter()
        .enumerate()
        .map(|(index, filter)| {
            let mut found = Vec::new();
            let mut total = 0usize;
            for answer in &answers {
                let Some(result) = answer.pointer(&format!("/results/{index}")) else {
                    continue;
                };
                total += result
                    .pointer("/resultMetadata/0/metadataItems/0/count")
                    .and_then(Value::as_u64)
                    .unwrap_or(0) as usize;
                found.extend(
                    result
                        .get("extensions")
                        .and_then(Value::as_array)
                        .cloned()
                        .unwrap_or_default(),
                );
            }
            let given = found.len();
            let merged = merge_extensions(found, latest_only);
            let total = total.saturating_sub(given - merged.len()).max(merged.len());
            let page: Vec<Value> = merged
                .into_iter()
                .skip((filter.page_number - 1) * filter.page_size)
                .take(filter.page_size)
                .collect();
            result(page, total)
        })
        .collect();
    json!({"results": results})
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queries_parse_and_widen_for_group_members() {
        let query = Query::parse(&json!({
            "filters": [{
                "criteria": [
                    {"filterType": 8, "value": "Microsoft.VisualStudio.Code"},
                    {"filterType": 10, "value": "hello"},
                    {"filterType": 12, "value": "4096"}
                ],
                "pageNumber": 3,
                "pageSize": 10,
                "sortBy": 4,
                "sortOrder": 0
            }],
            "assetTypes": [],
            "flags": 950
        }))
        .unwrap();
        assert_eq!(query.flags, 950);
        assert_eq!(query.filters[0].criteria.len(), 3);
        let wide = query.widened();
        assert_eq!(wide.filters[0].page_number, 1);
        assert_eq!(wide.filters[0].page_size, 30);
        assert_eq!(Query::parse(&wide.to_value()).unwrap(), wide);
        assert!(Query::parse(&json!([])).is_err());
        assert_eq!(Query::parse(&json!({})).unwrap().filters.len(), 1);
    }

    #[test]
    fn merged_results_keep_the_first_member_per_version_and_platform() {
        let extension = |member: &str, versions: &[(&str, &str)]| {
            json!({
                "extensionName": "hello",
                "publisher": {"publisherName": "acme"},
                "member": member,
                "versions": versions.iter().map(|(version, platform)| json!({"version": version, "targetPlatform": platform, "member": member})).collect::<Vec<_>>(),
            })
        };
        let first = json!({"results": [{"extensions": [extension("own", &[("1.0.0", "universal")])], "resultMetadata": [{"metadataType": "ResultCount", "metadataItems": [{"name": "TotalCount", "count": 1}]}]}]});
        let second = json!({"results": [{"extensions": [extension("far", &[("1.0.0", "universal"), ("2.0.0", "linux-x64"), ("0.9.0", "linux-x64")]), json!({"extensionName": "other", "publisher": {"publisherName": "acme"}, "versions": []})], "resultMetadata": [{"metadataType": "ResultCount", "metadataItems": [{"name": "TotalCount", "count": 2}]}]}]});
        let query = Query::parse(
            &json!({"filters": [{"criteria": [], "pageNumber": 1, "pageSize": 10}], "flags": 1}),
        )
        .unwrap();
        let merged = merge(&query, vec![first.clone(), second.clone()]);
        let result = &merged["results"][0];
        assert_eq!(result["resultMetadata"][0]["metadataItems"][0]["count"], 2);
        let extensions = result["extensions"].as_array().unwrap();
        assert_eq!(extensions.len(), 2);
        assert_eq!(extensions[0]["member"], "own");
        let versions: Vec<(String, String)> = extensions[0]["versions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| {
                (
                    entry["version"].as_str().unwrap().to_owned(),
                    entry["member"].as_str().unwrap().to_owned(),
                )
            })
            .collect();
        assert_eq!(
            versions,
            [
                ("2.0.0".to_owned(), "far".to_owned()),
                ("1.0.0".to_owned(), "own".to_owned()),
                ("0.9.0".to_owned(), "far".to_owned())
            ]
        );
        let latest =
            Query::parse(&json!({"filters": [{"criteria": []}], "flags": FLAG_LATEST_ONLY}))
                .unwrap();
        let merged = merge(&latest, vec![first, second]);
        let versions = merged["results"][0]["extensions"][0]["versions"]
            .as_array()
            .unwrap()
            .clone();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0]["version"], "2.0.0");
        assert_eq!(versions[1]["version"], "1.0.0");
        assert_eq!(versions[1]["member"], "own");
    }
}
