use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

use crate::path::Name;

const ABBREVIATED: &[&str] = &[
    "name",
    "version",
    "deprecated",
    "dependencies",
    "optionalDependencies",
    "devDependencies",
    "bundleDependencies",
    "bundledDependencies",
    "peerDependencies",
    "peerDependenciesMeta",
    "acceptDependencies",
    "bin",
    "directories",
    "dist",
    "engines",
    "funding",
    "license",
    "_hasShrinkwrap",
    "hasInstallScript",
    "cpu",
    "os",
    "libc",
];

const INSTALL_SCRIPTS: &[&str] = &["preinstall", "install", "postinstall"];

const RESERVED: &[&str] = &[
    "_id",
    "_rev",
    "_attachments",
    "_revisions",
    "access",
    "name",
    "versions",
    "dist-tags",
    "time",
];

pub fn versions(doc: &Value) -> Option<&Map<String, Value>> {
    doc.get("versions").and_then(Value::as_object)
}

pub fn dist_tags(doc: &Value) -> Map<String, Value> {
    doc.get("dist-tags")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

pub fn tarball_file(manifest: &Value) -> Option<String> {
    let url = manifest.pointer("/dist/tarball")?.as_str()?;
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let file = path.rsplit('/').next()?;
    (!file.is_empty()).then(|| file.to_owned())
}

pub fn rewrite_tarballs(doc: &mut Value, repository_url: &str, name: &Name) {
    let Some(versions) = doc.get_mut("versions").and_then(Value::as_object_mut) else {
        return;
    };
    for (version, manifest) in versions.iter_mut() {
        let file = tarball_file(manifest).unwrap_or_else(|| name.tarball_file(version));
        if let Some(dist) = manifest.get_mut("dist").and_then(Value::as_object_mut) {
            dist.insert(
                "tarball".to_owned(),
                Value::String(format!("{repository_url}{}", name.tarball_path(&file))),
            );
        }
    }
}

pub fn abbreviate(doc: &Value) -> Value {
    let mut versions = Map::new();
    for (version, manifest) in self::versions(doc).into_iter().flatten() {
        let mut picked = Map::new();
        for field in ABBREVIATED {
            if let Some(value) = manifest.get(*field) {
                picked.insert((*field).to_owned(), value.clone());
            }
        }
        if !picked.contains_key("hasInstallScript") && has_install_script(manifest) {
            picked.insert("hasInstallScript".to_owned(), Value::Bool(true));
        }
        versions.insert(version.clone(), Value::Object(picked));
    }
    let mut abbreviated = Map::new();
    abbreviated.insert(
        "name".to_owned(),
        doc.get("name").cloned().unwrap_or_default(),
    );
    if let Some(modified) = doc.pointer("/time/modified") {
        abbreviated.insert("modified".to_owned(), modified.clone());
    }
    abbreviated.insert("dist-tags".to_owned(), Value::Object(dist_tags(doc)));
    abbreviated.insert("versions".to_owned(), Value::Object(versions));
    Value::Object(abbreviated)
}

fn has_install_script(manifest: &Value) -> bool {
    manifest
        .get("scripts")
        .and_then(Value::as_object)
        .is_some_and(|scripts| {
            INSTALL_SCRIPTS
                .iter()
                .any(|name| scripts.contains_key(*name))
        })
}

pub fn resolve<'a>(doc: &'a Value, wanted: &str) -> Option<&'a Value> {
    let versions = versions(doc)?;
    if let Some(manifest) = versions.get(wanted) {
        return Some(manifest);
    }
    let tagged = doc.get("dist-tags")?.get(wanted)?.as_str()?;
    versions.get(tagged)
}

fn parsed(version: &str) -> Option<semver::Version> {
    semver::Version::parse(version.trim_start_matches(['v', '='])).ok()
}

pub fn valid_version(version: &str) -> bool {
    semver::Version::parse(version).is_ok()
}

