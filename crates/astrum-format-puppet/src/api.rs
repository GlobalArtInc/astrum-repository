use std::cmp::Ordering;
use std::collections::BTreeSet;

use astrum_content::{ContentError, ContentRequest};
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::Response;
use chrono::{DateTime, FixedOffset, Utc};
use serde_json::{Value, json};

use crate::names::{self, Module};

pub const JSON_TYPE: &str = "application/json";
pub const DEFAULT_LIMIT: usize = 20;
pub const MAX_LIMIT: usize = 100;
const TIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S %z";

pub fn time(at: DateTime<Utc>) -> String {
    at.format(TIME_FORMAT).to_string()
}

pub fn parse_time(text: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_str(text, TIME_FORMAT)
        .or_else(|_| DateTime::parse_from_rfc3339(text))
        .ok()
}

pub fn json(status: StatusCode, request: Option<&ContentRequest>, value: &Value) -> Response {
    let body = value.to_string();
    let head = request.is_some_and(|request| request.method == Method::HEAD);
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, JSON_TYPE)
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::CACHE_CONTROL, "no-cache")
        .body(if head {
            Body::empty()
        } else {
            Body::from(body)
        })
        .expect("a JSON response is valid")
}

pub fn error(error: ContentError) -> Response {
    let status = error.status();
    let detail = match &error {
        ContentError::Internal(inner) => {
            tracing::error!(
                error = format!("{inner:#}"),
                "a Puppet Forge request failed"
            );
            "the server failed; see its log".to_owned()
        }
        ContentError::Unauthorized => {
            "This endpoint requires a valid Authorization header".to_owned()
        }
        other => other.to_string(),
    };
    let message = format!(
        "{} {}",
        status.as_u16(),
        status.canonical_reason().unwrap_or_default()
    );
    let mut response = json(
        status,
        None,
        &json!({"message": message, "errors": [detail]}),
    );
    let headers = response.headers_mut();
    match error {
        ContentError::Unauthorized => {
            headers.insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(astrum_content::CHALLENGE),
            );
        }
        ContentError::MethodNotAllowed { allow, .. } => {
            headers.insert(header::ALLOW, HeaderValue::from_static(allow));
        }
        _ => {}
    }
    response
}

#[derive(Clone, Debug, Default)]
pub struct Params {
    pairs: Vec<(String, String, String)>,
    pub limit: usize,
    pub offset: usize,
}

impl Params {
    pub fn of(query: Option<&str>) -> Self {
        let mut params = Params {
            pairs: Vec::new(),
            limit: DEFAULT_LIMIT,
            offset: 0,
        };
        for raw in query.unwrap_or_default().split('&') {
            if raw.is_empty() {
                continue;
            }
            let Some((key, value)) = url::form_urlencoded::parse(raw.as_bytes()).next() else {
                continue;
            };
            match (key.as_ref(), value.trim().parse::<usize>()) {
                ("limit", Ok(limit)) if limit > 0 => params.limit = limit.min(MAX_LIMIT),
                ("offset", Ok(offset)) => params.offset = offset,
                ("limit" | "offset", _) => {}
                _ => params
                    .pairs
                    .push((key.into_owned(), value.into_owned(), raw.to_owned())),
            }
        }
        params
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs
            .iter()
            .find(|(name, value, _)| name == key && !value.trim().is_empty())
            .map(|(_, value, _)| value.trim())
    }

    pub fn flag(&self, key: &str) -> bool {
        self.get(key)
            .is_some_and(|value| !matches!(value, "false" | "0" | "no"))
    }

