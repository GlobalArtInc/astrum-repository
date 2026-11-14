mod accounts;
mod support;

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use astrum_content::transfer::fingerprint;
use astrum_content::{Format, store};
use astrum_replication::dispatch::{Dispatcher, Lock, Pass, Settings};
use astrum_replication::queue;
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::Body;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, Method, StatusCode, header};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

const REMOTES: &str = "/service/rest/v1/replication/remotes";
const REPLICATIONS: &str = "/service/rest/v1/replications";
const LEASE: Duration = Duration::from_secs(600);

#[derive(Default)]
struct Plan {
    status: u16,
    left: usize,
    retry_after: Option<&'static str>,
    commits: usize,
}

#[derive(Clone, Default)]
struct Force(Arc<Mutex<Plan>>);

impl Force {
    fn answer(&self, status: u16, times: usize, retry_after: Option<&'static str>) {
        *self.0.lock().unwrap() = Plan {
            status,
            left: times,
            retry_after,
            commits: self.commits(),
        };
    }

    fn commits(&self) -> usize {
        self.0.lock().unwrap().commits
    }
}

async fn forced(State(force): State<Force>, request: Request, next: Next) -> Response {
    let refused = {
        let mut plan = force.0.lock().unwrap();
        match plan.left > 0 {
            true => {
                plan.left -= 1;
                Some((plan.status, plan.retry_after))
            }
            false => {
                if request.method() == Method::POST && request.uri().path().ends_with("/components")
                {
                    plan.commits += 1;
                }
                None
            }
        }
    };
    let Some((status, retry_after)) = refused else {
        return next.run(request).await;
    };
    let body = json!({"code": "forced", "message": "forced by the test"}).to_string();
    let mut response = (
        StatusCode::from_u16(status).unwrap(),
        [(header::CONTENT_TYPE, "application/json")],
        body,
    )
        .into_response();
    if let Some(seconds) = retry_after {
        response
            .headers_mut()
            .insert(header::RETRY_AFTER, HeaderValue::from_static(seconds));
    }
    response
}

struct Node {
    fixture: Fixture,
    address: SocketAddr,
    token: String,
    force: Force,
}

impl Node {
    async fn start(repositories: &[&str]) -> Self {
        let fixture = Fixture::new().await;
        astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
            .await
            .unwrap();
        fixture
            .state
            .content()
            .register(Arc::new(astrum_format_raw::Raw));
        fixture
            .state
            .content()
            .register(Arc::new(astrum_format_maven::Maven));
        for name in repositories {
            let body = json!({
                "name": name, "online": true,
                "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
                "raw": {"contentDisposition": "INLINE"}
            });
            let path = "/service/rest/v1/repositories/raw/hosted";
            let made = fixture
                .call(Method::POST, path, Auth::Admin, Some(body))
                .await;
            assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
        }
        astrum_db::setting::put(
            fixture.state.db().conn(),
            SSRF_KEY,
            &SsrfProtection {
                enabled: true,
                allowed_ips: vec!["127.0.0.1".to_owned()],
                allowed_domains: Vec::new(),
            },
        )
        .await
        .unwrap();
        let tokens = "/service/rest/internal/access-tokens";
        let wanted = json!({"name": "replication", "scope": "WRITE", "expiresInDays": 1});
        let made = fixture
            .call(Method::POST, tokens, Auth::Admin, Some(wanted))
            .await;
        let token = made.json()["token"].as_str().unwrap().to_owned();
        let force = Force::default();
        let app = astrum_api::router(fixture.state.clone())
            .layer(middleware::from_fn_with_state(force.clone(), forced))
            .into_make_service_with_connect_info::<SocketAddr>();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        Node {
            fixture,
            address,
            token,
            force,
        }
    }

    fn dispatcher(&self) -> Dispatcher {
        Dispatcher::new(self.fixture.state.content().clone())
    }

    async fn pass(&self) -> Pass {
        self.dispatcher().run_once().await.unwrap()
    }

