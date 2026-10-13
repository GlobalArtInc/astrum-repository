mod support;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use astrum_db::system::{BASE_URL_KEY, BaseUrl};
use astrum_notify::{Mail, MailError, MailSender, Timing};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::Router;
use axum::body::{Body, Bytes};
use axum::extract::State;
use axum::http::{HeaderMap, Method, StatusCode};
use axum::response::IntoResponse;
use axum::routing::post;
use serde_json::{Value, json};
use support::{Auth, Fixture};

const CHANNELS: &str = "/service/rest/internal/ui/notifications";

#[derive(Clone, Default)]
struct Receiver {
    requests: Arc<Mutex<Vec<(HeaderMap, Bytes)>>>,
    failures: Arc<Mutex<Vec<StatusCode>>>,
}

impl Receiver {
    fn taken(&self) -> Vec<(HeaderMap, Value)> {
        self.requests
            .lock()
            .unwrap()
            .drain(..)
            .map(|(headers, body)| (headers, serde_json::from_slice(&body).unwrap()))
            .collect()
    }
}

async fn receive(
    State(receiver): State<Receiver>,
    headers: HeaderMap,
    body: Bytes,
) -> impl IntoResponse {
    let failure = {
        let mut failures = receiver.failures.lock().unwrap();
        (!failures.is_empty()).then(|| failures.remove(0))
    };
    if let Some(status) = failure {
        return (status, [("retry-after", "0")], "try later").into_response();
    }
    receiver.requests.lock().unwrap().push((headers, body));
    (StatusCode::OK, Json(json!({"ok": true}))).into_response()
}

use axum::Json;

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
        attempts: 4,
    });
    fixture
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

