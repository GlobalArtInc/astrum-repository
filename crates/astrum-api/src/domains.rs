use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use astrum_repo::config::DomainRouting;
use axum::extract::{Request, State};
use axum::http::uri::PathAndQuery;
use axum::http::{StatusCode, Uri, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use crate::AppState;

const FRESH: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Entry {
    Root { repository: String, format: String },
    Shared { repositories: Vec<(String, String)> },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DomainRoute {
    Root {
        origin: String,
        repository: String,
    },
    Shared {
        origin: String,
        repositories: Arc<Vec<(String, String)>>,
    },
}

impl DomainRoute {
    pub fn origin(&self) -> &str {
        match self {
            Self::Root { origin, .. } | Self::Shared { origin, .. } => origin,
        }
    }

    pub fn admits(&self, name: &str) -> bool {
        match self {
            Self::Root { repository, .. } => repository.eq_ignore_ascii_case(name),
            Self::Shared { repositories, .. } => repositories
                .iter()
                .any(|(_, repository)| repository.eq_ignore_ascii_case(name)),
        }
    }

    pub fn repository_for(&self, segment: &str) -> Option<&str> {
        match self {
            Self::Root { .. } => None,
            Self::Shared { repositories, .. } => repositories
                .iter()
                .find(|(path, _)| path.eq_ignore_ascii_case(segment))
                .map(|(_, repository)| repository.as_str()),
        }
    }

    pub fn segment_of(&self, name: &str) -> Option<&str> {
        match self {
            Self::Root { .. } => None,
            Self::Shared { repositories, .. } => repositories
                .iter()
                .find(|(_, repository)| repository.eq_ignore_ascii_case(name))
                .map(|(path, _)| path.as_str()),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Plan {
    Pass,
    Rewrite(String),
    NotFound,
}

pub type Table = Arc<HashMap<String, Entry>>;

#[derive(Default)]
pub(crate) struct Cache {
    value: RwLock<Option<(Instant, Table)>>,
}

pub fn table(repositories: &[astrum_repo::config::Repository]) -> HashMap<String, Entry> {
    let mut table: HashMap<String, Entry> = HashMap::new();
    let mut sorted: Vec<&astrum_repo::config::Repository> = repositories.iter().collect();
    sorted.sort_by(|one, other| one.name.cmp(&other.name));
    for repository in sorted {
        let Some((host, routing)) = repository.domain() else {
            continue;
        };
        match (routing, table.get_mut(host)) {
            (DomainRouting::Root, None) => {
                table.insert(
                    host.to_owned(),
                    Entry::Root {
                        repository: repository.name.clone(),
                        format: repository.format.id.to_owned(),
                    },
                );
            }
            (DomainRouting::Path, None) => {
                table.insert(
                    host.to_owned(),
                    Entry::Shared {
                        repositories: vec![(
                            repository.domain_path().to_owned(),
                            repository.name.clone(),
                        )],
                    },
                );
            }
            (DomainRouting::Path, Some(Entry::Shared { repositories }))
                if !repositories
                    .iter()
                    .any(|(path, _)| path.eq_ignore_ascii_case(repository.domain_path())) =>
            {
                repositories.push((repository.domain_path().to_owned(), repository.name.clone()));
            }
            _ => tracing::warn!(
                host,
                repository = %repository.name,
                "two repositories claim the same domain; the first one keeps it"
            ),
        }
    }
    table
}

pub fn plan(entry: &Entry, path: &str) -> Plan {
    let docker = path == "/v2" || path.starts_with("/v2/");
    if path.starts_with("/service/") {
        return Plan::Pass;
    }
    let first = path
        .trim_start_matches('/')
        .split('/')
        .next()
        .unwrap_or_default();
    match entry {
        Entry::Root { repository, format } => {
            if docker && format == "docker" {
                return Plan::Pass;
            }
            if let Some(rest) = path.strip_prefix("/repository/") {
                let named = rest.split('/').next().unwrap_or_default();
                return if named.eq_ignore_ascii_case(repository) {
                    Plan::Pass
                } else {
                    Plan::NotFound
                };
            }
            let rest = if path.is_empty() { "/" } else { path };
            Plan::Rewrite(format!("/repository/{repository}{rest}"))
        }
        Entry::Shared { repositories } => {
            if docker {
                return Plan::Pass;
            }
            if let Some(rest) = path.strip_prefix("/repository/") {
                let named = rest.split('/').next().unwrap_or_default();
                let admitted = repositories
                    .iter()
                    .any(|(_, repository)| repository.eq_ignore_ascii_case(named));
                return if admitted { Plan::Pass } else { Plan::NotFound };
            }
            let found = repositories
                .iter()
                .find(|(segment, _)| segment.eq_ignore_ascii_case(first))
                .map(|(_, repository)| repository);
            match found {
                Some(repository) => {
                    let rest = &path.trim_start_matches('/')[first.len()..];
                    let rest = if rest.is_empty() { "/" } else { rest };
                    Plan::Rewrite(format!("/repository/{repository}{rest}"))
                }
                None => Plan::NotFound,
            }
        }
    }
}

impl AppState {
    pub async fn domains(&self) -> Table {
        let cache = &self.inner.domains;
        if let Some((at, value)) = cache.value.read().expect("never poisoned").clone()
            && at.elapsed() < FRESH
        {
            return value;
        }
        let value = match astrum_repo::store::list(self.db().conn()).await {
            Ok(repositories) => Arc::new(table(&repositories)),
            Err(error) => {
                tracing::warn!(%error, "the repository domains could not be read");
                match cache.value.read().expect("never poisoned").clone() {
                    Some((_, stale)) => return stale,
                    None => Arc::new(HashMap::new()),
                }
            }
        };
        *cache.value.write().expect("never poisoned") = Some((Instant::now(), value.clone()));
        value
    }

    pub fn forget_domains(&self) {
        *self.inner.domains.value.write().expect("never poisoned") = None;
    }
}

pub fn request_host(request: &Request) -> Option<String> {
    let headers = request.headers();
    let first = |name: &str| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.split(',').next())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
    };
    first("x-forwarded-host")
        .or_else(|| first(header::HOST.as_str()))
        .or_else(|| {
            request
                .uri()
                .authority()
                .map(|authority| authority.to_string())
        })
}

pub fn bare_host(host: &str) -> String {
    let host = match host.rsplit_once(':') {
        Some((name, port))
            if !host.starts_with('[')
                && !port.is_empty()
                && port.chars().all(|c| c.is_ascii_digit()) =>
        {
            name
        }
        _ => host,
    };
    astrum_repo::config::normalize_host(host)
}

fn scheme(request: &Request) -> String {
    request
        .headers()
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|scheme| scheme == "http" || scheme == "https")
        .or_else(|| request.uri().scheme_str().map(str::to_owned))
        .unwrap_or_else(|| "http".to_owned())
}

pub async fn route(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    let Some(raw) = request_host(&request) else {
        return next.run(request).await;
    };
    let host = bare_host(&raw);
    let domains = state.domains().await;
    let Some(entry) = domains.get(&host) else {
        return next.run(request).await;
    };
    let origin = format!("{}://{raw}", scheme(&request));
    let route = match entry {
        Entry::Root { repository, .. } => DomainRoute::Root {
            origin,
            repository: repository.clone(),
        },
        Entry::Shared { repositories } => DomainRoute::Shared {
            origin,
            repositories: Arc::new(repositories.clone()),
        },
    };
    match plan(entry, request.uri().path()) {
        Plan::Pass => {}
        Plan::NotFound => return StatusCode::NOT_FOUND.into_response(),
        Plan::Rewrite(path) => {
            let target = match request.uri().query() {
                Some(query) => format!("{path}?{query}"),
                None => path,
            };
            let mut parts = request.uri().clone().into_parts();
            parts.path_and_query = PathAndQuery::try_from(target.as_str()).ok();
            match Uri::from_parts(parts) {
                Ok(uri) => *request.uri_mut() = uri,
                Err(_) => return StatusCode::BAD_REQUEST.into_response(),
            }
        }
    }
    request.extensions_mut().insert(route);
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(repository: &str, format: &str) -> Entry {
        Entry::Root {
            repository: repository.to_owned(),
            format: format.to_owned(),
        }
    }

    fn shared(repositories: &[(&str, &str)]) -> Entry {
        Entry::Shared {
            repositories: repositories
                .iter()
                .map(|(segment, name)| ((*segment).to_owned(), (*name).to_owned()))
                .collect(),
        }
    }

    #[test]
    fn a_root_domain_serves_one_repository_at_the_top() {
        let docker = root("docker-onprem", "docker");
        assert_eq!(plan(&docker, "/v2/"), Plan::Pass);
        assert_eq!(plan(&docker, "/v2/herald-api/manifests/0.2.0"), Plan::Pass);
        assert_eq!(plan(&docker, "/v2/token"), Plan::Pass);
        assert_eq!(
            plan(&docker, "/"),
            Plan::Rewrite("/repository/docker-onprem/".to_owned())
        );

        let raw = root("files", "raw");
        assert_eq!(
            plan(&raw, "/tools/1.0/install.sh"),
            Plan::Rewrite("/repository/files/tools/1.0/install.sh".to_owned())
        );
        assert_eq!(
            plan(&raw, "/v2/x"),
            Plan::Rewrite("/repository/files/v2/x".to_owned())
        );
        assert_eq!(plan(&raw, "/repository/files/tools/a"), Plan::Pass);
        assert_eq!(plan(&raw, "/repository/FILES/tools/a"), Plan::Pass);
        assert_eq!(plan(&raw, "/repository/other/tools/a"), Plan::NotFound);
        assert_eq!(plan(&raw, "/service/rest/v1/components"), Plan::Pass);
    }

    #[test]
    fn a_shared_domain_takes_the_repository_from_the_path() {
        let images = shared(&[("prod", "docker-onprem"), ("files", "files")]);
        assert_eq!(plan(&images, "/v2/prod/herald-api/tags/list"), Plan::Pass);
        assert_eq!(
            plan(&images, "/prod/a"),
            Plan::Rewrite("/repository/docker-onprem/a".to_owned())
        );
        assert_eq!(plan(&images, "/docker-onprem/a"), Plan::NotFound);
        assert_eq!(plan(&images, "/repository/docker-onprem/a"), Plan::Pass);
        assert_eq!(
            plan(&images, "/files/tools/1.0/install.sh"),
            Plan::Rewrite("/repository/files/tools/1.0/install.sh".to_owned())
        );
        assert_eq!(
            plan(&images, "/files"),
            Plan::Rewrite("/repository/files/".to_owned())
        );
        assert_eq!(
            plan(&images, "/Files/a"),
            Plan::Rewrite("/repository/files/a".to_owned())
        );
        assert_eq!(plan(&images, "/repository/files/a"), Plan::Pass);
        assert_eq!(plan(&images, "/repository/secret/a"), Plan::NotFound);
        assert_eq!(plan(&images, "/secret/a"), Plan::NotFound);
        assert_eq!(plan(&images, "/"), Plan::NotFound);
    }

    #[test]
    fn hosts_lose_their_port_and_case() {
        assert_eq!(
            bare_host("Docker-Prem.GlobalArt.hu:443"),
            "docker-prem.globalart.hu"
        );
        assert_eq!(bare_host("images.globalart.hu"), "images.globalart.hu");
        assert_eq!(bare_host("images.globalart.hu."), "images.globalart.hu");
    }

    #[test]
    fn routes_admit_their_repositories() {
        let route = DomainRoute::Shared {
            origin: "https://images.globalart.hu".to_owned(),
            repositories: Arc::new(vec![("prod".to_owned(), "docker-onprem".to_owned())]),
        };
        assert!(route.admits("Docker-Onprem"));
        assert!(!route.admits("docker-hub"));
        assert_eq!(route.repository_for("PROD"), Some("docker-onprem"));
        assert_eq!(route.repository_for("docker-onprem"), None);
        assert_eq!(route.segment_of("docker-onprem"), Some("prod"));
        assert_eq!(route.origin(), "https://images.globalart.hu");
    }
}