    async fn sql(&self, sql: &str) {
        let db = self.fixture.state.db().conn();
        db.execute(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap();
    }

    async fn number(&self, sql: &str) -> i64 {
        let db = self.fixture.state.db().conn();
        let row = db
            .query_one(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap()
            .unwrap();
        row.try_get("", "n").unwrap()
    }

    async fn text(&self, sql: &str) -> Option<String> {
        let db = self.fixture.state.db().conn();
        let row = db
            .query_one(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap()?;
        row.try_get("", "t").unwrap()
    }

    async fn put(&self, repository: &str, path: &str, bytes: &[u8]) {
        let uri = format!("/repository/{repository}{path}");
        let body = Body::from(bytes.to_vec());
        let sent = self
            .fixture
            .send_body(Method::PUT, &uri, Auth::Admin, body, &[])
            .await;
        assert!(sent.status.is_success(), "{}", sent.text());
    }

    async fn get(&self, repository: &str, path: &str) -> Reply {
        let uri = format!("/repository/{repository}{path}");
        self.fixture
            .call(Method::GET, &uri, Auth::Admin, None)
            .await
    }

    async fn component(&self, repository: &str, path: &str) -> i64 {
        let sql = format!(
            "SELECT id AS n FROM component WHERE repository = '{repository}' AND name = '{path}'"
        );
        self.number(&sql).await
    }

    async fn delete(&self, repository: &str, path: &str) {
        let id = self.component(repository, path).await;
        let content = self.fixture.state.content();
        assert!(store::delete_component(content, id).await.unwrap());
    }

    async fn instance(&self) -> String {
        self.dispatcher().instance().await.unwrap().to_string()
    }

    async fn state(&self) -> Option<(String, i32, i64)> {
        let db = self.fixture.state.db().conn();
        let row = db
            .query_one(Statement::from_string(
                DbBackend::Postgres,
                "SELECT state, attempts, generation FROM replication_item ORDER BY id LIMIT 1",
            ))
            .await
            .unwrap()?;
        Some((
            row.try_get("", "state").unwrap(),
            row.try_get("", "attempts").unwrap(),
            row.try_get("", "generation").unwrap(),
        ))
    }

    async fn insert(&self, action: &str, name: &str) {
        let txn = self.fixture.state.db().conn().begin().await.unwrap();
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT set_config('astrum.action', $1, true)",
            [action.into()],
        ))
        .await
        .unwrap();
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "INSERT INTO component (repository, format, namespace, name, version) \
             VALUES ('watched', 'raw', '/d', $1, '')",
            [name.into()],
        ))
        .await
        .unwrap();
        txn.commit().await.unwrap();
    }

    async fn due(&self) {
        self.sql("UPDATE replication_item SET not_before = now()")
            .await;
    }

    async fn bundle_fingerprint(
        &self,
        format: &dyn Format,
        repository: &str,
        path: &str,
    ) -> String {
        fingerprint(&support::bundle_of(&self.fixture, format, repository, path).await)
    }

    async fn maven(&self, name: &str) {
        let body = json!({
            "name": name, "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
            "maven": {"versionPolicy": "RELEASE", "layoutPolicy": "STRICT"}
        });
        let path = "/service/rest/v1/repositories/maven/hosted";
        let made = self
            .fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    }
}

async fn connect(from: &Node, to: &Node) -> String {
    let body = json!({
        "name": format!("remote-{}", Uuid::new_v4().simple()),
        "url": format!("http://{}", to.address),
        "token": to.token,
        "allowPrivateNetwork": true,
    });
    let made = from
        .fixture
        .call(Method::POST, REMOTES, Auth::Admin, Some(body))
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    made.json()["id"].as_str().unwrap().to_owned()
}

async fn replicate(from: &Node, remote: &str, source: &str, target: &str, extra: Value) -> String {
    let mut body = json!({
        "name": format!("replication-{}", Uuid::new_v4().simple()),
        "sourceRepository": source,
        "remoteId": remote,
        "remoteRepository": target,
        "settleSeconds": 0,
    });
    for (key, value) in extra.as_object().unwrap() {
        body[key] = value.clone();
    }
    let made = from
        .fixture
        .call(Method::POST, REPLICATIONS, Auth::Admin, Some(body))
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    made.json()["id"].as_str().unwrap().to_owned()
}

