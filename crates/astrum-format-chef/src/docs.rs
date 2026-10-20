use std::collections::BTreeMap;

use astrum_content::{CHALLENGE, ContentError, ContentRequest, Ctx};
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::Response;
use serde_json::{Map, Value, json};

use crate::names;

pub const JSON_TYPE: &str = "application/json";
pub const DEFAULT_ITEMS: usize = 10;
pub const MAX_ITEMS: usize = 100;

pub type Universe = BTreeMap<String, BTreeMap<String, Value>>;

#[derive(Clone, Debug)]
pub struct Urls {
    pub base: String,
}

impl Urls {
    pub fn of(ctx: &Ctx) -> Self {
        Self {
            base: ctx.repository_url(),
        }
    }

    pub fn api(&self) -> String {
        format!("{}/api/v1", self.base)
    }

    pub fn cookbook(&self, name: &str) -> String {
        format!("{}/api/v1/cookbooks/{name}", self.base)
    }

    pub fn version(&self, name: &str, version: &str) -> String {
        format!("{}/versions/{version}", self.cookbook(name))
    }

    pub fn download(&self, name: &str, version: &str) -> String {
        format!("{}/download", self.version(name, version))
    }
}

pub fn json(status: StatusCode, request: Option<&ContentRequest>, value: &Value) -> Response {
    let body = value.to_string();
    let head = request.is_some_and(|request| request.method == Method::HEAD);
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, JSON_TYPE)
        .header(header::CONTENT_LENGTH, body.len())
        .header(header::CACHE_CONTROL, "no-cache")
        .body(match head {
            true => Body::empty(),
            false => Body::from(body),
        })
        .expect("a JSON response is valid")
}

pub fn etagged(request: &ContentRequest, body: bytes::Bytes, etag: &str) -> Response {
    let etag = format!("\"{etag}\"");
    let unchanged = request.header("if-none-match").is_some_and(|tags| {
        tags.split(',')
            .map(|tag| tag.trim().trim_start_matches("W/"))
            .any(|tag| tag == "*" || tag == etag)
    });
    let builder = Response::builder()
        .header(header::ETAG, &etag)
        .header(header::CACHE_CONTROL, "no-cache");
    let response = match (unchanged, &request.method) {
        (true, _) => builder.status(StatusCode::NOT_MODIFIED).body(Body::empty()),
        (false, &Method::HEAD) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, JSON_TYPE)
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::empty()),
        (false, _) => builder
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, JSON_TYPE)
            .header(header::CONTENT_LENGTH, body.len())
            .body(Body::from(body)),
    };
    response.expect("a JSON response is valid")
}

fn code_of(error: &ContentError) -> &'static str {
    match error {
        ContentError::NotFound(_) => "NOT_FOUND",
        ContentError::Unauthorized => "UNAUTHORIZED",
        ContentError::Forbidden => "FORBIDDEN",
        ContentError::MethodNotAllowed { .. } => "METHOD_NOT_ALLOWED",
        ContentError::Conflict(_) => "CONFLICT",
        ContentError::BadGateway(_) | ContentError::Unavailable(_) => "REMOTE_FAILED",
        ContentError::Internal(_) => "SERVER_ERROR",
        _ => "INVALID_DATA",
    }
}

pub fn error(error: ContentError, message: Option<String>) -> Response {
    let status = error.status();
    let message = message.unwrap_or_else(|| match &error {
        ContentError::Internal(inner) => {
            tracing::error!(
                error = format!("{inner:#}"),
                "a Chef Supermarket request failed"
            );
            "the server failed; see its log".to_owned()
        }
        other => other.to_string(),
    });
    let mut response = json(
        status,
        None,
        &json!({"error_messages": [message], "error_code": code_of(&error)}),
    );
    let headers = response.headers_mut();
    match error {
        ContentError::Unauthorized => {
            headers.insert(
                header::WWW_AUTHENTICATE,
                HeaderValue::from_static(CHALLENGE),
            );
        }
        ContentError::MethodNotAllowed { allow, .. } => {
            headers.insert(header::ALLOW, HeaderValue::from_static(allow));
        }
        _ => {}
    }
    response
}

pub fn merge_universe(into: &mut Universe, other: Universe) {
    for (name, versions) in other {
        let known = into.entry(name).or_default();
        for (version, dependencies) in versions {
            known.entry(version).or_insert(dependencies);
        }
    }
}