async fn upload(fixture: &Fixture, path: &str, body: &'static str) {
    let reply = fixture
        .send_body(Method::PUT, path, Auth::Admin, Body::from(body), &[])
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn create(fixture: &Fixture, body: Value) -> Value {
    let reply = fixture
        .call(Method::POST, CHANNELS, Auth::Admin, Some(body))
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()
}

fn hook(name: &str, url: &str) -> Value {
    json!({
        "name": name,
        "type": "webhook",
        "secret": url,
        "events": ["published", "updated", "deleted"],
        "signingSecret": "s3cret",
        "authorization": "Bearer abc",
    })
}

async fn deliveries(fixture: &Fixture) -> i64 {
    let (count,): (i64,) = sea_orm::sqlx::query_as("SELECT count(*) FROM notification_delivery")
        .fetch_one(fixture.state.db().conn().get_postgres_connection_pool())
        .await
        .unwrap();
    count
}

#[tokio::test]
async fn channels_are_managed_and_their_secrets_stay_hidden() {
    let fixture = ready().await;
    hosted(&fixture, "files").await;
    let anonymous = fixture.call(Method::GET, CHANNELS, Auth::None, None).await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    fixture.add_user("viewer", "viewer-pw", &[]).await;
    let forbidden = fixture
        .call(
            Method::POST,
            CHANNELS,
            Auth::Basic("viewer", "viewer-pw"),
            Some(hook("x", "https://hooks.example.com/a")),
        )
        .await;
    assert_eq!(forbidden.status, StatusCode::FORBIDDEN);

    let created = create(&fixture, {
        let mut body = hook("Releases", "https://hooks.example.com/a/b?token=secret");
        body["repositories"] = json!(["files"]);
        body
    })
    .await;
    assert_eq!(created["target"], "hooks.example.com");
    assert_eq!(created["hasSecret"], true);
    assert_eq!(created["hasSigningSecret"], true);
    assert_eq!(created["hasAuthorization"], true);
    assert!(!created.to_string().contains("token=secret"));
    assert!(!created.to_string().contains("s3cret"));
    let id = created["id"].as_str().unwrap().to_owned();

    let duplicate = fixture
        .call(
            Method::POST,
            CHANNELS,
            Auth::Admin,
            Some(hook("RELEASES", "https://hooks.example.com/c")),
        )
        .await;
    assert_eq!(duplicate.status, StatusCode::BAD_REQUEST);
    assert_eq!(duplicate.json()[0]["id"], "name");
    let unknown = fixture
        .call(
            Method::POST,
            CHANNELS,
            Auth::Admin,
            Some({
                let mut body = hook("Other", "https://hooks.example.com/c");
                body["repositories"] = json!(["missing"]);
                body
            }),
        )
        .await;
    assert_eq!(unknown.status, StatusCode::BAD_REQUEST);
    assert_eq!(unknown.json()[0]["id"], "repositories");

    let moved = fixture
        .call(
            Method::PUT,
            &format!("{CHANNELS}/{id}"),
            Auth::Admin,
            Some(json!({"name": "Releases", "type": "webhook", "secret": "https://elsewhere.example.com/x", "events": ["published"]})),
        )
        .await;
    assert_eq!(moved.status, StatusCode::BAD_REQUEST);
    assert_eq!(moved.json()[0]["id"], "authorization");
    let kept = fixture
        .call(
            Method::PUT,
            &format!("{CHANNELS}/{id}"),
            Auth::Admin,
            Some(json!({"name": "Releases", "type": "webhook", "events": ["published"], "enabled": false, "signingSecret": ""})),
        )
        .await;
    assert_eq!(kept.status, StatusCode::OK, "{}", kept.text());
    let kept = kept.json();
    assert_eq!(kept["hasSecret"], true);
    assert_eq!(kept["hasSigningSecret"], false);
    assert_eq!(kept["hasAuthorization"], true);
    assert_eq!(kept["enabled"], false);
    assert_eq!(kept["events"], json!(["published"]));

    let listed = fixture.call(Method::GET, CHANNELS, Auth::Admin, None).await;
    assert_eq!(listed.json().as_array().unwrap().len(), 1);
    let deleted = fixture
        .call(
            Method::DELETE,
            &format!("{CHANNELS}/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(deleted.status, StatusCode::NO_CONTENT);
    let again = fixture
        .call(
            Method::DELETE,
            &format!("{CHANNELS}/{id}"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(again.status, StatusCode::NOT_FOUND);

    let audit = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit?domain=system.notifications",
            Auth::Admin,
            None,
        )
        .await
        .json();
    let mut kinds: Vec<_> = audit
        .as_array()
        .unwrap()
        .iter()
        .map(|event| event["type"].as_str().unwrap().to_owned())
        .collect();
    kinds.sort();
    assert_eq!(kinds, ["created", "deleted", "updated"], "{audit}");
    assert!(!audit.to_string().contains("s3cret"));
}

#[tokio::test]
async fn component_changes_reach_a_signed_webhook() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    hosted(&fixture, "files").await;
    astrum_db::setting::put(
        fixture.state.db().conn(),
        BASE_URL_KEY,
        &BaseUrl {
            url: "https://repo.example.com/".into(),
            ..BaseUrl::default()
        },
    )
    .await
    .unwrap();
    let (url, receiver) = start_receiver().await;
    create(&fixture, hook("Hook", &format!("{url}/hook"))).await;

    upload(&fixture, "/repository/files/docs/a.txt", "one").await;
    upload(&fixture, "/repository/files/docs/a.txt", "two").await;
    fixture.state.notifier().flush_now().await.unwrap();
    let mut received = receiver.taken();
    assert_eq!(received.len(), 1);
    let (headers, body) = received.remove(0);
    assert_eq!(headers["x-astrum-event"], "component.published");
    assert_eq!(headers["authorization"], "Bearer abc");
    assert_eq!(body["event"], "component.published");
    assert_eq!(
        body["repository"],
        json!({"name": "files", "format": "raw", "type": "hosted"})
    );
    assert_eq!(body["component"]["coordinates"], body["component"]["name"]);
    assert_eq!(body["initiator"], "admin");
    assert!(
        body["url"]
            .as_str()
            .unwrap()
            .starts_with("https://repo.example.com/search/component/raw%3A")
    );
    assert_eq!(
        headers["x-astrum-delivery"],
        body["deliveryId"].as_str().unwrap()
    );
    let raw = serde_json::to_vec(&body).unwrap();
    assert_eq!(
        headers["x-astrum-signature"].to_str().unwrap(),
        astrum_notify::payload::signature("s3cret", &raw)
    );

    let removed = fixture
        .call(
            Method::DELETE,
            "/repository/files/docs/a.txt",
            Auth::Admin,
            None,
        )
        .await;
    assert!(removed.status.is_success(), "{}", removed.status);
    fixture.state.notifier().flush_now().await.unwrap();
    let received = receiver.taken();
    assert_eq!(received.len(), 1);
    assert_eq!(received[0].1["event"], "component.deleted");
    assert_eq!(received[0].1["url"], Value::Null);
    assert_eq!(deliveries(&fixture).await, 0);
}

#[tokio::test]
async fn failed_deliveries_are_retried_and_wait_for_the_lock() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    hosted(&fixture, "files").await;
    let (url, receiver) = start_receiver().await;
    create(&fixture, hook("Hook", &format!("{url}/hook"))).await;
    receiver
        .failures
        .lock()
        .unwrap()
        .push(StatusCode::SERVICE_UNAVAILABLE);

    let pool = fixture
        .state
        .db()
        .conn()
        .get_postgres_connection_pool()
        .clone();
    let mut other = pool.acquire().await.unwrap();
    let key = i64::from_be_bytes(*b"astrntfy");
    sea_orm::sqlx::query("SELECT pg_advisory_lock($1)")
        .bind(key)
        .execute(&mut *other)
        .await
        .unwrap();
    upload(&fixture, "/repository/files/b.txt", "b").await;
    fixture.state.notifier().tick().await.unwrap();
    assert_eq!(deliveries(&fixture).await, 0);
    assert!(receiver.taken().is_empty());
    sea_orm::sqlx::query("SELECT pg_advisory_unlock($1)")
        .bind(key)
        .execute(&mut *other)
        .await
        .unwrap();

    fixture.state.notifier().tick().await.unwrap();
    assert!(receiver.taken().is_empty());
    assert_eq!(deliveries(&fixture).await, 1);
    fixture.state.notifier().tick().await.unwrap();
    assert_eq!(receiver.taken().len(), 1);
    assert_eq!(deliveries(&fixture).await, 0);
}

#[tokio::test]
async fn webhooks_respect_the_ssrf_protection() {
    let fixture = ready().await;
    let (url, receiver) = start_receiver().await;
    let reply = fixture
        .call(
            Method::POST,
            &format!("{CHANNELS}/test"),
            Auth::Admin,
            Some(json!({"type": "slack", "secret": format!("{url}/hook")})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK);
    let result = reply.json();
    assert_eq!(result["success"], false);
    assert!(
        result["message"].as_str().unwrap().contains("SSRF"),
        "{result}"
    );
    assert!(!result["message"].as_str().unwrap().contains("/hook"));
    assert!(receiver.taken().is_empty());

    allow_loopback(&fixture).await;
    let reply = fixture
        .call(
            Method::POST,
            &format!("{CHANNELS}/test"),
            Auth::Admin,
            Some(json!({"type": "slack", "secret": format!("{url}/hook"), "language": "en", "name": "Team"})),
        )
        .await
        .json();
    assert_eq!(reply["success"], true, "{reply}");
    let received = receiver.taken();
    assert!(
        received[0].1["text"]
            .as_str()
            .unwrap()
            .starts_with("✅ *Test message from Astrum Repository*\nNotifications of the channel “Team” are set up correctly.")
    );
}

#[tokio::test]
async fn telegram_tests_go_to_the_bot_api() {
    let fixture = ready().await;
    let (url, receiver) = start_receiver().await;
    fixture.state.notifier().set_telegram_api(url);
    let created = create(
        &fixture,
        json!({
            "name": "Chat",
            "type": "telegram",
            "secret": "123456:ABC-def",
            "chatId": "-1001234567890",
            "messageThreadId": 7,
            "events": ["published"],
        }),
    )
    .await;
    assert_eq!(created["target"], "-1001234567890");
    let reply = fixture
        .call(
            Method::POST,
            &format!("{CHANNELS}/{}/test", created["id"].as_str().unwrap()),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(reply["success"], true, "{reply}");
    let received = receiver.taken();
    assert_eq!(received[0].1["chat_id"], "-1001234567890");
    assert_eq!(received[0].1["message_thread_id"], 7);
    assert_eq!(received[0].1["parse_mode"], "HTML");
    assert!(
        received[0].1["text"]
            .as_str()
            .unwrap()
            .starts_with("✅ <b>Тестовое сообщение от Astrum Repository</b>")
    );
}

struct Outbox(Mutex<Vec<Mail>>);

#[async_trait::async_trait]
impl MailSender for Outbox {
    async fn send(&self, mail: Mail) -> Result<(), MailError> {
        self.0.lock().unwrap().push(mail);
        Ok(())
    }
}

#[tokio::test]
async fn email_channels_use_the_mail_sender() {
    let fixture = ready().await;
    hosted(&fixture, "files").await;
    let body = json!({
        "name": "Mail",
        "type": "email",
        "recipients": ["team@example.com"],
        "events": ["published"],
        "language": "en",
    });
    let refused = fixture
        .call(
            Method::POST,
            &format!("{CHANNELS}/test"),
            Auth::Admin,
            Some(body.clone()),
        )
        .await
        .json();
    assert_eq!(refused["success"], false);
    assert!(
        refused["message"]
            .as_str()
            .unwrap()
            .contains("Email is not set up")
    );

    let outbox = Arc::new(Outbox(Mutex::new(Vec::new())));
    fixture.state.notifier().set_mail_sender(outbox.clone());
    create(&fixture, body).await;
    upload(&fixture, "/repository/files/c.txt", "c").await;
    fixture.state.notifier().flush_now().await.unwrap();
    let mails = outbox.0.lock().unwrap().clone();
    assert_eq!(mails.len(), 1);
    assert_eq!(mails[0].to, ["team@example.com"]);
    assert_eq!(mails[0].subject, "Component published: /c.txt");
    assert!(
        mails[0].text.contains("Repository: files · raw"),
        "{}",
        mails[0].text
    );
}