async fn pair() -> (Node, Node) {
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["mirror"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(&sender, &remote, "source", "mirror", json!({})).await;
    (sender, receiver)
}

async fn wait_for(mut done: impl AsyncFnMut() -> bool) {
    for _ in 0..100 {
        if done().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the condition did not come true in ten seconds");
}

#[tokio::test]
async fn a_pushed_component_arrives_with_its_bytes_origin_and_fingerprint() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    sender.put("source", "/docs/b.txt", b"beta").await;
    let pass = sender.pass().await;
    assert_eq!((pass.leased, pass.sent), (2, 2), "{pass:?}");
    assert_eq!(sender.state().await, None);
    for (path, bytes) in [("/docs/a.txt", &b"alpha"[..]), ("/docs/b.txt", b"beta")] {
        let arrived = receiver.get("mirror", path).await;
        assert_eq!(arrived.status, StatusCode::OK);
        assert_eq!(arrived.body, bytes);
        assert_eq!(
            receiver
                .bundle_fingerprint(&astrum_format_raw::Raw, "mirror", path)
                .await,
            sender
                .bundle_fingerprint(&astrum_format_raw::Raw, "source", path)
                .await
        );
    }
    let origin = "SELECT attributes -> 'replication' ->> 'origin' AS t FROM component \
                  WHERE repository = 'mirror' AND name = '/docs/a.txt'";
    assert_eq!(receiver.text(origin).await, Some(sender.instance().await));
    let stamped = "SELECT count(*) AS n FROM replication WHERE last_success_at IS NOT NULL";
    assert_eq!(sender.number(stamped).await, 1);
    assert_eq!(sender.pass().await, Pass::default());
}

#[tokio::test]
async fn a_maven_component_arrives_whole_with_the_same_fingerprint() {
    let sender = Node::start(&[]).await;
    let receiver = Node::start(&[]).await;
    sender.maven("releases").await;
    receiver.maven("mirror").await;
    let remote = connect(&sender, &receiver).await;
    replicate(&sender, &remote, "releases", "mirror", json!({})).await;
    let pom = "<project><modelVersion>4.0.0</modelVersion><groupId>com.acme</groupId>\
               <artifactId>lib</artifactId><version>1.0</version></project>";
    let jar = "/com/acme/lib/1.0/lib-1.0.jar";
    sender.put("releases", jar, b"jar bytes").await;
    sender
        .put("releases", "/com/acme/lib/1.0/lib-1.0.pom", pom.as_bytes())
        .await;
    let pass = sender.pass().await;
    assert_eq!((pass.sent, pass.failed), (1, 0), "{pass:?}");
    for (path, bytes) in [(jar, "jar bytes"), ("/com/acme/lib/1.0/lib-1.0.pom", pom)] {
        let arrived = receiver.get("mirror", path).await;
        assert_eq!(arrived.status, StatusCode::OK, "{path}");
        assert_eq!(arrived.body, bytes.as_bytes());
    }
    assert_eq!(
        receiver
            .bundle_fingerprint(&astrum_format_maven::Maven, "mirror", jar)
            .await,
        sender
            .bundle_fingerprint(&astrum_format_maven::Maven, "releases", jar)
            .await
    );
}

#[tokio::test]
async fn a_burst_of_events_on_one_key_gives_one_transfer() {
    let (sender, receiver) = pair().await;
    for round in 0..50 {
        sender
            .put(
                "source",
                "/docs/a.txt",
                format!("version {round}").as_bytes(),
            )
            .await;
    }
    let (state, _, generation) = sender.state().await.unwrap();
    assert_eq!(state, "pending");
    assert!(generation >= 50, "{generation}");
    let pass = sender.pass().await;
    assert_eq!((pass.leased, pass.sent), (1, 1), "{pass:?}");
    assert_eq!(receiver.force.commits(), 1);
    assert_eq!(
        receiver.get("mirror", "/docs/a.txt").await.body,
        b"version 49"
    );
}

#[tokio::test]
async fn an_event_while_the_item_is_leased_is_sent_again_after_the_first_finishes() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"one").await;
    let db = sender.fixture.state.db().conn();
    let remote = "SELECT remote_id::text AS t FROM replication";
    let remote = Uuid::parse_str(&sender.text(remote).await.unwrap()).unwrap();
    let mut leased = queue::lease(db, remote, LEASE, 5).await.unwrap();
    let item = leased.pop().unwrap();
    assert!(leased.is_empty());
    sender.put("source", "/docs/a.txt", b"two").await;
    let (state, _, generation) = sender.state().await.unwrap();
    assert_eq!(state, "leased");
    assert!(generation > item.generation);
    assert_eq!(
        queue::finish(db, &item).await.unwrap(),
        queue::Finished::Again
    );
    assert_eq!(sender.state().await.unwrap().0, "pending");
    assert_eq!(sender.pass().await.sent, 1);
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"two");
    assert_eq!(sender.state().await, None);
}

