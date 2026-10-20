mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use astrum_notify::Timing;
use astrum_notify::alerts::Problem;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode, header};
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{Value, json};
use support::{Auth, Fixture};

const CAPABILITIES: &str = "/service/rest/v1/capabilities";
const METRICS: &str = "/service/metrics/prometheus";
const CHANNELS: &str = "/service/rest/internal/ui/notifications";

#[derive(Clone, Default)]
struct Receiver {
    requests: Arc<Mutex<Vec<(HeaderMap, Value)>>>,
}

impl Receiver {
    fn taken(&self) -> Vec<(HeaderMap, Value)> {
        self.requests.lock().unwrap().drain(..).collect()
    }
}

async fn receive(State(receiver): State<Receiver>, headers: HeaderMap, body: Bytes) -> Json<Value> {
    receiver
        .requests
        .lock()
        .unwrap()
        .push((headers, serde_json::from_slice(&body).unwrap()));
    Json(json!({"ok": true}))
}

async fn start_receiver() -> (String, Receiver) {
    let receiver = Receiver::default();
    let app = Router::new()
        .route("/{*path}", post(receive))
        .with_state(receiver.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{address}"), receiver)
}

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    fixture.state.notifier().set_timing(Timing {
        quiet: Duration::ZERO,
        max_delay: Duration::ZERO,
        backoff: Duration::ZERO,
        attempts: 2,
    });
    fixture
}

async fn capability(fixture: &Fixture, body: Value) -> support::Reply {
    fixture
        .call(Method::POST, CAPABILITIES, Auth::Admin, Some(body))
        .await
}

