use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use astrum_db::audit::Event;
use astrum_security::ip_allow::{Surface, decode_path, surface_of_decoded};
use astrum_security::sessions::throttle_address;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde_json::json;

use crate::client::ClientAddress;
use crate::{AppState, Caller};

pub const DENIED_MESSAGE: &str = "Access from this address is not allowed";

const AUDIT_WINDOW: Duration = Duration::from_secs(600);
const SWEEP_EVERY: Duration = Duration::from_secs(30);
const TRACKED_LIMIT: usize = 10_000;

#[derive(Default)]
struct Audited {
    at: HashMap<(String, Surface), Instant>,
    swept: Option<Instant>,
}

#[derive(Default)]
pub struct Denials {
    counts: [AtomicU64; 3],
    audited: Mutex<Audited>,
}

impl Denials {
    pub(crate) fn counts(&self) -> [(Surface, u64); 3] {
        Surface::ALL.map(|surface| {
            (
                surface,
                self.counts[surface as usize].load(Ordering::Relaxed),
            )
        })
    }

    fn refused(&self, address: Option<IpAddr>, surface: Surface) -> bool {
        self.counts[surface as usize].fetch_add(1, Ordering::Relaxed);
        let now = Instant::now();
        let key = (throttle_address(address), surface);
        let mut audited = self
            .audited
            .lock()
            .expect("the denial audit lock is never poisoned");
        if audited
            .at
            .get(&key)
            .is_some_and(|at| now.duration_since(*at) < AUDIT_WINDOW)
        {
            return false;
        }
        if audited.at.len() >= TRACKED_LIMIT {
            let due = audited
                .swept
                .is_none_or(|swept| now.duration_since(swept) >= SWEEP_EVERY);
            if due {
                audited
                    .at
                    .retain(|_, at| now.duration_since(*at) < AUDIT_WINDOW);
                audited.swept = Some(now);
            }
            if audited.at.len() >= TRACKED_LIMIT {
                tracing::debug!(
                    surface = surface.as_str(),
                    "a denied request was counted but not audited: too many addresses were denied in the last ten minutes"
                );
                return false;
            }
        }
        audited.at.insert(key, now);
        true
    }
}

pub async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let decoded = decode_path(request.uri().path());
    let Some(surface) = surface_of_decoded(request.method().as_str(), decoded.as_deref()) else {
        return next.run(request).await;
    };
    let model = state.security().model();
    if !model.ip_allow.restricts(surface) {
        return next.run(request).await;
    }
    let address = request
        .extensions()
        .get::<ClientAddress>()
        .and_then(|client| client.0);
    if model.ip_allow.permits(surface, address) {
        return next.run(request).await;
    }
    if state.ip_allow_denials().refused(address, surface) {
        let path = decoded.as_deref().unwrap_or_else(|| request.uri().path());
        audit_denial(&state, address, surface, path).await;
    }
    denied()
}

async fn audit_denial(state: &AppState, address: Option<IpAddr>, surface: Surface, path: &str) {
    state
        .audit(
            &Caller(None),
            Event::new("security.ipAllow", "denied", surface.as_str()).with(json!({
                "address": address.map(|address| address.to_string()),
                "surface": surface.as_str(),
                "path": audited_path(path),
            })),
        )
        .await;
}

fn audited_path(path: &str) -> String {
    let mut parts = path.split('/').filter(|part| !part.is_empty());
    match (parts.next(), parts.next()) {
        (Some("service"), Some(second)) => format!("/service/{second}"),
        (Some(first), _) => format!("/{first}"),
        (None, _) => "/".to_owned(),
    }
}

fn denied() -> Response {
    (
        StatusCode::FORBIDDEN,
        axum::Json(json!({"message": DENIED_MESSAGE})),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_denial_is_audited_once_per_address_and_surface() {
        let denials = Denials::default();
        let address: Option<IpAddr> = Some("203.0.113.7".parse().unwrap());
        assert!(denials.refused(address, Surface::Management));
        assert!(!denials.refused(address, Surface::Management));
        assert!(denials.refused(address, Surface::Content));
        assert!(denials.refused(Some("203.0.113.8".parse().unwrap()), Surface::Management));
        assert!(denials.refused(None, Surface::Management));
        assert!(!denials.refused(None, Surface::Management));
        assert_eq!(
            denials.counts(),
            [
                (Surface::Management, 5),
                (Surface::Content, 1),
                (Surface::Agents, 0)
            ]
        );
    }

    #[test]
    fn an_ipv6_network_of_64_bits_is_one_address_for_the_audit() {
        let denials = Denials::default();
        assert!(denials.refused(Some("2001:db8:1:2::1".parse().unwrap()), Surface::Agents));
        assert!(!denials.refused(Some("2001:db8:1:2::ffff".parse().unwrap()), Surface::Agents));
        assert!(denials.refused(Some("2001:db8:1:3::1".parse().unwrap()), Surface::Agents));
    }

    #[test]
    fn the_audited_path_keeps_no_secret_tail() {
        assert_eq!(audited_path("/mcp/upload/ticket-secret"), "/mcp");
        assert_eq!(
            audited_path("/service/rest/v1/repositories"),
            "/service/rest"
        );
        assert_eq!(audited_path("/repository/raw/a.txt"), "/repository");
        assert_eq!(audited_path("/"), "/");
    }
}
