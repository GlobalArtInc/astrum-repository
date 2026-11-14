mod support;

use std::sync::Arc;

use astrum_api::customer_expiry::CustomerTokenExpiry;
use astrum_db::setting;
use astrum_db::system::{CUSTOMER_EXPIRY_NOTICE_KEY, CustomerExpiryNotice};
use astrum_security::catalogue::ADMIN_USER;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use astrum_security::{NewToken, Scope};
use axum::http::{Method, StatusCode};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use support::{Auth, Fixture};
use uuid::Uuid;

const CAPABILITIES: &str = "/service/rest/v1/capabilities";
const CHANNELS: &str = "/service/rest/internal/ui/notifications";
const TASKS: &str = "/service/rest/v1/tasks";
const UI_TASKS: &str = "/service/rest/internal/ui/tasks";
const NOTICE: &str = "customers.expiryNotice";
const TASK_KIND: &str = "customers.tokens.expiry";

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .tasks()
        .register(Arc::new(CustomerTokenExpiry::new(
            fixture.state.notifier().clone(),
        )));
    astrum_tasks::bootstrap(fixture.state.db().conn())
        .await
        .unwrap();
    fixture
}

async fn allow_loopback(fixture: &Fixture) {
    setting::put(
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

async fn channel(fixture: &Fixture, name: &str, enabled: bool) {
    let reply = fixture
        .call(
            Method::POST,
            CHANNELS,
            Auth::Admin,
            Some(json!({
                "name": name,
                "type": "webhook",
                "secret": format!("http://127.0.0.1:9/{name}"),
                "events": ["published"],
                "enabled": enabled,
                "language": "en",
            })),
        )
        .await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
}

async fn notice(fixture: &Fixture, days: &str, channels: &str) -> support::Reply {
    fixture
        .call(
            Method::POST,
            CAPABILITIES,
            Auth::Admin,
            Some(json!({
                "type": NOTICE,
                "enabled": true,
                "properties": {"days": days, "channels": channels},
            })),
        )
        .await
}

async fn execute(fixture: &Fixture, sql: &str) {
    fixture
        .state
        .db()
        .conn()
        .execute(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap();
}

async fn set_expiry(fixture: &Fixture, id: Uuid, interval: &str) {
    execute(
        fixture,
        &format!(
            "UPDATE access_token SET expires_at = now() + interval '{interval}' WHERE id = '{id}'"
        ),
    )
    .await;
}

async fn expiry_notice(fixture: &Fixture, id: Uuid) -> Option<i16> {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT expiry_notice FROM access_token WHERE id = $1",
            [id.into()],
        ))
        .await
        .unwrap()
        .unwrap()
        .try_get::<Option<i16>>("", "expiry_notice")
        .unwrap()
}

async fn deliveries(fixture: &Fixture) -> Vec<(String, String)> {
    fixture
        .state
        .db()
        .conn()
        .query_all(Statement::from_string(
            DbBackend::Postgres,
            "SELECT event, text FROM notification_delivery ORDER BY written_at, event",
        ))
        .await
        .unwrap()
        .iter()
        .map(|row| {
            (
                row.try_get::<String>("", "event").unwrap(),
                row.try_get::<String>("", "text").unwrap(),
            )
        })
        .collect()
}

async fn task_id(fixture: &Fixture) -> String {
    let listed = fixture
        .call(Method::GET, TASKS, Auth::Admin, None)
        .await
        .json();
    listed["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|task| task["type"] == TASK_KIND)
        .unwrap()["id"]
        .as_str()
        .unwrap()
        .to_owned()
}

async fn run_task(fixture: &Fixture) -> Value {
    let id = task_id(fixture).await;
    let reply = fixture
        .call(
            Method::POST,
            &format!("{TASKS}/{id}/run"),
            Auth::Admin,
            None,
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
    fixture.state.tasks().run_pending().await.unwrap();
    let runs = fixture
        .call(
            Method::GET,
            &format!("{UI_TASKS}/{id}/runs"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(runs[0]["state"], "done", "{runs}");
    runs[0].clone()
}

#[tokio::test]
async fn each_threshold_is_announced_once() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    channel(&fixture, "Ops", true).await;
    let created = notice(&fixture, "30, 7, 1", "Ops").await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let (token, _) = fixture.customer_token("acme", &["files"]).await;

    set_expiry(&fixture, token, "29 days").await;
    assert_eq!(
        run_task(&fixture).await["message"],
        "1 expiring and 0 expired customer tokens announced"
    );
    let sent = deliveries(&fixture).await;
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].0, "customer.token.expiring");
    assert!(sent[0].1.contains("acme") && sent[0].1.contains("release"));
    assert_eq!(expiry_notice(&fixture, token).await, Some(30));
    run_task(&fixture).await;
    assert_eq!(deliveries(&fixture).await.len(), 1);

    set_expiry(&fixture, token, "6 days").await;
    run_task(&fixture).await;
    let sent = deliveries(&fixture).await;
    assert_eq!(sent.len(), 2);
    assert_eq!(sent[1].0, "customer.token.expiring");
    assert_eq!(expiry_notice(&fixture, token).await, Some(7));

    set_expiry(&fixture, token, "-1 hour").await;
    run_task(&fixture).await;
    let sent = deliveries(&fixture).await;
    assert_eq!(sent.len(), 3);
    assert_eq!(sent[2].0, "customer.token.expired");
    assert_eq!(expiry_notice(&fixture, token).await, Some(0));
    run_task(&fixture).await;
    assert_eq!(deliveries(&fixture).await.len(), 3);
}

#[tokio::test]
async fn a_token_expired_long_ago_is_not_announced() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    channel(&fixture, "Ops", true).await;
    assert_eq!(
        notice(&fixture, "30, 7, 1", "Ops").await.status,
        StatusCode::CREATED
    );
    let (token, _) = fixture.customer_token("acme", &["files"]).await;
    set_expiry(&fixture, token, "-30 days").await;
    run_task(&fixture).await;
    assert!(deliveries(&fixture).await.is_empty());
    assert_eq!(expiry_notice(&fixture, token).await, None);
}

#[tokio::test]
async fn nothing_is_sent_or_marked_without_an_enabled_channel_of_the_listed_name() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    channel(&fixture, "Quiet", false).await;
    let (token, _) = fixture.customer_token("acme", &["files"]).await;
    set_expiry(&fixture, token, "29 days").await;

    assert_eq!(run_task(&fixture).await["message"], "no channels are set");
    assert!(deliveries(&fixture).await.is_empty());
    assert_eq!(expiry_notice(&fixture, token).await, None);

    assert_eq!(
        notice(&fixture, "30, 7, 1", "Quiet").await.status,
        StatusCode::CREATED
    );
    assert_eq!(run_task(&fixture).await["message"], "no channels are set");
    assert!(deliveries(&fixture).await.is_empty());
    assert_eq!(expiry_notice(&fixture, token).await, None);

    let stored = CustomerExpiryNotice {
        channels: vec!["Gone".to_owned()],
        ..CustomerExpiryNotice::default()
    };
    setting::put(
        fixture.state.db().conn(),
        CUSTOMER_EXPIRY_NOTICE_KEY,
        &stored,
    )
    .await
    .unwrap();
    assert_eq!(run_task(&fixture).await["message"], "no channels are set");
    assert!(deliveries(&fixture).await.is_empty());
    assert_eq!(expiry_notice(&fixture, token).await, None);
}

