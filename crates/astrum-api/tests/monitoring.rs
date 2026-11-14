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
    space_alert_with(fixture, method, minimum, None).await
}

async fn space_alert_with(
    fixture: &Fixture,
    method: Method,
    minimum: &str,
    forecast_days: Option<&str>,
) -> support::Reply {
    let mut body = json!({
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
    if let Some(days) = forecast_days {
        body["properties"]["forecastDays"] = json!(days);
    }
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

async fn quota_repository(fixture: &Fixture, name: &str) {
    let reply = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(json!({
                "name": name,
                "online": true,
                "storage": {
                    "blobStoreName": "default",
                    "strictContentTypeValidation": false,
                    "writePolicy": "ALLOW",
                    "quota": {"limitBytes": 1000, "warnPercent": 50, "enforce": false}
                },
                "raw": {"contentDisposition": "ATTACHMENT"}
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn execute(fixture: &Fixture, sql: &str, values: Vec<sea_orm::Value>) {
    use sea_orm::{ConnectionTrait, DbBackend, Statement};
    fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            values,
        ))
        .await
        .unwrap();
}

async fn checked(
    fixture: &Fixture,
    receiver: &Receiver,
) -> (astrum_api::space::Outcome, Vec<Value>) {
    let outcome = astrum_api::space::check(&fixture.state)
        .await
        .unwrap()
        .unwrap();
    fixture.state.notifier().flush_now().await.unwrap();
    let bodies = receiver.taken().into_iter().map(|(_, body)| body).collect();
    (outcome, bodies)
}

#[tokio::test]
async fn a_repository_crossing_warn_then_its_limit_then_recovering_sends_three_messages() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let (url, receiver) = start_receiver().await;
    channel(&fixture, "Ops", &format!("{url}/ops"), "en").await;
    let created = space_alert(&fixture, Method::POST, "1").await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    quota_repository(&fixture, "capped").await;
    hosted(&fixture, "free").await;
    let set_used = |bytes: i64| {
        execute(
            &fixture,
            "UPDATE repository_usage SET bytes = $2 WHERE repository = $1",
            vec!["capped".into(), bytes.into()],
        )
    };

    set_used(400).await;
    let (outcome, sent) = checked(&fixture, &receiver).await;
    assert!(outcome.alerted.is_empty() && outcome.recovered.is_empty());
    assert!(sent.is_empty());

    set_used(600).await;
    let (outcome, sent) = checked(&fixture, &receiver).await;
    assert_eq!(outcome.alerted.len(), 1);
    assert!(outcome.recovered.is_empty());
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0]["event"], "blobstore.alert");
    let text = sent[0]["text"].as_str().unwrap();
    assert!(
        text.contains("• `capped`: repository is at 60% of its 1.00 KB quota (600 B used)"),
        "{text}"
    );
    let (_, sent) = checked(&fixture, &receiver).await;
    assert!(sent.is_empty());

    set_used(1000).await;
    let (outcome, sent) = checked(&fixture, &receiver).await;
    assert_eq!(outcome.alerted.len(), 1);
    assert!(outcome.recovered.is_empty());
    assert_eq!(sent.len(), 1);
    let text = sent[0]["text"].as_str().unwrap();
    assert!(
        text.contains("• `capped`: repository reached its 1.00 KB quota (1.00 KB used)"),
        "{text}"
    );
    let (_, sent) = checked(&fixture, &receiver).await;
    assert!(sent.is_empty());

    set_used(100).await;
    let (outcome, sent) = checked(&fixture, &receiver).await;
    assert!(outcome.alerted.is_empty());
    assert_eq!(outcome.recovered.len(), 1);
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0]["event"], "blobstore.recovered");
    let text = sent[0]["text"].as_str().unwrap();
    assert!(
        text.contains("• `capped`: repository is below its quota warning again"),
        "{text}"
    );
    let (_, sent) = checked(&fixture, &receiver).await;
    assert!(sent.is_empty());
}