    pub fn list(&self, key: &str) -> Vec<String> {
        self.pairs
            .iter()
            .filter(|(name, _, _)| name == key)
            .flat_map(|(_, value, _)| {
                value
                    .split(|c: char| c.is_whitespace() || c == ',')
                    .filter(|part| !part.is_empty())
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    pub fn module(&self) -> Result<Option<Module>, ContentError> {
        match self.get("module") {
            None => Ok(None),
            Some(slug) => Module::parse(slug).map(Some).ok_or_else(|| {
                ContentError::BadRequest(format!("'{slug}' is not a valid module slug"))
            }),
        }
    }

    pub fn query_without_paging(&self) -> String {
        self.pairs
            .iter()
            .map(|(_, _, raw)| raw.as_str())
            .collect::<Vec<_>>()
            .join("&")
    }

    pub fn link(&self, base: &str, offset: usize) -> String {
        let mut query = self.query_without_paging();
        if !query.is_empty() {
            query.push('&');
        }
        format!("{base}?{query}limit={}&offset={offset}", self.limit)
    }

    pub fn page(&self, base: &str, items: Vec<Value>, total: usize) -> Value {
        let next = self.offset + self.limit;
        let results: Vec<Value> = items
            .into_iter()
            .skip(self.offset)
            .take(self.limit)
            .map(|item| self.exclude(item))
            .collect();
        json!({
            "pagination": {
                "limit": self.limit,
                "offset": self.offset,
                "first": self.link(base, 0),
                "previous": match self.offset {
                    0 => Value::Null,
                    offset => json!(self.link(base, offset.saturating_sub(self.limit))),
                },
                "current": self.link(base, self.offset),
                "next": if next < total { json!(self.link(base, next)) } else { Value::Null },
                "total": total,
            },
            "results": results,
        })
    }

    pub fn exclude(&self, mut value: Value) -> Value {
        let excluded = self.list("exclude_fields");
        if let Some(object) = value.as_object_mut() {
            for field in excluded {
                object.remove(&field);
            }
        }
        value
    }
}

pub fn relative(link: &str) -> String {
    if link.starts_with("/v3/") {
        return link.to_owned();
    }
    match link.find("/v3/") {
        Some(at) => link[at..].to_owned(),
        None => link.to_owned(),
    }
}

fn text<'a>(value: &'a Value, key: &str) -> &'a str {
    value.get(key).and_then(Value::as_str).unwrap_or_default()
}

pub fn version_of(value: &Value) -> &str {
    text(value, "version")
}

fn downloads(value: &Value) -> i64 {
    value.get("downloads").and_then(Value::as_i64).unwrap_or(0)
}

fn created(value: &Value) -> Option<DateTime<FixedOffset>> {
    parse_time(text(value, "created_at"))
}

pub fn owner_document(owner: &str) -> Value {
    json!({
        "uri": format!("/v3/users/{owner}"),
        "slug": owner,
        "username": owner,
        "gravatar_id": null,
    })
}

pub fn module_summary(module: &Module) -> Value {
    json!({
        "uri": module.uri(),
        "slug": module.slug(),
        "name": module.name,
        "deprecated_at": null,
        "owner": owner_document(&module.owner),
    })
}

pub fn release_slug_of(value: &Value) -> Option<(Module, String)> {
    names::parse_release(text(value, "slug")).or_else(|| {
        let module = value
            .pointer("/module/slug")
            .and_then(Value::as_str)
            .and_then(Module::parse)
            .or_else(|| {
                value
                    .pointer("/metadata/name")
                    .and_then(Value::as_str)
                    .and_then(Module::parse)
            })?;
        let version = version_of(value);
        names::valid_version(version).then(|| (module, version.to_owned()))
    })
}

pub fn rewrite_release(mut document: Value, module: &Module, version: &str) -> Value {
    if let Some(object) = document.as_object_mut() {
        object.insert("uri".to_owned(), json!(module.release_uri(version)));
        object.insert("slug".to_owned(), json!(module.release_slug(version)));
        object.insert("version".to_owned(), json!(version));
        object.insert("file_uri".to_owned(), json!(module.file_uri(version)));
        if let Some(Value::Object(summary)) = object.get_mut("module") {
            summary.insert("uri".to_owned(), json!(module.uri()));
            summary.insert("slug".to_owned(), json!(module.slug()));
            if let Some(Value::Object(owner)) = summary.get_mut("owner")
                && let Some(Value::String(uri)) = owner.get_mut("uri")
            {
                *uri = relative(uri);
            }
        }
        if let Some(Value::Array(plans)) = object.get_mut("plans") {
            for plan in plans {
                if let Some(Value::String(uri)) = plan.get_mut("uri") {
                    *uri = relative(uri);
                }
            }
        }
    }
    document
}

pub fn rewrite_remote_release(document: Value) -> Option<Value> {
    let (module, version) = release_slug_of(&document)?;
    Some(rewrite_release(document, &module, &version))
}

pub fn abbreviated(release: &Value) -> Value {
    let pick = |key: &str| release.get(key).cloned().unwrap_or(Value::Null);
    json!({
        "uri": pick("uri"),
        "slug": pick("slug"),
        "version": pick("version"),
        "supported": release.get("supported").cloned().unwrap_or(json!(false)),
        "created_at": pick("created_at"),
        "deleted_at": pick("deleted_at"),
        "file_uri": pick("file_uri"),
        "file_size": pick("file_size"),
    })
}

pub fn rewrite_module(mut document: Value, module: &Module) -> Value {
    if let Some(object) = document.as_object_mut() {
        object.insert("uri".to_owned(), json!(module.uri()));
        object.insert("slug".to_owned(), json!(module.slug()));
        if let Some(Value::Object(owner)) = object.get_mut("owner")
            && let Some(Value::String(uri)) = owner.get_mut("uri")
        {
            *uri = relative(uri);
        }
        let current = object.remove("current_release");
        if let Some(current) = current {
            let current = match current {
                Value::Object(_) => match release_slug_of(&current) {
                    Some((owner, version)) if owner == *module => {
                        rewrite_release(current, module, &version)
                    }
                    _ => Value::Null,
                },
                other => other,
            };
            object.insert("current_release".to_owned(), current);
        }
        if let Some(Value::Array(releases)) = object.get_mut("releases") {
            releases.retain(|release| names::valid_version(version_of(release)));
            for release in releases.iter_mut() {
                let version = version_of(release).to_owned();
                *release = rewrite_release(release.take(), module, &version);
            }
            releases.sort_by(|a, b| names::newest_first(version_of(a), version_of(b)));
        }
    }
    document
}

pub fn rewrite_remote_module(document: Value) -> Option<Value> {
    let module = Module::parse(text(&document, "slug"))?;
    Some(rewrite_module(document, &module))
}

pub fn module_document(module: &Module, releases: &[Value]) -> Option<Value> {
    let highest = names::highest(releases.iter().map(version_of))?;
    let current = releases
        .iter()
        .find(|release| version_of(release) == highest)?
        .clone();
    let mut sorted: Vec<&Value> = releases.iter().collect();
    sorted.sort_by(|a, b| names::newest_first(version_of(a), version_of(b)));
    let first = releases
        .iter()
        .filter_map(|release| Some((created(release)?, text(release, "created_at"))))
        .min_by_key(|(at, _)| *at)
        .map(|(_, text)| text.to_owned());
    let last = releases
        .iter()
        .filter_map(|release| {
            Some((
                parse_time(text(release, "updated_at"))?,
                text(release, "updated_at"),
            ))
        })
        .max_by_key(|(at, _)| *at)
        .map(|(_, text)| text.to_owned());
    let metadata = current.get("metadata").cloned().unwrap_or(Value::Null);
    let link = |key: &str| {
        metadata
            .get(key)
            .filter(|value| value.is_string())
            .cloned()
            .unwrap_or(Value::Null)
    };
    Some(json!({
        "uri": module.uri(),
        "slug": module.slug(),
        "name": module.name,
        "downloads": releases.iter().map(downloads).sum::<i64>(),
        "created_at": first,
        "updated_at": last,
        "deprecated_at": null,
        "deprecated_for": null,
        "superseded_by": null,
        "supported": false,
        "endorsement": null,
        "module_group": "base",
        "premium": false,
        "owner": owner_document(&module.owner),
        "current_release": current,
        "releases": sorted.into_iter().map(abbreviated).collect::<Vec<_>>(),
        "feedback_score": null,
        "homepage_url": link("project_page"),
        "issues_url": link("issues_url"),
    }))
}

pub fn merge_modules(documents: Vec<Value>) -> Option<Value> {
    let mut documents = documents.into_iter();
    let mut merged = documents.next()?;
    let mut releases: Vec<Value> = merged
        .get("releases")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut currents: Vec<Value> = merged
        .get("current_release")
        .filter(|current| current.is_object())
        .cloned()
        .into_iter()
        .collect();
    let mut total = downloads(&merged);
    for other in documents {
        for release in other
            .get("releases")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if !releases
                .iter()
                .any(|known| version_of(known) == version_of(release))
            {
                releases.push(release.clone());
            }
        }
        if let Some(current) = other
            .get("current_release")
            .filter(|current| current.is_object())
        {
            currents.push(current.clone());
        }
        total += downloads(&other);
    }
    releases.sort_by(|a, b| names::newest_first(version_of(a), version_of(b)));
    let current = names::highest(currents.iter().map(version_of))
        .and_then(|highest| {
            currents
                .iter()
                .find(|current| version_of(current) == highest)
        })
        .cloned()
        .unwrap_or(Value::Null);
    if let Some(object) = merged.as_object_mut() {
        object.insert("releases".to_owned(), Value::Array(releases));
        object.insert("current_release".to_owned(), current);
        object.insert("downloads".to_owned(), json!(total));
    }
    Some(merged)
}