pub fn valid_tag(tag: &str) -> bool {
    !tag.is_empty()
        && tag.len() <= 128
        && parsed(tag).is_none()
        && semver::VersionReq::parse(tag).is_err()
        && tag
            .chars()
            .all(|c| c.is_ascii_graphic() && !matches!(c, '/' | '%' | '\\'))
}

pub fn highest<'a>(versions: impl Iterator<Item = &'a String>) -> Option<String> {
    let mut known: Vec<(semver::Version, &String)> = Vec::new();
    let mut unknown: Vec<&String> = Vec::new();
    for version in versions {
        match parsed(version) {
            Some(parsed) => known.push((parsed, version)),
            None => unknown.push(version),
        }
    }
    let stable = known
        .iter()
        .filter(|(parsed, _)| parsed.pre.is_empty())
        .max_by(|a, b| a.0.cmp(&b.0));
    stable
        .or_else(|| known.iter().max_by(|a, b| a.0.cmp(&b.0)))
        .map(|(_, version)| (*version).clone())
        .or_else(|| unknown.into_iter().max().cloned())
}

pub fn tidy_tags(doc: &mut Value) {
    let names: BTreeSet<String> = versions(doc)
        .map(|versions| versions.keys().cloned().collect())
        .unwrap_or_default();
    let mut tags = dist_tags(doc);
    tags.retain(|_, version| version.as_str().is_some_and(|v| names.contains(v)));
    if !tags.contains_key("latest")
        && let Some(latest) = highest(names.iter())
    {
        tags.insert("latest".to_owned(), Value::String(latest));
    }
    doc["dist-tags"] = Value::Object(tags);
}

pub fn next_rev(doc: Option<&Value>) -> String {
    let current = doc
        .and_then(|doc| doc.get("_rev"))
        .and_then(Value::as_str)
        .and_then(|rev| rev.split('-').next())
        .and_then(|rev| rev.parse::<u64>().ok())
        .unwrap_or(0);
    (current + 1).to_string()
}

pub fn fresh(name: &Name) -> Value {
    json!({
        "_id": name.to_string(),
        "name": name.to_string(),
        "dist-tags": {},
        "versions": {},
        "time": {},
    })
}

fn time_map(doc: &mut Value) -> &mut Map<String, Value> {
    if !doc.get("time").is_some_and(Value::is_object) {
        doc["time"] = json!({});
    }
    doc["time"]
        .as_object_mut()
        .expect("the time map was just made an object")
}

fn versions_mut(doc: &mut Value) -> &mut Map<String, Value> {
    if !doc.get("versions").is_some_and(Value::is_object) {
        doc["versions"] = json!({});
    }
    doc["versions"]
        .as_object_mut()
        .expect("the versions map was just made an object")
}

fn copy_top_level(target: &mut Value, incoming: &Value) {
    let Some(fields) = incoming.as_object() else {
        return;
    };
    for (key, value) in fields {
        if !RESERVED.contains(&key.as_str()) {
            target[key.as_str()] = value.clone();
        }
    }
}

pub fn publish(
    stored: Option<Value>,
    name: &Name,
    incoming: &Value,
    added: &[(String, Value)],
    now: &str,
) -> Value {
    let rev = next_rev(stored.as_ref());
    let mut doc = stored.unwrap_or_else(|| fresh(name));
    copy_top_level(&mut doc, incoming);
    doc["_id"] = Value::String(name.to_string());
    doc["name"] = Value::String(name.to_string());
    for (version, manifest) in added {
        versions_mut(&mut doc).insert(version.clone(), manifest.clone());
        time_map(&mut doc).insert(version.clone(), Value::String(now.to_owned()));
    }
    let mut tags = dist_tags(&doc);
    for (tag, version) in dist_tags(incoming) {
        if version.is_string() && valid_tag(&tag) {
            tags.insert(tag, version);
        }
    }
    doc["dist-tags"] = Value::Object(tags);
    tidy_tags(&mut doc);
    stamp(&mut doc, now, rev);
    doc
}