#[tokio::test]
async fn a_temporary_failure_is_retried_with_backoff_and_the_hint_of_the_remote() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver.force.answer(503, 1, Some("120"));
    let pass = sender.pass().await;
    assert_eq!((pass.failed, pass.dead, pass.sent), (1, 0, 0), "{pass:?}");
    let (state, attempts, _) = sender.state().await.unwrap();
    assert_eq!((state.as_str(), attempts), ("pending", 1));
    let delay =
        "SELECT (extract(epoch FROM (not_before - now())))::bigint AS n FROM replication_item";
    assert!(sender.number(delay).await >= 100);
    let error = "SELECT last_error AS t FROM replication_item";
    assert!(sender.text(error).await.unwrap().contains("503"));
    assert_eq!(sender.pass().await, Pass::default());
    assert_eq!(
        receiver.get("mirror", "/docs/a.txt").await.status,
        StatusCode::NOT_FOUND
    );
    sender.due().await;
    assert_eq!(sender.pass().await.sent, 1);
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");
    assert_eq!(sender.state().await, None);
    let failed = "SELECT count(*) AS n FROM replication WHERE last_failure_at IS NOT NULL AND last_error LIKE '%503%'";
    assert_eq!(sender.number(failed).await, 1);
}

#[tokio::test]
async fn a_permanent_refusal_kills_the_item_at_once_and_a_new_event_revives_it() {
    let (sender, receiver) = pair().await;
    for (index, status) in [409_u16, 403].into_iter().enumerate() {
        let path = format!("/docs/{index}.txt");
        sender.put("source", &path, b"alpha").await;
        receiver.force.answer(status, 1, None);
        let pass = sender.pass().await;
        assert_eq!((pass.failed, pass.dead), (1, 1), "{status}: {pass:?}");
        let (state, attempts, _) = sender.state().await.unwrap();
        assert_eq!((state.as_str(), attempts), ("dead", 1));
        let error = "SELECT last_error AS t FROM replication_item";
        let text = sender.text(error).await.unwrap();
        assert!(
            text.contains(&status.to_string()) && text.contains("forced"),
            "{text}"
        );
        assert_eq!(sender.pass().await, Pass::default());
        sender.put("source", &path, b"beta").await;
        assert_eq!(sender.state().await.unwrap().0, "pending");
        assert_eq!(sender.pass().await.sent, 1);
        assert_eq!(receiver.get("mirror", &path).await.body, b"beta");
    }
}

#[tokio::test]
async fn twelve_temporary_failures_make_an_item_dead() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    receiver.force.answer(503, usize::MAX, None);
    for attempt in 1..=12 {
        sender.due().await;
        let pass = sender.pass().await;
        assert_eq!(pass.failed, 1, "{attempt}: {pass:?}");
        let (state, attempts, _) = sender.state().await.unwrap();
        assert_eq!(attempts, attempt);
        assert_eq!(state, if attempt == 12 { "dead" } else { "pending" });
    }
    sender.due().await;
    assert_eq!(sender.pass().await, Pass::default());
}

