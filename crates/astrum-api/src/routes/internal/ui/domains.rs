use std::collections::BTreeSet;
use std::net::IpAddr;
use std::time::Duration;

use astrum_repo::config::{normalize_host, valid_host};
use axum::Router;
use axum::extract::{Query, State};
use axum::routing::get;
use serde::{Deserialize, Serialize};

use super::UI;
use crate::domains::bare_host;
use crate::routes::repositories::BaseUrl;
use crate::{ApiError, AppState, Caller};

const LOOKUP: Duration = Duration::from_secs(3);

pub fn routes() -> Router<AppState> {
    Router::new().route(&format!("{UI}/domains/check"), get(check))
}

#[derive(Deserialize)]
struct Asked {
    host: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Checked {
    pub host: String,
    pub status: &'static str,
    pub addresses: Vec<String>,
    pub server_addresses: Vec<String>,
}

async fn check(
    State(_state): State<AppState>,
    caller: Caller,
    base: BaseUrl,
    Query(asked): Query<Asked>,
) -> Result<axum::Json<Checked>, ApiError> {
    caller.signed_in()?;
    let host = normalize_host(asked.host.as_deref().unwrap_or_default());
    if !valid_host(&host) {
        return Ok(axum::Json(Checked {
            host,
            status: "invalid",
            addresses: Vec::new(),
            server_addresses: Vec::new(),
        }));
    }
    let server = base
        .0
        .split_once("://")
        .map(|(_, rest)| rest.split('/').next().unwrap_or_default())
        .map(bare_host)
        .unwrap_or_default();
    let addresses = resolve(&host).await;
    let server_addresses = match server.trim_matches(['[', ']']).parse::<IpAddr>() {
        Ok(address) => vec![address.to_string()],
        Err(_) if server.is_empty() => Vec::new(),
        Err(_) => resolve(&server).await,
    };
    let status = verdict(&addresses, &server_addresses);
    Ok(axum::Json(Checked {
        host,
        status,
        addresses,
        server_addresses,
    }))
}

async fn resolve(host: &str) -> Vec<String> {
    let found = tokio::time::timeout(LOOKUP, tokio::net::lookup_host((host, 443))).await;
    let addresses: BTreeSet<String> = match found {
        Ok(Ok(found)) => found.map(|address| address.ip().to_string()).collect(),
        _ => BTreeSet::new(),
    };
    addresses.into_iter().collect()
}

pub fn verdict(addresses: &[String], server: &[String]) -> &'static str {
    if addresses.is_empty() {
        "missing"
    } else if server.is_empty() {
        "unknown"
    } else if addresses.iter().any(|address| server.contains(address)) {
        "same"
    } else {
        "other"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn list(items: &[&str]) -> Vec<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    #[test]
    fn the_verdict_compares_the_domain_with_the_server() {
        assert_eq!(verdict(&[], &list(&["89.235.182.84"])), "missing");
        assert_eq!(verdict(&list(&["89.235.182.84"]), &[]), "unknown");
        assert_eq!(
            verdict(&list(&["89.235.182.84"]), &list(&["89.235.182.84"])),
            "same"
        );
        assert_eq!(
            verdict(
                &list(&["10.0.0.1", "89.235.182.84"]),
                &list(&["89.235.182.84"])
            ),
            "same"
        );
        assert_eq!(
            verdict(&list(&["10.0.0.1"]), &list(&["89.235.182.84"])),
            "other"
        );
    }
}