#[tokio::test]
async fn disabled_customers_and_personal_tokens_are_skipped() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    channel(&fixture, "Ops", true).await;
    assert_eq!(
        notice(&fixture, "30, 7, 1", "Ops").await.status,
        StatusCode::CREATED
    );
    let (customer_token, _) = fixture.customer_token("acme", &["files"]).await;
    set_expiry(&fixture, customer_token, "29 days").await;
    execute(
        &fixture,
        "UPDATE customer SET disabled = true WHERE id = 'acme'",
    )
    .await;
    let (personal, _) = fixture
        .state
        .security()
        .create_token(
            ADMIN_USER,
            NewToken::personal("mine", Scope::Read, Some(30)),
        )
        .await
        .unwrap();
    set_expiry(&fixture, personal.id, "29 days").await;

    run_task(&fixture).await;
    assert!(deliveries(&fixture).await.is_empty());
    assert_eq!(expiry_notice(&fixture, customer_token).await, None);
    assert_eq!(expiry_notice(&fixture, personal.id).await, None);
}

#[tokio::test]
async fn one_message_per_channel_lists_every_customer() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    channel(&fixture, "Ops", true).await;
    channel(&fixture, "Releases", true).await;
    assert_eq!(
        notice(&fixture, "30, 7, 1", "Ops, Releases").await.status,
        StatusCode::CREATED
    );
    let (acme, _) = fixture.customer_token("acme", &["files"]).await;
    let (beta, _) = fixture.customer_token("beta", &["files"]).await;
    set_expiry(&fixture, acme, "29 days").await;
    set_expiry(&fixture, beta, "3 days").await;

    run_task(&fixture).await;
    let sent = deliveries(&fixture).await;
    assert_eq!(sent.len(), 2, "{sent:?}");
    for (event, text) in &sent {
        assert_eq!(event, "customer.token.expiring");
        assert!(text.contains("acme") && text.contains("beta"), "{text}");
    }
    assert_eq!(expiry_notice(&fixture, acme).await, Some(30));
    assert_eq!(expiry_notice(&fixture, beta).await, Some(7));
}

#[tokio::test]
async fn bootstrap_creates_the_task_once_after_an_old_first_bootstrap() {
    let fixture = Fixture::new().await;
    let db = fixture.state.db().conn();
    setting::put(db, "tasks.provisioned", &true).await.unwrap();
    assert_eq!(astrum_tasks::bootstrap(db).await.unwrap(), 1);
    assert_eq!(astrum_tasks::bootstrap(db).await.unwrap(), 0);
}

#[tokio::test]
async fn the_notice_capability_keeps_one_to_five_distinct_day_counts() {
    let fixture = ready().await;
    allow_loopback(&fixture).await;
    channel(&fixture, "Ops", true).await;
    let types = fixture
        .call(
            Method::GET,
            &format!("{CAPABILITIES}/types"),
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert!(
        types
            .as_array()
            .unwrap()
            .iter()
            .any(|kind| kind["id"] == NOTICE)
    );

    let refused = notice(&fixture, "7, 30, 7", "Ops").await;
    assert_eq!(
        refused.status,
        StatusCode::BAD_REQUEST,
        "{}",
        refused.text()
    );
    assert!(refused.text().contains("only once"));

    let created = notice(&fixture, "30, 7", "Ops").await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    assert_eq!(created.json()["properties"]["days"], "30, 7");

    let updated = fixture
        .call(
            Method::PUT,
            &format!("{CAPABILITIES}/{NOTICE}"),
            Auth::Admin,
            Some(json!({"type": NOTICE, "enabled": true, "properties": {"days": "7, 30", "channels": "Ops"}})),
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    assert_eq!(updated.json()["properties"]["days"], "30, 7");
}
