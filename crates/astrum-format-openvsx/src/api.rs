use std::cmp::Ordering;
use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::model::{Ext, Release, Urls, extension_document, search_entry, version_reference};
use crate::names;

const MAX_SIZE: usize = 1000;
const MAX_VERSIONS: usize = 100;

fn pairs(query: Option<&str>) -> Vec<(String, String)> {
    url::form_urlencoded::parse(query.unwrap_or("").as_bytes())
        .map(|(key, value)| (key.into_owned(), value.trim().to_owned()))
        .filter(|(_, value)| !value.is_empty())
        .collect()
}

fn find<'a>(pairs: &'a [(String, String)], key: &str) -> Option<&'a str> {
    pairs
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

fn count(
    pairs: &[(String, String)],
    key: &str,
    default: usize,
    max: usize,
) -> Result<usize, String> {
    match find(pairs, key) {
        None => Ok(default),
        Some(value) => value
            .parse::<usize>()
            .map(|value| value.min(max))
            .map_err(|_| format!("{key} must be a number")),
    }
}

fn platform_of(pairs: &[(String, String)]) -> Result<Option<String>, String> {
    match find(pairs, "targetPlatform") {
        None => Ok(None),
        Some(platform) if names::is_platform(platform) => Ok(Some(platform.to_owned())),
        Some(platform) => Err(format!("the target platform {platform} is not known")),
    }
}

fn encode(pairs: Vec<(&str, String)>) -> String {
    let mut serializer = url::form_urlencoded::Serializer::new(String::new());
    for (key, value) in pairs {
        serializer.append_pair(key, &value);
    }
    serializer.finish()
}

pub fn paging(query: Option<&str>, default: usize) -> Result<(usize, usize), String> {
    let pairs = pairs(query);
    Ok((
        count(&pairs, "offset", 0, usize::MAX)?,
        count(&pairs, "size", default, MAX_VERSIONS)?,
    ))
}