#[tokio::test]
async fn an_expired_lease_returns_to_pending_and_is_taken_again() {
    let (sender, receiver) = pair().await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    let db = sender.fixture.state.db().conn();
    let remote = "SELECT remote_id::text AS t FROM replication";
    let remote = Uuid::parse_str(&sender.text(remote).await.unwrap()).unwrap();
    assert_eq!(queue::lease(db, remote, LEASE, 5).await.unwrap().len(), 1);
    assert_eq!(sender.pass().await, Pass::default());
    sender
        .sql("UPDATE replication_item SET leased_until = now() - interval '1 minute'")
        .await;
    let pass = sender.pass().await;
    assert_eq!((pass.expired, pass.sent), (1, 1), "{pass:?}");
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");
}

#[tokio::test]
async fn only_the_node_that_holds_the_lock_dispatches() {
    let (sender, receiver) = pair().await;
    let content = sender.fixture.state.content().clone();
    let holder = Lock::try_take(&content).await.unwrap().unwrap();
    assert!(Lock::try_take(&content).await.unwrap().is_none());
    let quick = Settings {
        tick: Duration::from_millis(50),
        idle: Duration::from_millis(50),
        ..Settings::default()
    };
    let stop = CancellationToken::new();
    let second =
        tokio::spawn(Dispatcher::with_settings(content.clone(), quick).serve(stop.clone()));
    sender.put("source", "/docs/a.txt", b"alpha").await;
    tokio::time::sleep(Duration::from_millis(600)).await;
    assert_eq!(sender.state().await.unwrap().0, "pending");
    assert_eq!(
        receiver.get("mirror", "/docs/a.txt").await.status,
        StatusCode::NOT_FOUND
    );
    holder.release().await.unwrap();
    wait_for(async || sender.state().await.is_none()).await;
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"alpha");
    stop.cancel();
    second.await.unwrap();
    assert!(Lock::try_take(&content).await.unwrap().is_some());
}

#[tokio::test]
async fn a_bidirectional_pair_does_not_send_a_component_back_to_its_origin() {
    let first = Node::start(&["left"]).await;
    let second = Node::start(&["right"]).await;
    let forth = connect(&first, &second).await;
    let back = connect(&second, &first).await;
    replicate(&first, &forth, "left", "right", json!({})).await;
    replicate(&second, &back, "right", "left", json!({})).await;
    first.put("left", "/docs/a.txt", b"alpha").await;
    assert_eq!(first.pass().await.sent, 1);
    assert_eq!(second.get("right", "/docs/a.txt").await.body, b"alpha");
    assert_eq!(second.state().await, None);
    second
        .sql("SELECT replication_mark('right', '/docs', '/docs/a.txt', '')")
        .await;
    assert_eq!(second.state().await.unwrap().0, "pending");
    let pass = second.pass().await;
    assert_eq!(
        (pass.leased, pass.dropped, pass.sent),
        (1, 1, 0),
        "{pass:?}"
    );
    assert_eq!(second.state().await, None);
    assert_eq!(first.force.commits(), 0);
    let learned = "SELECT remote_instance_id::text AS t FROM replication_remote";
    assert_eq!(second.text(learned).await, Some(first.instance().await));
}