fn stamp(doc: &mut Value, now: &str, rev: String) {
    let time = time_map(doc);
    if !time.contains_key("created") {
        time.insert("created".to_owned(), Value::String(now.to_owned()));
    }
    time.insert("modified".to_owned(), Value::String(now.to_owned()));
    doc["_rev"] = Value::String(rev);
}

#[derive(Debug, PartialEq)]
pub struct Update {
    pub doc: Value,
    pub removed: Vec<(String, Value)>,
}

pub fn update(stored: Value, incoming: &Value, now: &str) -> Result<Update, String> {
    let rev = next_rev(Some(&stored));
    let mut doc = stored;
    let wanted = versions(incoming).cloned().unwrap_or_default();
    if let Some(unknown) = wanted
        .keys()
        .find(|version| !versions(&doc).is_some_and(|known| known.contains_key(*version)))
    {
        return Err(format!(
            "version {unknown} has no tarball; publish it with its tarball"
        ));
    }
    let mut removed = Vec::new();
    let current = std::mem::take(versions_mut(&mut doc));
    let mut kept = Map::new();
    for (version, manifest) in current {
        match wanted.get(&version) {
            Some(changed) => {
                let mut merged = changed.clone();
                match (merged.as_object_mut(), manifest.get("dist")) {
                    (Some(fields), Some(dist)) => {
                        fields.insert("dist".to_owned(), dist.clone());
                        if let Some(name) = manifest.get("name") {
                            fields.insert("name".to_owned(), name.clone());
                        }
                        fields.insert("version".to_owned(), Value::String(version.clone()));
                    }
                    _ => merged = manifest.clone(),
                }
                kept.insert(version, merged);
            }
            None => removed.push((version, manifest)),
        }
    }
    doc["versions"] = Value::Object(kept);
    for (version, _) in &removed {
        time_map(&mut doc).remove(version);
    }
    copy_top_level(&mut doc, incoming);
    if incoming.get("dist-tags").is_some_and(Value::is_object) {
        doc["dist-tags"] = Value::Object(dist_tags(incoming));
    }
    tidy_tags(&mut doc);
    stamp(&mut doc, now, rev);
    Ok(Update { doc, removed })
}

pub fn set_tag(mut doc: Value, tag: &str, version: Option<&str>, now: &str) -> Value {
    let rev = next_rev(Some(&doc));
    let mut tags = dist_tags(&doc);
    match version {
        Some(version) => {
            tags.insert(tag.to_owned(), Value::String(version.to_owned()));
        }
        None => {
            tags.remove(tag);
        }
    }
    doc["dist-tags"] = Value::Object(tags);
    tidy_tags(&mut doc);
    stamp(&mut doc, now, rev);
    doc
}

pub fn replace_tags(mut doc: Value, tags: Map<String, Value>, merge: bool, now: &str) -> Value {
    let rev = next_rev(Some(&doc));
    let mut current = if merge { dist_tags(&doc) } else { Map::new() };
    current.extend(tags);
    doc["dist-tags"] = Value::Object(current);
    tidy_tags(&mut doc);
    stamp(&mut doc, now, rev);
    doc
}

pub fn remove_versions(mut doc: Value, gone: &BTreeSet<String>) -> Value {
    if gone.is_empty() {
        return doc;
    }
    versions_mut(&mut doc).retain(|version, _| !gone.contains(version));
    let time = time_map(&mut doc);
    time.retain(|key, _| !gone.contains(key));
    tidy_tags(&mut doc);
    doc
}