pub fn merge_releases(lists: Vec<Vec<Value>>) -> Vec<Value> {
    let mut merged: Vec<Value> = Vec::new();
    let mut seen = BTreeSet::new();
    for list in lists {
        for release in list {
            let key = text(&release, "slug").to_owned();
            if seen.insert(key) {
                merged.push(release);
            }
        }
    }
    merged
}

pub fn sort_releases(releases: &mut [Value], sort_by: Option<&str>) {
    let by_version = |a: &Value, b: &Value| {
        let module = |value: &Value| {
            release_slug_of(value)
                .map(|(module, _)| module.slug())
                .unwrap_or_default()
        };
        module(a)
            .cmp(&module(b))
            .then_with(|| names::newest_first(version_of(a), version_of(b)))
    };
    match sort_by {
        Some("downloads") => releases.sort_by(|a, b| {
            downloads(b)
                .cmp(&downloads(a))
                .then_with(|| by_version(a, b))
        }),
        Some("release_date") => {
            releases.sort_by(|a, b| created(b).cmp(&created(a)).then_with(|| by_version(a, b)))
        }
        _ => releases.sort_by(by_version),
    }
}

pub fn sort_modules(modules: &mut [Value], sort_by: Option<&str>, query: Option<&str>) {
    let slug = |value: &Value| text(value, "slug").to_owned();
    let latest = |value: &Value| value.get("current_release").and_then(created);
    let popular = |a: &Value, b: &Value| {
        downloads(b)
            .cmp(&downloads(a))
            .then_with(|| slug(a).cmp(&slug(b)))
    };
    match (sort_by, query) {
        (Some("latest_release"), _) => modules.sort_by(|a, b| {
            latest(b)
                .cmp(&latest(a))
                .then_with(|| slug(a).cmp(&slug(b)))
        }),
        (Some("rank") | None, Some(query)) => {
            let wanted = query.trim().to_lowercase();
            let rank = |value: &Value| match text(value, "name") == wanted || slug(value) == wanted
            {
                true => Ordering::Less,
                false => Ordering::Greater,
            };
            modules.sort_by(|a, b| rank(a).cmp(&rank(b)).then_with(|| popular(a, b)))
        }
        _ => modules.sort_by(popular),
    }
}