#[tokio::test]
async fn deletes_are_kept_by_default_and_travel_with_the_flag() {
    let sender = Node::start(&["keep", "mirror-on"]).await;
    let receiver = Node::start(&["kept", "mirrored"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(&sender, &remote, "keep", "kept", json!({})).await;
    replicate(
        &sender,
        &remote,
        "mirror-on",
        "mirrored",
        json!({"propagateDeletes": true}),
    )
    .await;
    for (source, target) in [("keep", "kept"), ("mirror-on", "mirrored")] {
        sender.put(source, "/docs/a.txt", b"alpha").await;
        assert_eq!(sender.pass().await.sent, 1);
        assert_eq!(
            receiver.get(target, "/docs/a.txt").await.status,
            StatusCode::OK
        );
        sender.delete(source, "/docs/a.txt").await;
        let pass = sender.pass().await;
        assert_eq!((pass.leased, pass.failed), (1, 0), "{source}: {pass:?}");
    }
    assert_eq!(
        receiver.get("kept", "/docs/a.txt").await.status,
        StatusCode::OK
    );
    assert_eq!(
        receiver.get("mirrored", "/docs/a.txt").await.status,
        StatusCode::NOT_FOUND
    );
    let recorded = "SELECT count(*) AS n FROM replication_sent";
    assert_eq!(sender.number(recorded).await, 0);
    assert_eq!(sender.state().await, None);
}

#[tokio::test]
async fn a_tombstone_for_a_republished_copy_is_refused_and_the_item_dies() {
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["mirror"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(
        &sender,
        &remote,
        "source",
        "mirror",
        json!({"propagateDeletes": true}),
    )
    .await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    assert_eq!(sender.pass().await.sent, 1);
    receiver.put("mirror", "/docs/a.txt", b"local").await;
    sender.delete("source", "/docs/a.txt").await;
    let pass = sender.pass().await;
    assert_eq!((pass.failed, pass.dead), (1, 1), "{pass:?}");
    let error = "SELECT last_error AS t FROM replication_item";
    assert!(sender.text(error).await.unwrap().contains("409"));
    assert_eq!(receiver.get("mirror", "/docs/a.txt").await.body, b"local");
}

#[tokio::test]
async fn components_outside_the_filters_and_disabled_replications_are_not_sent() {
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["by-name", "by-version", "off"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(
        &sender,
        &remote,
        "source",
        "by-name",
        json!({"nameFilter": ".*a\\.txt"}),
    )
    .await;
    replicate(
        &sender,
        &remote,
        "source",
        "by-version",
        json!({"versionFilter": "1\\..*"}),
    )
    .await;
    let off = replicate(&sender, &remote, "source", "off", json!({})).await;
    sender.put("source", "/docs/a.txt", b"alpha").await;
    sender.put("source", "/docs/b.txt", b"beta").await;
    let disabled = json!({
        "name": "off", "sourceRepository": "source", "remoteId": remote,
        "remoteRepository": "off", "enabled": false, "settleSeconds": 0
    });
    let updated = sender
        .fixture
        .call(
            Method::PUT,
            &format!("{REPLICATIONS}/{off}"),
            Auth::Admin,
            Some(disabled),
        )
        .await;
    assert_eq!(updated.status, StatusCode::OK, "{}", updated.text());
    let pass = sender.pass().await;
    assert_eq!(
        (pass.leased, pass.sent, pass.dropped),
        (6, 1, 5),
        "{pass:?}"
    );
    assert_eq!(
        receiver.get("by-name", "/docs/a.txt").await.status,
        StatusCode::OK
    );
    for (repository, path) in [
        ("by-name", "/docs/b.txt"),
        ("by-version", "/docs/a.txt"),
        ("off", "/docs/a.txt"),
    ] {
        let missing = receiver.get(repository, path).await;
        assert_eq!(missing.status, StatusCode::NOT_FOUND, "{repository}");
    }
    sender.put("source", "/docs/c.txt", b"gamma").await;
    let queued = "SELECT count(*) AS n FROM replication_item i JOIN replication r \
                  ON r.id = i.replication_id WHERE r.enabled = false";
    assert_eq!(sender.number(queued).await, 0);
}

#[tokio::test]
async fn the_capture_skips_replicated_writes_and_repositories_without_replications() {
    let node = Node::start(&["watched", "plain"]).await;
    let other = Node::start(&["mirror"]).await;
    let remote = connect(&node, &other).await;
    replicate(&node, &remote, "watched", "mirror", json!({})).await;
    replicate(&node, &remote, "plain", "mirror", json!({"enabled": false})).await;
    node.put("plain", "/docs/a.txt", b"alpha").await;
    assert_eq!(node.state().await, None);
    node.insert("replicated", "/d/r").await;
    node.insert("tiered", "/d/t").await;
    assert_eq!(node.state().await, None);
    node.insert("", "/d/l").await;
    let queued = "SELECT count(*) AS n FROM replication_item WHERE name = '/d/l'";
    assert_eq!(node.number(queued).await, 1);
    let all = "SELECT count(*) AS n FROM replication_item";
    assert_eq!(node.number(all).await, 1);
}
