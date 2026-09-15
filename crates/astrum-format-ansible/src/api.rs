use astrum_content::{ContentError, ContentRequest, Ctx};
use axum::body::Body;
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::response::Response;
use chrono::{DateTime, SecondsFormat, Utc};
use serde_json::{Map, Value, json};

use crate::names::{self, Coll};

pub const JSON_TYPE: &str = "application/json";
pub const DEFAULT_LIMIT: usize = 10;
pub const MAX_LIMIT: usize = 1000;

pub fn time(at: DateTime<Utc>) -> String {
    at.to_rfc3339_opts(SecondsFormat::Micros, true)
}

/// Addresses of one repository: `root` is the path from the host root, as Galaxy writes `href`
/// and pagination links, `absolute` the full URL used for downloads.
#[derive(Clone, Debug)]
pub struct Urls {
    pub root: String,
    pub absolute: String,
}

impl Urls {
    pub fn of(ctx: &Ctx) -> Self {
        let absolute = ctx.repository_url();
        let root = url::Url::parse(&absolute)
            .map(|url| url.path().trim_end_matches('/').to_owned())
            .unwrap_or_else(|_| format!("/repository/{}", ctx.name()));
        Self { root, absolute }
    }

    pub fn api(&self) -> String {
        format!("{}/api/", self.root)
    }

    pub fn collections(&self) -> String {
        format!("{}/api/v3/collections/", self.root)
    }

    pub fn collection(&self, coll: &Coll) -> String {
        format!("{}{}", self.root, coll.api_path())
    }

    pub fn versions(&self, coll: &Coll) -> String {
        format!("{}versions/", self.collection(coll))
    }

    pub fn version(&self, coll: &Coll, version: &str) -> String {
        format!("{}versions/{version}/", self.collection(coll))
    }

    pub fn download(&self, coll: &Coll, version: &str) -> String {
        format!("{}{}", self.absolute, coll.artifact_path(version))
    }