pub fn parse_universe(value: &Value) -> Universe {
    let mut universe = Universe::new();
    let Some(cookbooks) = value.as_object() else {
        return universe;
    };
    for (name, versions) in cookbooks {
        if !names::valid_name(name) {
            continue;
        }
        let Some(versions) = versions.as_object() else {
            continue;
        };
        let mut kept = BTreeMap::new();
        for (version, entry) in versions {
            if !names::valid_version(version) {
                continue;
            }
            let dependencies = match entry.get("dependencies") {
                Some(Value::Object(dependencies)) => Value::Object(dependencies.clone()),
                _ => json!({}),
            };
            kept.insert(version.clone(), dependencies);
        }
        if !kept.is_empty() {
            universe.insert(name.clone(), kept);
        }
    }
    universe
}

fn push_text(out: &mut Vec<u8>, text: &str) {
    serde_json::to_writer(&mut *out, text).expect("a string can be written as JSON");
}

pub fn render_universe(
    urls: &Urls,
    universe: &Universe,
    visible: impl Fn(&str, &str) -> bool,
) -> Vec<u8> {
    let api = urls.api();
    let mut out = Vec::with_capacity(universe.len() * 256 + 2);
    out.push(b'{');
    let mut first = true;
    for (name, versions) in universe {
        let mut opened = false;
        for (version, dependencies) in versions {
            if !visible(name, version) {
                continue;
            }
            if opened {
                out.push(b',');
            } else {
                if !first {
                    out.push(b',');
                }
                first = false;
                push_text(&mut out, name);
                out.extend_from_slice(b":{");
                opened = true;
            }
            push_text(&mut out, version);
            out.extend_from_slice(b":{\"location_type\":\"opscode\",\"location_path\":");
            push_text(&mut out, &api);
            out.extend_from_slice(b",\"download_url\":");
            push_text(&mut out, &urls.download(name, version));
            out.extend_from_slice(b",\"dependencies\":");
            serde_json::to_writer(&mut out, dependencies).expect("a value can be written as JSON");
            out.push(b'}');
        }
        if opened {
            out.push(b'}');
        }
    }
    out.push(b'}');
    out
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cookbook {
    pub name: String,
    pub document: Map<String, Value>,
    pub versions: Vec<String>,
}

fn downloads_of(document: &Map<String, Value>) -> (i64, Map<String, Value>) {
    let downloads = document
        .get("metrics")
        .and_then(|metrics| metrics.get("downloads"));
    let total = downloads
        .and_then(|downloads| downloads.get("total"))
        .and_then(Value::as_i64)
        .unwrap_or(0);
    let versions = downloads
        .and_then(|downloads| downloads.get("versions"))
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    (total, versions)
}

impl Cookbook {
    pub fn from_remote(name: &str, value: &Value) -> Option<Self> {
        let mut document = value.as_object()?.clone();
        let mut versions: Vec<String> = document
            .remove("versions")
            .and_then(|versions| versions.as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(Value::as_str)
            .filter_map(names::version_of_url)
            .collect();
        if versions.is_empty()
            && let Some(latest) = document
                .get("latest_version")
                .and_then(Value::as_str)
                .and_then(names::version_of_url)
        {
            versions.push(latest);
        }
        document.remove("latest_version");
        versions.dedup();
        document.insert("name".to_owned(), json!(name));
        Some(Self {
            name: name.to_owned(),
            document,
            versions,
        })
    }

    pub fn merge(&mut self, other: Cookbook) {
        for version in other.versions {
            if !self.versions.contains(&version) {
                self.versions.push(version);
            }
        }
        let (mine, mut by_version) = downloads_of(&self.document);
        let (theirs, other_versions) = downloads_of(&other.document);
        for (version, count) in other_versions {
            by_version.entry(version).or_insert(count);
        }
        let metrics = self
            .document
            .entry("metrics")
            .or_insert_with(|| json!({"followers": 0, "collaborators": 0}));
        if let Some(metrics) = metrics.as_object_mut() {
            metrics.insert(
                "downloads".to_owned(),
                json!({"total": mine + theirs, "versions": by_version}),
            );
        }
        for (key, value) in other.document {
            self.document.entry(key).or_insert(value);
        }
    }

    pub fn render(&self, urls: &Urls, visible: impl Fn(&str) -> bool) -> Option<Value> {
        let mut versions: Vec<String> = self
            .versions
            .iter()
            .filter(|version| visible(version))
            .cloned()
            .collect();
        if versions.is_empty() {
            return None;
        }
        names::newest_first(&mut versions);
        let mut document = self.document.clone();
        document.insert(
            "latest_version".to_owned(),
            json!(urls.version(&self.name, &versions[0])),
        );
        document.insert(
            "versions".to_owned(),
            json!(
                versions
                    .iter()
                    .map(|version| urls.version(&self.name, version))
                    .collect::<Vec<_>>()
            ),
        );
        Some(Value::Object(document))
    }
}

pub fn render_version(
    urls: &Urls,
    name: &str,
    version: &str,
    mut document: Map<String, Value>,
) -> Value {
    document.insert("version".to_owned(), json!(version));
    document.insert("cookbook".to_owned(), json!(urls.cookbook(name)));
    document.insert("file".to_owned(), json!(urls.download(name, version)));
    Value::Object(document)
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Item {
    pub name: String,
    pub maintainer: Value,
    pub description: Value,
    pub versions: Vec<String>,
    pub updated_at: String,
    pub created_at: String,
    pub downloads: i64,
}

impl Item {
    pub fn from_remote(value: &Value) -> Option<Self> {
        let name = value.get("cookbook_name")?.as_str()?;
        names::valid_name(name).then(|| Self {
            name: name.to_owned(),
            maintainer: value
                .get("cookbook_maintainer")
                .cloned()
                .unwrap_or(Value::Null),
            description: value
                .get("cookbook_description")
                .cloned()
                .unwrap_or(Value::Null),
            ..Self::default()
        })
    }

    pub fn render(&self, urls: &Urls) -> Value {
        json!({
            "cookbook_name": self.name,
            "cookbook_maintainer": self.maintainer,
            "cookbook_description": self.description,
            "cookbook": urls.cookbook(&self.name),
        })
    }

    pub fn matches(&self, query: &str) -> bool {
        let wanted = query.trim().to_lowercase();
        if wanted.is_empty() {
            return true;
        }
        let text = |value: &Value| value.as_str().unwrap_or_default().to_lowercase();
        wanted.split_whitespace().all(|word| {
            self.name.to_lowercase().contains(word)
                || text(&self.description).contains(word)
                || text(&self.maintainer).contains(word)
        })
    }

    pub fn rank(&self, query: &str) -> u8 {
        let wanted = query.trim().to_lowercase();
        let name = self.name.to_lowercase();
        match () {
            _ if wanted.is_empty() => 0,
            _ if name == wanted => 0,
            _ if name.starts_with(&wanted) => 1,
            _ if name.contains(&wanted) => 2,
            _ => 3,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Query {
    pub q: String,
    pub start: usize,
    pub items: usize,
    pub order: Option<String>,
    pub user: Option<String>,
}

impl Query {
    pub fn of(query: Option<&str>) -> Self {
        let mut parsed = Self {
            q: String::new(),
            start: 0,
            items: DEFAULT_ITEMS,
            order: None,
            user: None,
        };
        for (key, value) in url::form_urlencoded::parse(query.unwrap_or_default().as_bytes()) {
            match key.as_ref() {
                "q" => parsed.q = value.into_owned(),
                "start" => parsed.start = value.trim().parse().unwrap_or(0),
                "items" => {
                    parsed.items = value
                        .trim()
                        .parse()
                        .unwrap_or(DEFAULT_ITEMS)
                        .clamp(1, MAX_ITEMS)
                }
                "order" if !value.trim().is_empty() => parsed.order = Some(value.into_owned()),
                "user" if !value.trim().is_empty() => parsed.user = Some(value.into_owned()),
                _ => {}
            }
        }
        parsed
    }

    pub fn page(&self, urls: &Urls, total: usize, items: &[Item]) -> Value {
        let shown: Vec<Value> = items
            .iter()
            .skip(self.start)
            .take(self.items)
            .map(|item| item.render(urls))
            .collect();
        json!({"start": self.start, "total": total, "items": shown})
    }

    pub fn sort(&self, items: &mut [Item]) {
        match self.order.as_deref() {
            Some("recently_updated") => {
                items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at).then(a.name.cmp(&b.name)))
            }
            Some("recently_added") => {
                items.sort_by(|a, b| b.created_at.cmp(&a.created_at).then(a.name.cmp(&b.name)))
            }
            Some("most_downloaded") => {
                items.sort_by(|a, b| b.downloads.cmp(&a.downloads).then(a.name.cmp(&b.name)))
            }
            _ => items.sort_by(|a, b| {
                a.rank(&self.q)
                    .cmp(&b.rank(&self.q))
                    .then(a.name.cmp(&b.name))
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn urls() -> Urls {
        Urls {
            base: "https://repo.example/repository/chef".to_owned(),
        }
    }

    #[test]
    fn the_universe_points_back_at_the_repository() {
        let upstream = json!({
            "apt": {
                "7.4.0": {"location_type": "opscode", "location_path": "https://supermarket.example:443/api/v1", "download_url": "https://supermarket.example:443/api/v1/cookbooks/apt/versions/7.4.0/download", "dependencies": {}},
                "0.1.0\n": {"location_type": "opscode", "dependencies": {}},
            },
            "nginx": {"9.0.0": {"dependencies": {"apt": ">= 0.0.0"}}},
            "bad name": {"1.0.0": {}},
        });
        let universe = parse_universe(&upstream);
        assert_eq!(universe.len(), 2);
        assert_eq!(universe["apt"].len(), 1);
        let rendered: Value =
            serde_json::from_slice(&render_universe(&urls(), &universe, |_, _| true)).unwrap();
        assert_eq!(
            rendered["apt"]["7.4.0"],
            json!({
                "location_type": "opscode",
                "location_path": "https://repo.example/repository/chef/api/v1",
                "download_url": "https://repo.example/repository/chef/api/v1/cookbooks/apt/versions/7.4.0/download",
                "dependencies": {},
            })
        );
        assert_eq!(
            rendered["nginx"]["9.0.0"]["dependencies"]["apt"],
            ">= 0.0.0"
        );
        let hidden: Value =
            serde_json::from_slice(&render_universe(&urls(), &universe, |name, _| {
                name == "nginx"
            }))
            .unwrap();
        assert_eq!(hidden, json!({"nginx": rendered["nginx"].clone()}));
        let mut merged = universe.clone();
        merge_universe(
            &mut merged,
            parse_universe(&json!({"apt": {"7.4.0": {"dependencies": {"x": "1.0"}}, "8.0.0": {}}})),
        );
        assert_eq!(merged["apt"]["7.4.0"], json!({}));
        assert_eq!(merged["apt"]["8.0.0"], json!({}));
    }

    #[test]
    fn cookbook_documents_merge_and_name_the_newest_version() {
        let mut first = Cookbook::from_remote(
            "apt",
            &json!({
                "name": "apt",
                "maintainer": "sous-chefs",
                "latest_version": "https://supermarket.example/api/v1/cookbooks/apt/versions/7_4_0",
                "versions": ["https://supermarket.example/api/v1/cookbooks/apt/versions/7_4_0", "https://supermarket.example/api/v1/cookbooks/apt/versions/7.10.0"],
                "metrics": {"downloads": {"total": 5, "versions": {"7.4.0": 5}}, "followers": 3, "collaborators": 1},
            }),
        )
        .unwrap();
        let second = Cookbook {
            name: "apt".to_owned(),
            document: json!({"maintainer": "acme", "category": "Other", "metrics": {"downloads": {"total": 2, "versions": {"7.4.0": 9, "8.0.0": 2}}}})
                .as_object()
                .unwrap()
                .clone(),
            versions: vec!["8.0.0".to_owned(), "7.4.0".to_owned()],
        };
        first.merge(second);
        assert_eq!(
            names::highest(first.versions.iter().map(String::as_str)),
            Some("8.0.0")
        );
        let rendered = first.render(&urls(), |_| true).unwrap();
        assert_eq!(
            rendered["latest_version"],
            "https://repo.example/repository/chef/api/v1/cookbooks/apt/versions/8.0.0"
        );
        assert_eq!(rendered["versions"].as_array().unwrap().len(), 3);
        assert_eq!(rendered["maintainer"], "sous-chefs");
        assert_eq!(rendered["category"], "Other");
        assert_eq!(rendered["metrics"]["downloads"]["total"], 7);
        assert_eq!(rendered["metrics"]["downloads"]["versions"]["7.4.0"], 5);
        assert_eq!(rendered["metrics"]["followers"], 3);
        assert!(first.render(&urls(), |_| false).is_none());
        let version = render_version(&urls(), "apt", "7.4.0", Map::new());
        assert_eq!(
            version["file"],
            "https://repo.example/repository/chef/api/v1/cookbooks/apt/versions/7.4.0/download"
        );
    }

    #[test]
    fn queries_page_sort_and_match() {
        let query = Query::of(Some("q=nginx&start=1&items=500&order="));
        assert_eq!(query.items, MAX_ITEMS);
        assert_eq!(query.start, 1);
        assert_eq!(query.order, None);
        let item = |name: &str, description: &str| Item {
            name: name.to_owned(),
            description: json!(description),
            ..Item::default()
        };
        let mut items = vec![
            item("yum-nginx", "repository"),
            item("nginx", "web server"),
            item("apache", "Not NGINX"),
            item("nginx_conf", "files"),
        ];
        assert!(items.iter().all(|one| one.matches("nginx")));
        assert!(!item("apt", "packages").matches("nginx"));
        query.sort(&mut items);
        let names: Vec<&str> = items.iter().map(|one| one.name.as_str()).collect();
        assert_eq!(names, ["nginx", "nginx_conf", "yum-nginx", "apache"]);
        let page = query.page(&urls(), 4, &items);
        assert_eq!(page["start"], 1);
        assert_eq!(page["total"], 4);
        assert_eq!(page["items"][0]["cookbook_name"], "nginx_conf");
        assert_eq!(
            page["items"][0]["cookbook"],
            "https://repo.example/repository/chef/api/v1/cookbooks/nginx_conf"
        );
    }
}