pub fn paging_query(offset: usize, size: usize) -> String {
    encode(vec![
        ("offset", offset.to_string()),
        ("size", size.to_string()),
    ])
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Search {
    pub query: Option<String>,
    pub category: Option<String>,
    pub platform: Option<String>,
    pub size: usize,
    pub offset: usize,
    pub sort_by: String,
    pub sort_order: String,
    pub all_versions: bool,
}

impl Search {
    pub fn parse(query: Option<&str>) -> Result<Search, String> {
        let pairs = pairs(query);
        let sort_by = find(&pairs, "sortBy").unwrap_or("relevance").to_owned();
        if !["relevance", "timestamp", "rating", "downloadCount"].contains(&sort_by.as_str()) {
            return Err(format!("sortBy {sort_by} is not known"));
        }
        let sort_order = find(&pairs, "sortOrder").unwrap_or("desc").to_owned();
        if !["asc", "desc"].contains(&sort_order.as_str()) {
            return Err(format!("sortOrder {sort_order} is not asc or desc"));
        }
        Ok(Search {
            query: find(&pairs, "query").map(str::to_owned),
            category: find(&pairs, "category").map(str::to_owned),
            platform: platform_of(&pairs)?,
            size: count(&pairs, "size", 18, MAX_SIZE)?,
            offset: count(&pairs, "offset", 0, usize::MAX)?,
            sort_by,
            sort_order,
            all_versions: find(&pairs, "includeAllVersions") == Some("true"),
        })
    }

    pub fn to_query(&self) -> String {
        let mut pairs = Vec::new();
        if let Some(query) = &self.query {
            pairs.push(("query", query.clone()));
        }
        if let Some(category) = &self.category {
            pairs.push(("category", category.clone()));
        }
        if let Some(platform) = &self.platform {
            pairs.push(("targetPlatform", platform.clone()));
        }
        pairs.push(("size", self.size.to_string()));
        pairs.push(("offset", self.offset.to_string()));
        pairs.push(("sortBy", self.sort_by.clone()));
        pairs.push(("sortOrder", self.sort_order.clone()));
        pairs.push(("includeAllVersions", self.all_versions.to_string()));
        encode(pairs)
    }

    pub fn widened(&self) -> Search {
        Search {
            size: (self.offset + self.size).min(MAX_SIZE),
            offset: 0,
            ..self.clone()
        }
    }
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

fn named(ext: &Ext<'_>, words: &[String]) -> usize {
    let name = ext.name.to_lowercase();
    words
        .iter()
        .map(|word| match word {
            word if *word == name || *word == ext.key() => 2,
            word if name.contains(word.as_str()) => 1,
            _ => 0,
        })
        .sum()
}

fn page<T>(items: Vec<T>, offset: usize, size: usize) -> Vec<T> {
    items.into_iter().skip(offset).take(size).collect()
}

pub fn search(urls: &Urls, exts: &[Ext<'_>], params: &Search) -> Value {
    let words: Vec<String> = params
        .query
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .map(str::to_lowercase)
        .collect();
    let platform = params.platform.as_deref();
    let mut found: Vec<(&Ext<'_>, &Release)> = exts
        .iter()
        .filter(|ext| {
            let text = haystack(ext);
            words.iter().all(|word| text.contains(word.as_str()))
        })
        .filter(|ext| {
            params.category.as_deref().is_none_or(|category| {
                ext.releases.first().is_some_and(|release| {
                    release
                        .meta
                        .categories
                        .iter()
                        .any(|known| known.eq_ignore_ascii_case(category))
                })
            })
        })
        .filter_map(|ext| Some((ext, ext.latest(platform)?)))
        .collect();
    let ascending = params.sort_order == "asc";
    found.sort_by(|(a, a_release), (b, b_release)| {
        let ordering = match params.sort_by.as_str() {
            "timestamp" => a_release.published.cmp(&b_release.published),
            "rating" | "downloadCount" => a.downloads().cmp(&b.downloads()),
            _ => named(a, &words)
                .cmp(&named(b, &words))
                .then_with(|| a.downloads().cmp(&b.downloads())),
        };
        let ordering = match ascending {
            true => ordering,
            false => ordering.reverse(),
        };
        ordering.then_with(|| a.key().cmp(&b.key()))
    });
    let total = found.len();
    let extensions: Vec<Value> = page(found, params.offset, params.size)
        .into_iter()
        .map(|(ext, release)| search_entry(urls, ext, release, params.all_versions))
        .collect();
    json!({"offset": params.offset, "totalSize": total, "extensions": extensions})
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AllVersions {
    No,
    Yes,
    Links,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Lookup {
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub id: Option<String>,
    pub extension_uuid: Option<String>,
    pub namespace_uuid: Option<String>,
    pub all_versions: AllVersions,
    pub platform: Option<String>,
    pub size: usize,
    pub offset: usize,
}

impl Lookup {
    pub fn parse(query: Option<&str>, v2: bool) -> Result<Lookup, String> {
        let pairs = pairs(query);
        let text = |key: &str| find(&pairs, key).map(str::to_owned);
        let all_versions = match find(&pairs, "includeAllVersions") {
            Some("true") => AllVersions::Yes,
            Some("links") => AllVersions::Links,
            Some("false") => AllVersions::No,
            None if v2 => AllVersions::Links,
            None => AllVersions::No,
            Some(other) => return Err(format!("includeAllVersions {other} is not known")),
        };
        Ok(Lookup {
            namespace: text("namespaceName"),
            name: text("extensionName"),
            version: text("extensionVersion"),
            id: text("extensionId"),
            extension_uuid: text("extensionUuid"),
            namespace_uuid: text("namespaceUuid"),
            all_versions,
            platform: platform_of(&pairs)?,
            size: count(&pairs, "size", 100, MAX_SIZE)?,
            offset: count(&pairs, "offset", 0, usize::MAX)?,
        })
    }

    pub fn from_body(body: &Value) -> Result<Lookup, String> {
        let Some(object) = body.as_object() else {
            return Err("the query must be a JSON object".to_owned());
        };
        let pairs: Vec<(&str, String)> = object
            .iter()
            .filter_map(|(key, value)| {
                let text = match value {
                    Value::String(text) => text.clone(),
                    Value::Number(number) => number.to_string(),
                    Value::Bool(flag) => flag.to_string(),
                    _ => return None,
                };
                Some((key.as_str(), text))
            })
            .collect();
        Lookup::parse(Some(&encode(pairs)), false)
    }

    pub fn to_query(&self) -> String {
        let mut pairs = Vec::new();
        for (key, value) in [
            ("namespaceName", &self.namespace),
            ("extensionName", &self.name),
            ("extensionVersion", &self.version),
            ("extensionId", &self.id),
            ("extensionUuid", &self.extension_uuid),
            ("namespaceUuid", &self.namespace_uuid),
            ("targetPlatform", &self.platform),
        ] {
            if let Some(value) = value {
                pairs.push((key, value.clone()));
            }
        }
        pairs.push((
            "includeAllVersions",
            match self.all_versions {
                AllVersions::No => "false",
                AllVersions::Yes => "true",
                AllVersions::Links => "links",
            }
            .to_owned(),
        ));
        pairs.push(("size", self.size.to_string()));
        pairs.push(("offset", self.offset.to_string()));
        encode(pairs)
    }

    pub fn widened(&self) -> Lookup {
        Lookup {
            size: (self.offset + self.size).min(MAX_SIZE),
            offset: 0,
            ..self.clone()
        }
    }

    fn selects(&self, ext: &Ext<'_>) -> bool {
        let same = |wanted: &Option<String>, actual: &str| {
            wanted
                .as_deref()
                .is_none_or(|wanted| wanted.eq_ignore_ascii_case(actual))
        };
        let id = self.id.as_deref().is_none_or(|id| {
            names::split_id(id).is_some_and(|(namespace, name)| {
                namespace.eq_ignore_ascii_case(ext.namespace) && name.eq_ignore_ascii_case(ext.name)
            })
        });
        id && same(&self.namespace, ext.namespace)
            && same(&self.name, ext.name)
            && same(
                &self.extension_uuid,
                &names::extension_uuid(ext.namespace, ext.name),
            )
            && same(&self.namespace_uuid, &names::namespace_uuid(ext.namespace))
    }
}

pub fn lookup(urls: &Urls, exts: &[Ext<'_>], params: &Lookup) -> Value {
    let platform = params.platform.as_deref();
    let mut found: Vec<Value> = Vec::new();
    for ext in exts.iter().filter(|ext| params.selects(ext)) {
        let releases: Vec<&Release> = match (&params.version, params.all_versions) {
            (Some(version), _) => ext
                .of_version(version)
                .into_iter()
                .filter(|release| platform.is_none_or(|platform| release.platform == platform))
                .collect(),
            (None, AllVersions::Yes) => match platform {
                Some(platform) => ext
                    .versions()
                    .into_iter()
                    .filter_map(|version| ext.version(version, Some(platform)))
                    .collect(),
                None => ext.releases.clone(),
            },
            (None, _) => ext.latest(platform).into_iter().collect(),
        };
        found.extend(
            releases
                .into_iter()
                .map(|release| extension_document(urls, ext, release)),
        );
    }
    let total = found.len();
    json!({
        "offset": params.offset,
        "totalSize": total,
        "extensions": page(found, params.offset, params.size),
    })
}

pub fn versions(
    urls: &Urls,
    ext: &Ext<'_>,
    platform: Option<&str>,
    references: bool,
    offset: usize,
    size: usize,
) -> Value {
    if references {
        let all: Vec<Value> = ext
            .releases
            .iter()
            .filter(|release| {
                platform.is_none_or(|platform| {
                    release.platform == platform || release.platform == names::UNIVERSAL
                })
            })
            .map(|release| version_reference(urls, release))
            .collect();
        let total = all.len();
        return json!({"offset": offset, "totalSize": total, "versions": page(all, offset, size)});
    }
    let releases: Vec<&Release> = ext
        .versions()
        .into_iter()
        .filter_map(|version| ext.version(version, platform))
        .collect();
    let total = releases.len();
    let mut map = Map::new();
    for release in page(releases, offset, size) {
        let url = match platform {
            Some(platform) => urls.under(
                ext.namespace,
                ext.name,
                &format!("{platform}/{}", release.version),
            ),
            None => urls.under(ext.namespace, ext.name, &release.version),
        };
        map.insert(release.version.clone(), json!(url));
    }
    json!({"offset": offset, "totalSize": total, "versions": map})
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

fn pre_release(document: &Value) -> bool {
    document
        .get("preRelease")
        .and_then(Value::as_bool)
        .unwrap_or(false)
        || names::has_prerelease_tag(text(document, "version"))
}

fn merge_maps(target: &mut Value, key: &str, documents: &[&Value]) {
    let mut merged = Map::new();
    for document in documents {
        if let Some(map) = document.get(key).and_then(Value::as_object) {
            for (name, value) in map {
                merged.entry(name.clone()).or_insert_with(|| value.clone());
            }
        }
    }
    if let Some(object) = target.as_object_mut()
        && !merged.is_empty()
    {
        object.insert(key.to_owned(), Value::Object(merged));
    }
}

pub fn merge_extension(documents: Vec<Value>, alias: Option<&str>) -> Option<Value> {
    let chosen = match alias {
        Some(version) if version != "latest" && version != "pre-release" => {
            documents.first()?.clone()
        }
        _ => {
            let wanted_pre = alias == Some("pre-release");
            let released: Vec<&str> = documents
                .iter()
                .filter(|document| !pre_release(document))
                .map(|document| text(document, "version"))
                .collect();
            let releases: Vec<&Value> = documents
                .iter()
                .filter(|document| wanted_pre || released.contains(&text(document, "version")))
                .collect();
            let pool: Vec<&Value> = match releases.is_empty() {
                true => documents.iter().collect(),
                false => releases,
            };
            let mut best: Option<&Value> = None;
            for document in pool {
                if best.is_none_or(|known| {
                    names::compare_versions(text(document, "version"), text(known, "version"))
                        == Ordering::Greater
                }) {
                    best = Some(document);
                }
            }
            best?.clone()
        }
    };
    let version = text(&chosen, "version").to_owned();
    let same_version: Vec<&Value> = documents
        .iter()
        .filter(|document| text(document, "version") == version)
        .collect();
    let all: Vec<&Value> = documents.iter().collect();
    let mut merged = chosen;
    merge_maps(&mut merged, "downloads", &same_version);
    merge_maps(&mut merged, "allVersions", &all);
    Some(merged)
}

pub fn merge_namespace(documents: Vec<Value>) -> Option<Value> {
    let first = documents.first()?.clone();
    let all: Vec<&Value> = documents.iter().collect();
    let mut merged = first;
    merge_maps(&mut merged, "extensions", &all);
    Some(merged)
}

fn listed(answers: &[Value], key: &str) -> (Vec<Value>, usize) {
    let mut items = Vec::new();
    let mut total = 0usize;
    for answer in answers {
        total += answer.get("totalSize").and_then(Value::as_u64).unwrap_or(0) as usize;
        items.extend(
            answer
                .get(key)
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default(),
        );
    }
    (items, total)
}

fn unique(items: Vec<Value>, identity: impl Fn(&Value) -> String) -> (Vec<Value>, usize) {
    let mut seen = std::collections::BTreeSet::new();
    let mut duplicates = 0;
    let mut kept = Vec::new();
    for item in items {
        match seen.insert(identity(&item)) {
            true => kept.push(item),
            false => duplicates += 1,
        }
    }
    (kept, duplicates)
}

fn extension_key(item: &Value) -> String {
    names::key(text(item, "namespace"), text(item, "name"))
}

pub fn merge_search(params: &Search, answers: Vec<Value>) -> Value {
    let (items, total) = listed(&answers, "extensions");
    let (kept, duplicates) = unique(items, extension_key);
    let total = total.saturating_sub(duplicates).max(kept.len());
    json!({"offset": params.offset, "totalSize": total, "extensions": page(kept, params.offset, params.size)})
}

pub fn merge_lookup(params: &Lookup, answers: Vec<Value>) -> Value {
    let (items, total) = listed(&answers, "extensions");
    let (kept, duplicates) = unique(items, |item| {
        format!(
            "{}@{}@{}",
            extension_key(item),
            text(item, "version"),
            text(item, "targetPlatform")
        )
    });
    let total = total.saturating_sub(duplicates).max(kept.len());
    json!({"offset": params.offset, "totalSize": total, "extensions": page(kept, params.offset, params.size)})
}

pub fn merge_versions(answers: Vec<Value>, references: bool, offset: usize, size: usize) -> Value {
    match references {
        true => {
            let (items, _) = listed(&answers, "versions");
            let (mut kept, _) = unique(items, |item| {
                format!("{}@{}", text(item, "version"), text(item, "targetPlatform"))
            });
            kept.sort_by(|a, b| {
                names::compare_versions(text(b, "version"), text(a, "version")).then_with(|| {
                    names::platform_rank(text(a, "targetPlatform"))
                        .cmp(&names::platform_rank(text(b, "targetPlatform")))
                })
            });
            let total = kept.len();
            json!({"offset": offset, "totalSize": total, "versions": page(kept, offset, size)})
        }
        false => {
            let mut merged: BTreeMap<String, Value> = BTreeMap::new();
            for answer in &answers {
                for (version, url) in answer
                    .get("versions")
                    .and_then(Value::as_object)
                    .into_iter()
                    .flatten()
                {
                    merged.entry(version.clone()).or_insert_with(|| url.clone());
                }
            }
            let mut ordered: Vec<(String, Value)> = merged.into_iter().collect();
            ordered.sort_by(|(a, _), (b, _)| names::compare_versions(b, a));
            let total = ordered.len();
            let map: Map<String, Value> = page(ordered, offset, size).into_iter().collect();
            json!({"offset": offset, "totalSize": total, "versions": map})
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_parameters_parse_and_widen() {
        let search = Search::parse(Some("query=hello+world&size=5&offset=10&sortBy=downloadCount&targetPlatform=linux-x64&token=secret")).unwrap();
        assert_eq!(search.query.as_deref(), Some("hello world"));
        assert_eq!(search.platform.as_deref(), Some("linux-x64"));
        let wide = search.widened();
        assert_eq!((wide.offset, wide.size), (0, 15));
        assert!(!wide.to_query().contains("token"));
        assert_eq!(Search::parse(Some(&search.to_query())).unwrap(), search);
        assert!(Search::parse(Some("targetPlatform=mars")).is_err());
        assert!(Search::parse(Some("sortBy=name")).is_err());
        let lookup = Lookup::parse(
            Some("extensionId=acme.hello&includeAllVersions=true"),
            false,
        )
        .unwrap();
        assert_eq!(lookup.all_versions, AllVersions::Yes);
        assert_eq!(
            Lookup::parse(None, true).unwrap().all_versions,
            AllVersions::Links
        );
        assert_eq!(
            Lookup::parse(Some(&lookup.to_query()), false).unwrap(),
            lookup
        );
        let body = Lookup::from_body(&json!({"extensionId": "acme.hello", "size": 3})).unwrap();
        assert_eq!(body.size, 3);
    }

    #[test]
    fn group_documents_merge_first_member_first() {
        let own = json!({"namespace": "acme", "name": "hello", "version": "1.0.0", "member": "own", "downloads": {"universal": "own-u"}, "allVersions": {"1.0.0": "own-1"}});
        let far = json!({"namespace": "acme", "name": "hello", "version": "1.0.0", "member": "far", "downloads": {"universal": "far-u", "linux-x64": "far-l"}, "allVersions": {"1.0.0": "far-1", "0.9.0": "far-09"}});
        let newer = json!({"namespace": "acme", "name": "hello", "version": "2.0.0", "preRelease": true, "member": "pre"});
        let latest = merge_extension(vec![own.clone(), far.clone(), newer.clone()], None).unwrap();
        assert_eq!(latest["member"], "own");
        assert_eq!(latest["downloads"]["universal"], "own-u");
        assert_eq!(latest["downloads"]["linux-x64"], "far-l");
        assert_eq!(latest["allVersions"]["0.9.0"], "far-09");
        assert_eq!(latest["allVersions"]["1.0.0"], "own-1");
        let pre = merge_extension(vec![own.clone(), newer], Some("pre-release")).unwrap();
        assert_eq!(pre["version"], "2.0.0");
        let platform_pre = json!({"namespace": "acme", "name": "hello", "version": "1.0.0", "preRelease": true, "member": "own"});
        let platform = merge_extension(vec![platform_pre, far.clone()], None).unwrap();
        assert_eq!(platform["member"], "own");
        assert!(merge_extension(Vec::new(), None).is_none());
        let search = Search::parse(Some("size=1&offset=1")).unwrap();
        let merged = merge_search(
            &search,
            vec![
                json!({"totalSize": 1, "extensions": [own.clone()]}),
                json!({"totalSize": 2, "extensions": [far, json!({"namespace": "acme", "name": "other"})]}),
            ],
        );
        assert_eq!(merged["totalSize"], 2);
        assert_eq!(merged["extensions"][0]["name"], "other");
        let versions = merge_versions(
            vec![
                json!({"versions": {"1.0.0": "own"}}),
                json!({"versions": {"1.0.0": "far", "1.10.0": "far", "1.9.0": "far"}}),
            ],
            false,
            0,
            2,
        );
        assert_eq!(versions["totalSize"], 3);
        assert_eq!(versions["versions"]["1.10.0"], "far");
        assert!(versions["versions"].get("1.0.0").is_none());
    }
}