pub fn merge(docs: Vec<Value>) -> Option<Value> {
    let mut docs = docs.into_iter();
    let mut merged = docs.next()?;
    if let Some(fields) = merged.as_object_mut() {
        fields.remove("_rev");
    }
    let mut tags = dist_tags(&merged);
    for doc in docs {
        for (version, manifest) in versions(&doc).into_iter().flatten() {
            versions_mut(&mut merged)
                .entry(version.clone())
                .or_insert_with(|| manifest.clone());
        }
        for (tag, version) in dist_tags(&doc) {
            tags.entry(tag).or_insert(version);
        }
        let Some(times) = doc.get("time").and_then(Value::as_object) else {
            continue;
        };
        let target = time_map(&mut merged);
        for (key, value) in times {
            let keep_earlier = key == "created";
            let keep_later = key == "modified";
            match target.get(key) {
                None => {
                    target.insert(key.clone(), value.clone());
                }
                Some(existing) => {
                    let (old, new) = (existing.as_str(), value.as_str());
                    let replace = match (old, new) {
                        (Some(old), Some(new)) if keep_earlier => new < old,
                        (Some(old), Some(new)) if keep_later => new > old,
                        _ => false,
                    };
                    if replace {
                        target.insert(key.clone(), value.clone());
                    }
                }
            }
        }
    }
    merged["dist-tags"] = Value::Object(tags);
    Some(merged)
}