async fn hosted(fixture: &Fixture, name: &str) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(json!({
                "name": name,
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
                "raw": {"contentDisposition": "ATTACHMENT"}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn line<'a>(text: &'a str, prefix: &str) -> Option<&'a str> {
    text.lines().find(|line| line.starts_with(prefix))
}

#[tokio::test]
async fn prometheus_metrics_are_served_once_the_capability_is_on() {
    let fixture = ready().await;
    assert_eq!(
        fixture
            .call(Method::GET, METRICS, Auth::Admin, None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let created = capability(
        &fixture,
        json!({"type": "metrics.prometheus", "enabled": true, "properties": {"repositories": "true"}}),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    assert_eq!(created.json()["tags"]["Category"], "Monitoring");
    assert_eq!(created.json()["description"], METRICS);

    hosted(&fixture, "files").await;
    let uploaded = fixture
        .send_body(
            Method::PUT,
            "/repository/files/a/b.txt",
            Auth::Admin,
            Body::from("hello"),
            &[],
        )
        .await;
    assert_eq!(uploaded.status, StatusCode::CREATED);
    fixture
        .call(Method::GET, "/repository/nowhere/x", Auth::None, None)
        .await;

    assert_eq!(
        fixture
            .call(Method::GET, METRICS, Auth::None, None)
            .await
            .status,
        StatusCode::UNAUTHORIZED
    );
    fixture.add_user("plain", "plain-pass", &[]).await;
    assert_eq!(
        fixture
            .call(
                Method::GET,
                METRICS,
                Auth::Basic("plain", "plain-pass"),
                None
            )
            .await
            .status,
        StatusCode::FORBIDDEN
    );
    let role = fixture
        .call(
            Method::POST,
            "/service/rest/v1/security/roles",
            Auth::Admin,
            Some(json!({"id": "scrapers", "name": "scrapers", "description": "", "privileges": ["as-metrics-all"], "roles": []})),
        )
        .await;
    assert_eq!(role.status, StatusCode::OK, "{}", role.text());
    fixture
        .add_user("scraper", "scraper-pass", &["scrapers"])
        .await;
    let reply = fixture
        .call(
            Method::GET,
            METRICS,
            Auth::Basic("scraper", "scraper-pass"),
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    assert_eq!(
        reply.headers[header::CONTENT_TYPE],
        "text/plain; version=0.0.4; charset=utf-8"
    );
    let text = reply.text();
    for expected in [
        "# TYPE astrum_http_requests_total counter",
        "astrum_metrics_collection_complete 1",
        "astrum_blobstore_up{blobstore=\"default\",type=\"file\"} 1",
        "astrum_blobstore_blobs{blobstore=\"default\",type=\"file\"} 1",
        "astrum_repository_online{repository=\"files\",format=\"raw\",type=\"hosted\"} 1",
        "astrum_repository_requests_total{repository=\"files\",method=\"PUT\",status=\"2xx\"} 1",
        "astrum_health_check_healthy{check=\"Database\"} 1",
        "astrum_http_request_duration_seconds_bucket{area=\"repository\",le=\"+Inf\"} 2",
    ] {
        assert!(
            text.contains(expected),
            "{expected} is missing from\n{text}"
        );
    }
    assert!(line(&text, "astrum_build_info{version=").is_some());
    assert!(
        line(
            &text,
            "astrum_blobstore_available_bytes{blobstore=\"default\""
        )
        .is_some()
    );
    assert!(
        line(
            &text,
            "astrum_http_requests_total{area=\"rest\",method=\"POST\",status=\"2xx\"}"
        )
        .is_some()
    );
    assert!(!text.contains("nowhere"));

    let id = "metrics.prometheus";
    let updated = fixture
        .call(
            Method::PUT,
            &format!("{CAPABILITIES}/{id}"),
            Auth::Admin,
            Some(json!({"type": id, "enabled": true, "properties": {"repositories": "false"}})),
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    let text = fixture
        .call(Method::GET, METRICS, Auth::Admin, None)
        .await
        .text();
    assert!(!text.contains("astrum_repository_"));
    assert!(text.contains("astrum_blobstore_up"));

    let disabled = fixture
        .call(
            Method::PUT,
            &format!("{CAPABILITIES}/{id}"),
            Auth::Admin,
            Some(json!({"type": id, "enabled": false, "properties": {}})),
        )
        .await;
    assert_eq!(disabled.status, StatusCode::OK);
    assert_eq!(
        fixture
            .call(Method::GET, METRICS, Auth::Admin, None)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
}

async fn allow_loopback(fixture: &Fixture) {
    astrum_db::setting::put(
        fixture.state.db().conn(),
        SSRF_KEY,
        &SsrfProtection {
            enabled: true,
            allowed_ips: vec!["127.0.0.1".to_owned()],
            allowed_domains: vec![],
        },
    )
    .await
    .unwrap();
}

async fn channel(fixture: &Fixture, name: &str, url: &str, language: &str) {
    let reply = fixture
        .call(
            Method::POST,
            CHANNELS,
            Auth::Admin,
            Some(json!({
                "name": name,
                "type": "webhook",
                "secret": url,
                "events": ["published"],
                "language": language,
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

async fn space_alert(fixture: &Fixture, method: Method, minimum: &str) -> support::Reply {
    let body = json!({
        "type": "blobstores.space",
        "enabled": true,
        "properties": {
            "minimumFree": minimum,
            "channels": "ops, OPS",
            "interval": "5",
            "repeat": "24",
            "quota": "true",
            "unavailable": "true",
            "recovery": "true",
        }
    });
    match method {
        Method::POST => capability(fixture, body).await,
        _ => {
            fixture
                .call(
                    Method::PUT,
                    &format!("{CAPABILITIES}/blobstores.space"),
                    Auth::Admin,
                    Some(body),
                )
                .await
        }
    }
}

#[tokio::test]
async fn low_space_is_reported_once_and_again_when_it_is_over() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let (url, receiver) = start_receiver().await;

    let unknown = space_alert(&fixture, Method::POST, "10").await;
    assert_eq!(
        unknown.status,
        StatusCode::BAD_REQUEST,
        "{}",
        unknown.text()
    );
    assert!(
        unknown
            .text()
            .contains("No notification channel is named ops")
    );

    channel(&fixture, "Ops", &format!("{url}/ops"), "en").await;
    channel(&fixture, "Releases", &format!("{url}/releases"), "en").await;
    let created = space_alert(&fixture, Method::POST, "4000000000").await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    assert_eq!(created.json()["properties"]["channels"], "Ops");
    assert_eq!(
        created.json()["status"],
        "The blob stores have not been checked yet."
    );

    let outcome = astrum_api::space::check(&fixture.state)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(outcome.alerted.len(), 1);
    assert_eq!(outcome.alerted[0].store, "default");
    assert!(matches!(
        &outcome.alerted[0].problem,
        Problem::LowSpace { minimum, .. } if minimum == "4.00 EB"
    ));
    assert_eq!(outcome.channels, 1);
    fixture.state.notifier().flush_now().await.unwrap();
    let sent = receiver.taken();
    assert_eq!(sent.len(), 1);
    let (headers, body) = &sent[0];
    assert_eq!(headers["x-astrum-event"], "blobstore.alert");
    assert_eq!(body["event"], "blobstore.alert");
    let text = body["text"].as_str().unwrap();
    assert!(
        text.starts_with("⚠️ *Blob stores need attention*\n• `default`: "),
        "{text}"
    );
    assert!(text.contains("free, less than 4.00 EB"), "{text}");

    let again = astrum_api::space::check(&fixture.state)
        .await
        .unwrap()
        .unwrap();
    assert!(again.alerted.is_empty());
    assert!(again.recovered.is_empty());
    fixture.state.notifier().flush_now().await.unwrap();
    assert!(receiver.taken().is_empty());

    let read = fixture
        .call(
            Method::GET,
            &format!("{CAPABILITIES}/blobstores.space"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    let status = read["status"].as_str().unwrap();
    assert!(status.starts_with("Last checked at "), "{status}");
    assert!(status.contains("\ndefault: "), "{status}");

    capability(
        &fixture,
        json!({"type": "metrics.prometheus", "enabled": true, "properties": {}}),
    )
    .await;
    let metrics = fixture
        .call(Method::GET, METRICS, Auth::Admin, None)
        .await
        .text();
    assert!(
        metrics
            .contains("astrum_blobstore_space_alert{blobstore=\"default\",problem=\"lowSpace\"} 1"),
        "{metrics}"
    );

    let updated = space_alert(&fixture, Method::PUT, "1").await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    let outcome = astrum_api::space::check(&fixture.state)
        .await
        .unwrap()
        .unwrap();
    assert!(outcome.alerted.is_empty());
    assert_eq!(outcome.recovered.len(), 1);
    fixture.state.notifier().flush_now().await.unwrap();
    let sent = receiver.taken();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].1["event"], "blobstore.recovered");
    let text = sent[0].1["text"].as_str().unwrap();
    assert!(
        text.starts_with("✅ *Blob stores are fine again*\n• `default`: "),
        "{text}"
    );
    assert!(text.ends_with(" free again"), "{text}");

    let removed = fixture
        .call(
            Method::DELETE,
            &format!("{CAPABILITIES}/blobstores.space"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(removed.status, StatusCode::NO_CONTENT);
    assert!(
        astrum_api::space::check(&fixture.state)
            .await
            .unwrap()
            .is_none()
    );
}