    pub fn import(&self, id: &str) -> String {
        format!("{}/api/v3/imports/collections/{id}/", self.root)
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
        .body(if head {
            Body::empty()
        } else {
            Body::from(body)
        })
        .expect("a JSON response is valid")
}

/// The error document of the v3 API, which `ansible-galaxy` prints as
/// `(HTTP Code: <status>, Message: <detail> Code: <code>)`.
pub fn error(error: ContentError) -> Response {
    let status = error.status();
    let (code, title) = match &error {
        ContentError::NotFound(_) => ("not_found", "Not found."),
        ContentError::Unauthorized => (
            "not_authenticated",
            "Authentication credentials were not provided.",
        ),
        ContentError::Forbidden => (
            "permission_denied",
            "You do not have permission to perform this action.",
        ),
        ContentError::MethodNotAllowed { .. } => ("method_not_allowed", "Method not allowed."),
        ContentError::Conflict(_) => ("conflict", "Conflict."),
        ContentError::BadGateway(_) | ContentError::Unavailable(_) => {
            ("remote_failed", "The remote failed.")
        }
        ContentError::Internal(_) => ("server_error", "Server error."),
        _ => ("invalid", "Invalid input."),
    };
    let detail = match &error {
        ContentError::Internal(inner) => {
            tracing::error!(
                error = format!("{inner:#}"),
                "an Ansible Galaxy request failed"
            );
            "the server failed; see its log".to_owned()
        }
        other => other.to_string(),
    };
    let mut response = json(
        status,
        None,
        &json!({"errors": [{"status": status.as_u16().to_string(), "code": code, "title": title, "detail": detail}]}),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Page {
    pub limit: usize,
    pub offset: usize,
}

impl Page {
    pub fn of(query: Option<&str>) -> Self {
        let mut page = Page {
            limit: DEFAULT_LIMIT,
            offset: 0,
        };
        for pair in query.unwrap_or_default().split('&') {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            match (key, value.parse::<usize>()) {
                ("limit" | "page_size", Ok(limit)) if limit > 0 => {
                    page.limit = limit.min(MAX_LIMIT)
                }
                ("offset", Ok(offset)) => page.offset = offset,
                _ => {}
            }
        }
        page
    }

    pub fn list(self, base: &str, items: Vec<Value>) -> Value {
        let count = items.len();
        let link = |offset: usize| json!(format!("{base}?limit={}&offset={offset}", self.limit));
        let last = count.saturating_sub(1) / self.limit * self.limit;
        let next = self.offset + self.limit;
        let data: Vec<Value> = items
            .into_iter()
            .skip(self.offset)
            .take(self.limit)
            .collect();
        json!({
            "meta": {"count": count},
            "links": {
                "first": link(0),
                "previous": if self.offset == 0 { Value::Null } else { link(self.offset.saturating_sub(self.limit)) },
                "next": if next < count { link(next) } else { Value::Null },
                "last": link(last),
            },
            "data": data,
        })
    }
}

/// One version of a collection as a repository knows it, before its addresses are written.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub version: String,
    pub created_at: String,
    pub updated_at: String,
    pub requires_ansible: Value,
    pub marks: Value,
}

impl Entry {
    pub fn from_remote(value: &Value) -> Option<Self> {
        let version = value.get("version")?.as_str()?.to_owned();
        let text = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        Some(Entry {
            created_at: text("created_at"),
            updated_at: text("updated_at"),
            requires_ansible: value
                .get("requires_ansible")
                .cloned()
                .unwrap_or(Value::Null),
            marks: value.get("marks").cloned().unwrap_or_else(|| json!([])),
            version,
        })
    }

    pub fn document(&self, urls: &Urls, coll: &Coll) -> Value {
        json!({
            "version": self.version,
            "href": urls.version(coll, &self.version),
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "requires_ansible": self.requires_ansible,
            "marks": self.marks,
        })
    }
}

pub fn sort_entries(entries: &mut [Entry]) {
    entries.sort_by(|a, b| names::newest_first(&a.version, &b.version));
}

/// A collection as a repository knows it, before its addresses are written.
#[derive(Clone, Debug, PartialEq)]
pub struct Summary {
    pub coll: Coll,
    pub deprecated: bool,
    pub highest: String,
    pub created_at: String,
    pub updated_at: String,
    pub download_count: i64,
}

impl Summary {
    pub fn from_entries(coll: Coll, entries: &[Entry], download_count: i64) -> Option<Self> {
        let highest = names::highest(entries.iter().map(|entry| entry.version.as_str()))?;
        Some(Summary {
            highest: highest.to_owned(),
            created_at: entries
                .iter()
                .map(|entry| entry.created_at.clone())
                .filter(|at| !at.is_empty())
                .min()
                .unwrap_or_default(),
            updated_at: entries
                .iter()
                .map(|entry| entry.updated_at.clone())
                .max()
                .unwrap_or_default(),
            coll,
            deprecated: false,
            download_count,
        })
    }

    pub fn from_remote(coll: Coll, value: &Value) -> Option<Self> {
        let highest = value
            .get("highest_version")
            .and_then(|highest| highest.get("version"))
            .and_then(Value::as_str)?
            .to_owned();
        let text = |key: &str| {
            value
                .get(key)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        Some(Summary {
            coll,
            deprecated: value
                .get("deprecated")
                .and_then(Value::as_bool)
                .unwrap_or(false),
            highest,
            created_at: text("created_at"),
            updated_at: text("updated_at"),
            download_count: value
                .get("download_count")
                .and_then(Value::as_i64)
                .unwrap_or(0),
        })
    }

    /// Folds what another member of a group knows of the same collection into this one.
    pub fn merge(&mut self, other: &Summary) {
        if names::highest([self.highest.as_str(), other.highest.as_str()])
            == Some(other.highest.as_str())
        {
            self.highest = other.highest.clone();
        }
        if !other.created_at.is_empty()
            && (self.created_at.is_empty() || other.created_at < self.created_at)
        {
            self.created_at = other.created_at.clone();
        }
        if other.updated_at > self.updated_at {
            self.updated_at = other.updated_at.clone();
        }
        self.download_count += other.download_count;
    }

    pub fn document(&self, urls: &Urls) -> Value {
        json!({
            "href": urls.collection(&self.coll),
            "namespace": self.coll.namespace,
            "name": self.coll.name,
            "deprecated": self.deprecated,
            "versions_url": urls.versions(&self.coll),
            "highest_version": {
                "href": urls.version(&self.coll, &self.highest),
                "version": self.highest,
            },
            "created_at": self.created_at,
            "updated_at": self.updated_at,
            "download_count": self.download_count,
        })
    }
}

/// Points the addresses of a version document at this repository; the rest stays as the
/// repository that holds the version wrote it.
pub fn rewrite_version(mut document: Value, urls: &Urls, coll: &Coll, version: &str) -> Value {
    if let Some(object) = document.as_object_mut() {
        object.insert("href".to_owned(), json!(urls.version(coll, version)));
        object.insert(
            "download_url".to_owned(),
            json!(urls.download(coll, version)),
        );
        object.insert("version".to_owned(), json!(version));
        object.insert("name".to_owned(), json!(coll.name));
        let collection = object
            .entry("collection")
            .or_insert_with(|| Value::Object(Map::new()));
        if let Some(collection) = collection.as_object_mut() {
            collection.insert("name".to_owned(), json!(coll.name));
            collection.insert("href".to_owned(), json!(urls.collection(coll)));
        }
        let namespace = object
            .entry("namespace")
            .or_insert_with(|| Value::Object(Map::new()));
        match namespace {
            Value::Object(namespace) => {
                namespace.insert("name".to_owned(), json!(coll.namespace));
            }
            other => *other = json!({"name": coll.namespace}),
        }
    }
    document
}

#[cfg(test)]
mod tests {
    use super::*;

    fn urls() -> Urls {
        Urls {
            root: "/repository/galaxy".to_owned(),
            absolute: "https://r.example/repository/galaxy".to_owned(),
        }
    }

    #[test]
    fn pages_link_like_the_v3_api() {
        let page = Page::of(Some("limit=2&offset=2"));
        assert_eq!(
            page,
            Page {
                limit: 2,
                offset: 2
            }
        );
        let items: Vec<Value> = (0..5).map(|n| json!(n)).collect();
        let list = page.list("/r/versions/", items);
        assert_eq!(list["meta"]["count"], 5);
        assert_eq!(list["data"], json!([2, 3]));
        assert_eq!(list["links"]["next"], "/r/versions/?limit=2&offset=4");
        assert_eq!(list["links"]["previous"], "/r/versions/?limit=2&offset=0");
        assert_eq!(list["links"]["last"], "/r/versions/?limit=2&offset=4");
        let last =
            Page::of(Some("limit=2&offset=4")).list("/r/", (0..5).map(|n| json!(n)).collect());
        assert_eq!(last["links"]["next"], Value::Null);
        assert_eq!(Page::of(Some("limit=0&offset=x")), Page::of(None));
        assert_eq!(Page::of(Some("limit=100000")).limit, MAX_LIMIT);
        let empty = Page::of(None).list("/r/", vec![]);
        assert_eq!(empty["links"]["last"], "/r/?limit=10&offset=0");
    }

    #[test]
    fn version_documents_point_back_at_this_repository() {
        let coll = Coll::new("community", "general").unwrap();
        let remote = json!({
            "version": "8.0.0",
            "href": "/api/v3/plugin/x/",
            "download_url": "https://galaxy.example/download/community-general-8.0.0.tar.gz",
            "artifact": {"sha256": "ab", "filename": "community-general-8.0.0.tar.gz", "size": 3},
            "collection": {"id": "1", "name": "general", "href": "/api/x/"},
            "namespace": {"name": "community", "metadata_sha256": "cd"},
            "metadata": {"dependencies": {}},
        });
        let ours = rewrite_version(remote, &urls(), &coll, "8.0.0");
        assert_eq!(
            ours["download_url"],
            "https://r.example/repository/galaxy/community/general/8.0.0/community-general-8.0.0.tar.gz"
        );
        assert_eq!(
            ours["href"],
            "/repository/galaxy/api/v3/collections/community/general/versions/8.0.0/"
        );
        assert_eq!(
            ours["collection"]["href"],
            "/repository/galaxy/api/v3/collections/community/general/"
        );
        assert_eq!(ours["collection"]["id"], "1");
        assert_eq!(ours["namespace"]["metadata_sha256"], "cd");
        assert_eq!(ours["artifact"]["sha256"], "ab");
    }

    #[test]
    fn group_summaries_take_the_highest_version_and_widest_dates() {
        let coll = Coll::new("a", "b").unwrap();
        let entry = |version: &str, at: &str| Entry {
            version: version.to_owned(),
            created_at: at.to_owned(),
            updated_at: at.to_owned(),
            requires_ansible: Value::Null,
            marks: json!([]),
        };
        let mut first = Summary::from_entries(
            coll.clone(),
            &[
                entry("1.0.0", "2026-01-02"),
                entry("2.0.0-rc.1", "2026-01-05"),
            ],
            3,
        )
        .unwrap();
        assert_eq!(first.highest, "1.0.0");
        let second = Summary::from_entries(coll, &[entry("1.5.0", "2026-01-01")], 4).unwrap();
        first.merge(&second);
        assert_eq!(first.highest, "1.5.0");
        assert_eq!(first.created_at, "2026-01-01");
        assert_eq!(first.updated_at, "2026-01-05");
        assert_eq!(first.download_count, 7);
        let document = first.document(&urls());
        assert_eq!(
            document["highest_version"]["href"],
            "/repository/galaxy/api/v3/collections/a/b/versions/1.5.0/"
        );
    }
}