#[tokio::test]
async fn the_forecast_alert_follows_forecast_days_and_recovers_when_growth_stops() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    let (url, receiver) = start_receiver().await;
    channel(&fixture, "Ops", &format!("{url}/ops"), "en").await;
    let quota = fixture
        .call(
            Method::PUT,
            "/service/rest/v1/blobstores/file/default",
            Auth::Admin,
            Some(
                json!({"path": "default", "softQuota": {"type": "spaceUsedQuota", "limit": 10000}}),
            ),
        )
        .await;
    assert_eq!(quota.status, StatusCode::NO_CONTENT, "{}", quota.text());
    for offset in 1..=8_i64 {
        execute(
            &fixture,
            "INSERT INTO blob_store_usage_day (store, day, blob_count, total_bytes) \
             VALUES ('default', current_date - $1::int, 0, $2)",
            vec![(9 - offset as i32).into(), ((offset - 1) * 1000).into()],
        )
        .await;
    }

    let off = space_alert_with(&fixture, Method::POST, "1", Some("0")).await;
    assert_eq!(off.status, StatusCode::CREATED, "{}", off.text());
    let (outcome, sent) = checked(&fixture, &receiver).await;
    assert!(outcome.alerted.is_empty() && sent.is_empty());

    let on = space_alert_with(&fixture, Method::PUT, "1", Some("14")).await;
    assert_eq!(on.status, StatusCode::OK, "{}", on.text());
    let (outcome, sent) = checked(&fixture, &receiver).await;
    assert_eq!(outcome.alerted.len(), 1);
    assert_eq!(outcome.alerted[0].store, "default");
    assert!(matches!(
        outcome.alerted[0].problem,
        Problem::Forecast { days: 10 }
    ));
    assert_eq!(sent.len(), 1);
    let text = sent[0]["text"].as_str().unwrap();
    assert!(
        text.contains("• `default`: will be full in 10 days at the current growth"),
        "{text}"
    );
    let (_, sent) = checked(&fixture, &receiver).await;
    assert!(sent.is_empty());

    execute(
        &fixture,
        "UPDATE blob_store_usage_day SET total_bytes = 1000",
        vec![],
    )
    .await;
    let (outcome, sent) = checked(&fixture, &receiver).await;
    assert!(outcome.alerted.is_empty());
    assert_eq!(outcome.recovered.len(), 1);
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0]["event"], "blobstore.recovered");
    assert!(
        sent[0]["text"]
            .as_str()
            .unwrap()
            .ends_with("is no longer forecast to fill up soon")
    );

    let too_far = space_alert_with(&fixture, Method::PUT, "1", Some("366")).await;
    assert_eq!(
        too_far.status,
        StatusCode::BAD_REQUEST,
        "{}",
        too_far.text()
    );
    assert!(
        too_far
            .text()
            .contains("Forecast alert must be at most 365")
    );
}

#[tokio::test]
async fn repository_bytes_and_quota_come_from_the_usage_counter() {
    let fixture = ready().await;
    let created = capability(
        &fixture,
        json!({"type": "metrics.prometheus", "enabled": true, "properties": {"repositories": "true"}}),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    hosted(&fixture, "files").await;
    let capped = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(json!({
                "name": "capped",
                "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW", "quota": {"limitBytes": 100, "enforce": true}},
                "raw": {"contentDisposition": "ATTACHMENT"}
            })),
        )
        .await;
    assert_eq!(capped.status, StatusCode::CREATED, "{}", capped.text());
    let uploaded = fixture
        .send_body(
            Method::PUT,
            "/repository/capped/a.txt",
            Auth::Admin,
            Body::from("hello"),
            &[],
        )
        .await;
    assert_eq!(uploaded.status, StatusCode::CREATED);

    let text = fixture
        .call(Method::GET, METRICS, Auth::Admin, None)
        .await
        .text();
    for expected in [
        "astrum_repository_bytes{repository=\"capped\"} 5",
        "astrum_repository_bytes{repository=\"files\"} 0",
        "astrum_repository_quota_bytes{repository=\"capped\"} 100",
    ] {
        assert!(
            text.contains(expected),
            "{expected} is missing from\n{text}"
        );
    }
    assert!(line(&text, "astrum_repository_quota_bytes{repository=\"files\"}").is_none());
}
