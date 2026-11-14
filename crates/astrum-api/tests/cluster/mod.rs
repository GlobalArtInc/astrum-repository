#![allow(dead_code)]

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use astrum_content::transfer::fingerprint;
use astrum_content::{Format, store};
use astrum_replication::dispatch::{Dispatcher, Pass, Settings};
use astrum_security::ssrf::{SSRF_KEY, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use axum::middleware;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::support::{self, Auth, Fixture, Reply};

mod force;

#[allow(unused_imports)]
pub use force::{Fail, Force};

pub const REMOTES: &str = "/service/rest/v1/replication/remotes";
pub const REPLICATIONS: &str = "/service/rest/v1/replications";
pub const LEASE: Duration = Duration::from_secs(600);

pub struct Node {
    pub fixture: Fixture,
    pub address: SocketAddr,
    pub token: String,
    pub force: Force,
}

impl Node {
    pub async fn start(repositories: &[&str]) -> Self {
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
            .layer(middleware::from_fn_with_state(force.clone(), force::forced))
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

    pub fn dispatcher(&self) -> Dispatcher {
        self.dispatcher_with(Settings::default())
    }

    pub fn dispatcher_with(&self, settings: Settings) -> Dispatcher {
        Dispatcher::with_counters(
            self.fixture.state.content().clone(),
            settings,
            self.fixture.state.metrics().replication().clone(),
        )
    }

    pub async fn pass(&self) -> Pass {
        self.dispatcher().run_once().await.unwrap()
    }

    pub async fn sql(&self, sql: &str) {
        let db = self.fixture.state.db().conn();
        db.execute(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap();
    }

    pub async fn number(&self, sql: &str) -> i64 {
        let db = self.fixture.state.db().conn();
        let row = db
            .query_one(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap()
            .unwrap();
        row.try_get("", "n").unwrap()
    }

    pub async fn text(&self, sql: &str) -> Option<String> {
        let db = self.fixture.state.db().conn();
        let row = db
            .query_one(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap()?;
        row.try_get("", "t").unwrap()
    }

    pub async fn put(&self, repository: &str, path: &str, bytes: &[u8]) {
        let uri = format!("/repository/{repository}{path}");
        let body = Body::from(bytes.to_vec());
        let sent = self
            .fixture
            .send_body(Method::PUT, &uri, Auth::Admin, body, &[])
            .await;
        assert!(sent.status.is_success(), "{}", sent.text());
    }

    pub async fn get(&self, repository: &str, path: &str) -> Reply {
        let uri = format!("/repository/{repository}{path}");
        self.fixture
            .call(Method::GET, &uri, Auth::Admin, None)
            .await
    }

    pub async fn component(&self, repository: &str, path: &str) -> i64 {
        let sql = format!(
            "SELECT id AS n FROM component WHERE repository = '{repository}' AND name = '{path}'"
        );
        self.number(&sql).await
    }

    pub async fn delete(&self, repository: &str, path: &str) {
        let id = self.component(repository, path).await;
        let content = self.fixture.state.content();
        assert!(store::delete_component(content, id).await.unwrap());
    }

    pub async fn instance(&self) -> String {
        self.dispatcher().instance().await.unwrap().to_string()
    }

    pub async fn state(&self) -> Option<(String, i32, i64)> {
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

    pub async fn remote_id(&self) -> Uuid {
        let sql = "SELECT remote_id::text AS t FROM replication LIMIT 1";
        Uuid::parse_str(&self.text(sql).await.unwrap()).unwrap()
    }

    pub async fn insert(&self, repository: &str, action: &str, name: &str) {
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
             VALUES ($1, 'raw', '/d', $2, '')",
            [repository.into(), name.into()],
        ))
        .await
        .unwrap();
        txn.commit().await.unwrap();
    }

    pub async fn sql_as(&self, action: &str, sql: &str) {
        let txn = self.fixture.state.db().conn().begin().await.unwrap();
        txn.execute(Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT set_config('astrum.action', $1, true)",
            [action.into()],
        ))
        .await
        .unwrap();
        txn.execute(Statement::from_string(DbBackend::Postgres, sql))
            .await
            .unwrap();
        txn.commit().await.unwrap();
    }

    pub async fn due(&self) {
        self.sql("UPDATE replication_item SET not_before = now()")
            .await;
    }

    pub async fn bundle_fingerprint(
        &self,
        format: &dyn Format,
        repository: &str,
        path: &str,
    ) -> String {
        fingerprint(&support::bundle_of(&self.fixture, format, repository, path).await)
    }

    pub async fn maven(&self, name: &str) {
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

pub async fn connect(from: &Node, to: &Node) -> String {
    connect_with(from, to, json!({})).await
}

pub async fn connect_with(from: &Node, to: &Node, extra: Value) -> String {
    let mut body = json!({
        "name": format!("remote-{}", Uuid::new_v4().simple()),
        "url": format!("http://{}", to.address),
        "token": to.token,
        "allowPrivateNetwork": true,
    });
    for (key, value) in extra.as_object().unwrap() {
        body[key] = value.clone();
    }
    let made = from
        .fixture
        .call(Method::POST, REMOTES, Auth::Admin, Some(body))
        .await;
    assert_eq!(made.status, StatusCode::CREATED, "{}", made.text());
    made.json()["id"].as_str().unwrap().to_owned()
}

pub async fn replicate(
    from: &Node,
    remote: &str,
    source: &str,
    target: &str,
    extra: Value,
) -> String {
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

pub async fn pair() -> (Node, Node) {
    let sender = Node::start(&["source"]).await;
    let receiver = Node::start(&["mirror"]).await;
    let remote = connect(&sender, &receiver).await;
    replicate(&sender, &remote, "source", "mirror", json!({})).await;
    (sender, receiver)
}

pub async fn wait_for(mut done: impl AsyncFnMut() -> bool) {
    for _ in 0..150 {
        if done().await {
            return;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    panic!("the condition did not come true in fifteen seconds");
}