fn words_of(module: &Value) -> String {
    let current = module
        .get("current_release")
        .cloned()
        .unwrap_or(Value::Null);
    let metadata = current.get("metadata").cloned().unwrap_or(Value::Null);
    let mut words = vec![
        text(module, "slug").to_owned(),
        text(&metadata, "summary").to_owned(),
        text(&metadata, "description").to_owned(),
        text(&metadata, "author").to_owned(),
    ];
    for tag in current
        .get("tags")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        words.push(tag.to_owned());
    }
    words.join(" ").to_lowercase()
}

fn tags_of(module: &Value) -> Vec<String> {
    module
        .pointer("/current_release/tags")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_lowercase)
        .collect()
}

pub fn module_matches(module: &Value, params: &Params) -> bool {
    let slug = text(module, "slug").to_lowercase();
    if let Some(query) = params.get("query") {
        let words = words_of(module);
        if !query
            .to_lowercase()
            .split_whitespace()
            .all(|word| words.contains(word))
        {
            return false;
        }
    }
    if let Some(tag) = params.get("tag")
        && !tags_of(module).contains(&tag.to_lowercase())
    {
        return false;
    }
    if let Some(owner) = params.get("owner")
        && !slug.starts_with(&format!("{}-", owner.to_lowercase()))
    {
        return false;
    }
    if let Some(prefix) = params.get("starts_with")
        && !slug.starts_with(&prefix.to_lowercase())
    {
        return false;
    }
    let slugs: Vec<String> = params
        .list("slugs")
        .iter()
        .filter_map(|wanted| Module::parse(wanted))
        .map(|module| module.slug())
        .collect();
    if !slugs.is_empty() && !slugs.contains(&slug) {
        return false;
    }
    if params.flag("with_pdk") && module.pointer("/current_release/pdk") != Some(&Value::Bool(true))
    {
        return false;
    }
    true
}