pub fn search_attributes(doc: &Value) -> Value {
    let latest = doc
        .pointer("/dist-tags/latest")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let manifest = versions(doc).and_then(|versions| versions.get(latest));
    let pick = |field: &str| {
        manifest
            .and_then(|manifest| manifest.get(field))
            .or_else(|| doc.get(field))
            .cloned()
            .unwrap_or(Value::Null)
    };
    let keywords = match pick("keywords") {
        Value::Array(words) => Value::Array(words.into_iter().filter(Value::is_string).collect()),
        Value::String(words) => Value::Array(
            words
                .split([',', ' '])
                .filter(|word| !word.is_empty())
                .map(|word| Value::String(word.to_owned()))
                .collect(),
        ),
        _ => Value::Array(Vec::new()),
    };
    let description = match pick("description") {
        Value::String(text) => Value::String(text),
        _ => Value::Null,
    };
    json!({
        "npm": {
            "name": doc.get("name").cloned().unwrap_or_default(),
            "version": latest,
            "description": description,
            "keywords": keywords,
            "modified": doc.pointer("/time/modified").cloned().unwrap_or(Value::Null),
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name(full: &str) -> Name {
        Name::parse(full).unwrap()
    }

    fn manifest(name: &str, version: &str) -> Value {
        json!({
            "name": name,
            "version": version,
            "description": "a tool",
            "dependencies": {"left-pad": "^1.0.0"},
            "scripts": {"test": "jest", "postinstall": "node setup.js"},
            "readme": "# tool",
            "dist": {
                "shasum": "x",
                "integrity": "sha512-y",
                "tarball": format!("http://elsewhere.example/{name}/-/@team/tool-{version}.tgz")
            }
        })
    }

    #[test]
    fn tarball_urls_point_at_the_serving_repository() {
        let mut doc = publish(
            None,
            &name("@team/tool"),
            &json!({"dist-tags": {"latest": "1.0.0"}}),
            &[("1.0.0".to_owned(), manifest("@team/tool", "1.0.0"))],
            "2026-10-12T00:00:00.000Z",
        );
        rewrite_tarballs(
            &mut doc,
            "http://astrum.example/repository/npm",
            &name("@team/tool"),
        );
        assert_eq!(
            doc["versions"]["1.0.0"]["dist"]["tarball"],
            "http://astrum.example/repository/npm/@team/tool/-/tool-1.0.0.tgz"
        );
    }

    #[test]
    fn abbreviated_documents_keep_what_installs_need() {
        let doc = publish(
            None,
            &name("tool"),
            &json!({"dist-tags": {"latest": "1.0.0"}, "readme": "long text"}),
            &[("1.0.0".to_owned(), manifest("tool", "1.0.0"))],
            "2026-10-12T00:00:00.000Z",
        );
        let short = abbreviate(&doc);
        assert_eq!(short["name"], "tool");
        assert_eq!(short["modified"], "2026-10-12T00:00:00.000Z");
        assert_eq!(short["dist-tags"]["latest"], "1.0.0");
        let version = &short["versions"]["1.0.0"];
        assert_eq!(version["dependencies"]["left-pad"], "^1.0.0");
        assert_eq!(version["hasInstallScript"], true);
        assert!(version.get("readme").is_none());
        assert!(version.get("scripts").is_none());
        assert!(short.get("readme").is_none());
    }

    #[test]
    fn publishing_adds_versions_tags_and_times() {
        let tool = name("tool");
        let first = publish(
            None,
            &tool,
            &json!({"description": "first", "dist-tags": {"latest": "1.0.0"}}),
            &[("1.0.0".to_owned(), manifest("tool", "1.0.0"))],
            "2026-10-12T00:00:00.000Z",
        );
        assert_eq!(first["_rev"], "1");
        assert_eq!(first["time"]["created"], "2026-10-12T00:00:00.000Z");
        let second = publish(
            Some(first),
            &tool,
            &json!({"description": "second", "dist-tags": {"beta": "2.0.0-beta.1"}, "_attachments": {}}),
            &[("2.0.0-beta.1".to_owned(), manifest("tool", "2.0.0-beta.1"))],
            "2026-10-13T00:00:00.000Z",
        );
        assert_eq!(second["_rev"], "2");
        assert_eq!(second["description"], "second");
        assert!(second.get("_attachments").is_none());
        assert_eq!(
            second["dist-tags"],
            json!({"latest": "1.0.0", "beta": "2.0.0-beta.1"})
        );
        assert_eq!(second["time"]["created"], "2026-10-12T00:00:00.000Z");
        assert_eq!(second["time"]["modified"], "2026-10-13T00:00:00.000Z");
        assert_eq!(second["time"]["2.0.0-beta.1"], "2026-10-13T00:00:00.000Z");
        assert_eq!(versions(&second).unwrap().len(), 2);
    }

    #[test]
    fn updates_deprecate_and_unpublish_but_keep_dist() {
        let tool = name("tool");
        let mut doc = None;
        for version in ["1.0.0", "1.1.0", "2.0.0"] {
            doc = Some(publish(
                doc,
                &tool,
                &json!({"dist-tags": {"latest": version}}),
                &[(version.to_owned(), manifest("tool", version))],
                "2026-10-12T00:00:00.000Z",
            ));
        }
        let doc = doc.unwrap();
        let mut incoming = doc.clone();
        incoming["versions"]["1.0.0"]["deprecated"] = json!("use 2");
        incoming["versions"]["1.0.0"]["dist"]["tarball"] = json!("http://evil.example/x.tgz");
        incoming["versions"]
            .as_object_mut()
            .unwrap()
            .remove("2.0.0");
        incoming["dist-tags"] = json!({"latest": "2.0.0", "old": "1.0.0"});
        let updated = update(doc.clone(), &incoming, "2026-10-14T00:00:00.000Z").unwrap();
        assert_eq!(updated.removed.len(), 1);
        assert_eq!(updated.removed[0].0, "2.0.0");
        let result = &updated.doc;
        assert_eq!(result["versions"]["1.0.0"]["deprecated"], "use 2");
        assert_eq!(
            result["versions"]["1.0.0"]["dist"],
            doc["versions"]["1.0.0"]["dist"]
        );
        assert_eq!(
            result["dist-tags"],
            json!({"latest": "1.1.0", "old": "1.0.0"})
        );
        assert!(result["time"].get("2.0.0").is_none());
        assert_eq!(result["_rev"], "4");

        let mut sneaky = doc.clone();
        sneaky["versions"]["9.9.9"] = manifest("tool", "9.9.9");
        assert!(update(doc, &sneaky, "2026-10-14T00:00:00.000Z").is_err());
    }

    #[test]
    fn groups_merge_versions_tags_and_times_in_member_order() {
        let first = json!({
            "name": "tool", "_rev": "7", "description": "from the first",
            "dist-tags": {"latest": "1.0.0"},
            "versions": {"1.0.0": {"version": "1.0.0", "from": "first"}},
            "time": {"created": "2020-01-01T00:00:00.000Z", "modified": "2021-01-01T00:00:00.000Z", "1.0.0": "2020-01-01T00:00:00.000Z"}
        });
        let second = json!({
            "name": "tool", "description": "from the second",
            "dist-tags": {"latest": "2.0.0", "next": "3.0.0-rc.1"},
            "versions": {
                "1.0.0": {"version": "1.0.0", "from": "second"},
                "2.0.0": {"version": "2.0.0", "from": "second"},
                "3.0.0-rc.1": {"version": "3.0.0-rc.1", "from": "second"}
            },
            "time": {"created": "2019-01-01T00:00:00.000Z", "modified": "2022-01-01T00:00:00.000Z", "2.0.0": "2022-01-01T00:00:00.000Z"}
        });
        let merged = merge(vec![first, second]).unwrap();
        assert_eq!(merged["description"], "from the first");
        assert!(merged.get("_rev").is_none());
        assert_eq!(merged["versions"]["1.0.0"]["from"], "first");
        assert_eq!(merged["versions"]["2.0.0"]["from"], "second");
        assert_eq!(
            merged["dist-tags"],
            json!({"latest": "1.0.0", "next": "3.0.0-rc.1"})
        );
        assert_eq!(merged["time"]["created"], "2019-01-01T00:00:00.000Z");
        assert_eq!(merged["time"]["modified"], "2022-01-01T00:00:00.000Z");
        assert_eq!(merged["time"]["1.0.0"], "2020-01-01T00:00:00.000Z");
        assert_eq!(merged["time"]["2.0.0"], "2022-01-01T00:00:00.000Z");
        assert_eq!(merge(Vec::new()), None);
    }

    #[test]
    fn latest_prefers_the_highest_stable_version() {
        let versions = ["1.0.0", "1.10.0", "1.9.0", "2.0.0-rc.1"].map(str::to_owned);
        assert_eq!(highest(versions.iter()).as_deref(), Some("1.10.0"));
        let only_pre = ["2.0.0-rc.1", "2.0.0-rc.2"].map(str::to_owned);
        assert_eq!(highest(only_pre.iter()).as_deref(), Some("2.0.0-rc.2"));
        assert_eq!(highest(std::iter::empty()), None);
    }

    #[test]
    fn tags_and_versions_are_checked() {
        assert!(valid_tag("beta"));
        assert!(valid_tag("next-2"));
        assert!(!valid_tag("1.0.0"));
        assert!(!valid_tag("^1"));
        assert!(!valid_tag(""));
        assert!(!valid_tag("a/b"));
        assert!(valid_version("1.0.0-beta.1+build.5"));
        assert!(!valid_version("1.0"));
        assert!(!valid_version("../1.0.0"));
    }

    #[test]
    fn versions_resolve_by_number_or_tag() {
        let doc =
            json!({"dist-tags": {"latest": "1.0.0"}, "versions": {"1.0.0": {"version": "1.0.0"}}});
        assert_eq!(resolve(&doc, "1.0.0").unwrap()["version"], "1.0.0");
        assert_eq!(resolve(&doc, "latest").unwrap()["version"], "1.0.0");
        assert!(resolve(&doc, "2.0.0").is_none());
    }

    #[test]
    fn revisions_count_up() {
        assert_eq!(next_rev(None), "1");
        assert_eq!(next_rev(Some(&json!({"_rev": "41"}))), "42");
        assert_eq!(next_rev(Some(&json!({"_rev": "3-abcdef"}))), "4");
        assert_eq!(
            next_rev(Some(&json!({"_rev": "_we_dont_use_revs_any_more"}))),
            "1"
        );
    }

    #[test]
    fn search_attributes_describe_the_latest_version() {
        let doc = json!({
            "name": "tool",
            "dist-tags": {"latest": "1.0.0"},
            "versions": {"1.0.0": {"description": "does things", "keywords": ["cli", 3, "tool"]}},
            "time": {"modified": "2026-10-12T00:00:00.000Z"}
        });
        assert_eq!(
            search_attributes(&doc),
            json!({"npm": {"name": "tool", "version": "1.0.0", "description": "does things", "keywords": ["cli", "tool"], "modified": "2026-10-12T00:00:00.000Z"}})
        );
    }
}