pub fn release_matches(release: &Value, params: &Params, module: Option<&Module>) -> bool {
    let Some((found, _)) = release_slug_of(release) else {
        return false;
    };
    if module.is_some_and(|module| *module != found) {
        return false;
    }
    if let Some(owner) = params.get("owner")
        && !found.owner.eq_ignore_ascii_case(owner)
    {
        return false;
    }
    !params.flag("with_pdk") || release.get("pdk") == Some(&Value::Bool(true))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(slug: &str, version: &str, at: &str, downloads: i64) -> Value {
        let module = Module::parse(slug).unwrap();
        json!({
            "uri": module.release_uri(version),
            "slug": module.release_slug(version),
            "version": version,
            "module": module_summary(&module),
            "metadata": {"name": slug, "version": version, "summary": "A test module", "project_page": "https://example.org"},
            "tags": ["demo"],
            "created_at": at,
            "updated_at": at,
            "downloads": downloads,
            "file_uri": module.file_uri(version),
            "file_size": 10,
        })
    }

    #[test]
    fn pages_link_with_the_filters_of_the_request() {
        let params = Params::of(Some(
            "module=acme-tools&limit=2&offset=2&exclude_fields=readme+changelog",
        ));
        assert_eq!((params.limit, params.offset), (2, 2));
        assert_eq!(params.list("exclude_fields"), ["readme", "changelog"]);
        let items: Vec<Value> = (0..5).map(|n| json!({"n": n, "readme": "x"})).collect();
        let page = params.page("/v3/releases", items, 5);
        assert_eq!(page["results"], json!([{"n": 2}, {"n": 3}]));
        assert_eq!(page["pagination"]["total"], 5);
        assert_eq!(
            page["pagination"]["next"],
            "/v3/releases?module=acme-tools&exclude_fields=readme+changelog&limit=2&offset=4"
        );
        assert_eq!(
            page["pagination"]["previous"],
            "/v3/releases?module=acme-tools&exclude_fields=readme+changelog&limit=2&offset=0"
        );
        let last = Params::of(Some("offset=4&limit=2")).page("/v3/modules", vec![json!(1); 5], 5);
        assert_eq!(last["pagination"]["next"], Value::Null);
        assert_eq!(last["pagination"]["first"], "/v3/modules?limit=2&offset=0");
        assert_eq!(Params::of(Some("limit=500")).limit, MAX_LIMIT);
        assert_eq!(Params::of(Some("limit=0&offset=x")).limit, DEFAULT_LIMIT);
        assert!(Params::of(Some("module=bad")).module().is_err());
        assert_eq!(
            Params::of(Some("module=Acme%2Ftools"))
                .module()
                .unwrap()
                .unwrap()
                .slug(),
            "acme-tools"
        );
    }

    #[test]
    fn remote_documents_point_back_at_this_repository() {
        let remote = json!({
            "uri": "https://forge.example/api/v3/releases/acme-tools-1.0.0",
            "slug": "acme-tools-1.0.0",
            "version": "1.0.0",
            "module": {"uri": "/v3/modules/acme-tools", "slug": "acme-tools", "owner": {"uri": "https://forge.example/v3/users/acme"}},
            "file_uri": "https://cdn.example/files/acme-tools-1.0.0.tar.gz",
            "file_md5": "ab",
        });
        let ours = rewrite_remote_release(remote).unwrap();
        assert_eq!(ours["file_uri"], "/v3/files/acme-tools-1.0.0.tar.gz");
        assert_eq!(ours["uri"], "/v3/releases/acme-tools-1.0.0");
        assert_eq!(ours["module"]["owner"]["uri"], "/v3/users/acme");
        assert_eq!(ours["file_md5"], "ab");
        assert_eq!(
            relative("/repository/forge/v3/releases?offset=20"),
            "/v3/releases?offset=20"
        );
        let module = rewrite_remote_module(json!({
            "slug": "acme-tools",
            "current_release": {"slug": "acme-tools-1.0.0", "version": "1.0.0", "file_uri": "x"},
            "releases": [
                {"slug": "acme-tools-0.9.0", "version": "0.9.0", "file_uri": "x"},
                {"slug": "acme-tools-1.0.0", "version": "1.0.0", "file_uri": "x"},
                {"slug": "acme-tools-x", "version": "x"},
            ],
        }))
        .unwrap();
        assert_eq!(
            module["current_release"]["file_uri"],
            "/v3/files/acme-tools-1.0.0.tar.gz"
        );
        assert_eq!(module["releases"][0]["version"], "1.0.0");
        assert_eq!(module["releases"].as_array().unwrap().len(), 2);
        assert_eq!(
            module["releases"][1]["file_uri"],
            "/v3/files/acme-tools-0.9.0.tar.gz"
        );
    }

    #[test]
    fn modules_take_the_current_release_and_groups_merge_members() {
        let module = Module::parse("acme-tools").unwrap();
        let releases = [
            release("acme-tools", "1.0.0", "2026-01-02 03:04:05 +0000", 3),
            release("acme-tools", "2.0.0-rc.1", "2026-03-01 00:00:00 +0000", 1),
            release("acme-tools", "1.1.0", "2026-02-01 00:00:00 +0100", 2),
        ];
        let document = module_document(&module, &releases).unwrap();
        assert_eq!(document["current_release"]["version"], "1.1.0");
        assert_eq!(document["releases"][0]["version"], "2.0.0-rc.1");
        assert_eq!(document["downloads"], 6);
        assert_eq!(document["created_at"], "2026-01-02 03:04:05 +0000");
        assert_eq!(document["updated_at"], "2026-03-01 00:00:00 +0000");
        assert_eq!(document["homepage_url"], "https://example.org");
        let other = module_document(
            &module,
            &[
                release("acme-tools", "1.2.0", "2026-04-01 00:00:00 +0000", 5),
                release("acme-tools", "1.0.0", "2026-01-01 00:00:00 +0000", 7),
            ],
        )
        .unwrap();
        let merged = merge_modules(vec![document, other]).unwrap();
        assert_eq!(merged["current_release"]["version"], "1.2.0");
        assert_eq!(merged["downloads"], 18);
        let versions: Vec<&str> = merged["releases"]
            .as_array()
            .unwrap()
            .iter()
            .map(version_of)
            .collect();
        assert_eq!(versions, ["2.0.0-rc.1", "1.2.0", "1.1.0", "1.0.0"]);
        assert!(module_document(&module, &[]).is_none());
    }

    #[test]
    fn releases_and_modules_sort_and_filter_like_the_forge() {
        let mut releases = vec![
            release("acme-tools", "1.0.0", "2026-01-02 00:00:00 +0000", 9),
            release("acme-alpha", "1.0.0", "2026-03-01 00:00:00 +0000", 1),
            release("acme-tools", "1.10.0", "2026-02-01 00:00:00 +0000", 2),
        ];
        sort_releases(&mut releases, None);
        let slugs: Vec<&str> = releases.iter().map(|r| text(r, "slug")).collect();
        assert_eq!(
            slugs,
            ["acme-alpha-1.0.0", "acme-tools-1.10.0", "acme-tools-1.0.0"]
        );
        sort_releases(&mut releases, Some("release_date"));
        assert_eq!(text(&releases[0], "slug"), "acme-alpha-1.0.0");
        sort_releases(&mut releases, Some("downloads"));
        assert_eq!(text(&releases[0], "slug"), "acme-tools-1.0.0");
        let tools = Module::parse("acme-tools").unwrap();
        let params = Params::of(Some("owner=ACME"));
        assert!(release_matches(&releases[0], &params, Some(&tools)));
        assert!(!release_matches(
            &releases[0],
            &Params::of(Some("owner=other")),
            None
        ));
        let mut modules = vec![
            module_document(&tools, &releases[..1]).unwrap(),
            json!({"slug": "acme-other", "name": "other", "downloads": 100}),
            json!({"slug": "zed-tools", "name": "tools", "downloads": 1}),
        ];
        sort_modules(&mut modules, None, Some("tools"));
        assert_eq!(text(&modules[0], "slug"), "acme-tools");
        assert_eq!(text(&modules[1], "slug"), "zed-tools");
        sort_modules(&mut modules, Some("downloads"), None);
        assert_eq!(text(&modules[0], "slug"), "acme-other");
        let found = |query: &str| module_matches(&modules[1], &Params::of(Some(query)));
        assert!(found("query=test+module") && found("tag=Demo") && found("owner=acme"));
        assert!(found("starts_with=acme-t") && found("slugs=acme-tools,zed-tools"));
        assert!(!found("query=missing") && !found("tag=other") && !found("slugs=zed-tools"));
    }
}
