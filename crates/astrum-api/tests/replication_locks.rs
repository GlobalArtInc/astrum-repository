mod support;

use std::net::SocketAddr;
use std::time::{Duration, Instant};

use astrum_api::router;
use astrum_content::transfer::{Bundle, fingerprint};
use astrum_outbound::Snapshot;
use astrum_replication::protocol::Tombstone;
use astrum_replication::{Client, Remote};
use astrum_security::SecretBox;
use astrum_security::ssrf::{Guard, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use bytes::Bytes;
use futures_util::stream;
use sea_orm::{ConnectionTrait, DbBackend, Statement, TransactionTrait};
use serde_json::json;
use sha2::{Digest as _, Sha256};
use support::{Auth, Fixture};

const SENDER: &str = "0b8e6f0a-0000-4000-8000-0000000000a1";
const WIDTH: usize = 8;

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn tombstone(path: &str, fingerprint: &str) -> Tombstone {
    let parent = &path[..path.rfind('/').unwrap()];
    Tombstone {
        namespace: parent.to_owned(),
        name: path.to_owned(),
        version: String::new(),
        fingerprint: fingerprint.to_owned(),
        origin: SENDER.to_owned(),
    }
}

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    fixture
        .state
        .content()
        .register(std::sync::Arc::new(astrum_format_raw::Raw));
    for name in ["source", "files"] {
        let body = json!({
            "name": name, "online": true,
            "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
            "raw": {"contentDisposition": "INLINE"}
        });
        let path = "/service/rest/v1/repositories/raw/hosted";
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
    }
    fixture
}

async fn connect(fixture: &Fixture) -> Client {
    let tokens = "/service/rest/internal/access-tokens";
    let wanted = json!({"name": "replication", "scope": "WRITE", "expiresInDays": 1});
    let made = fixture
        .call(Method::POST, tokens, Auth::Admin, Some(wanted))
        .await;
    let token = made.json()["token"].as_str().unwrap().to_owned();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = listener.local_addr().unwrap();
    let app = router(fixture.state.clone()).into_make_service_with_connect_info::<SocketAddr>();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    let remote = Remote {
        url: format!("http://{address}"),
        use_trust_store: false,
        allow_private_network: true,
        max_concurrency: 2,
        max_bytes_per_second: None,
    };
    let off = SsrfProtection {
        enabled: false,
        ..SsrfProtection::default()
    };
    let secrets = SecretBox::from_key([7; 32]);
    Client::new(
        &remote,
        &token,
        &Snapshot::default(),
        &secrets,
        &Guard::new(&off),
    )
    .unwrap()
    .with_instance(SENDER)
    .unwrap()
}

async fn upload(fixture: &Fixture, repo: &str, path: &str, bytes: &[u8]) -> StatusCode {
    let uri = format!("/repository/{repo}{path}");
    let body = Body::from(bytes.to_vec());
    fixture
        .send_body(Method::PUT, &uri, Auth::Admin, body, &[])
        .await
        .status
}

async fn stage(client: &Client, bytes: &[u8]) {
    let parts = vec![Ok(Bytes::copy_from_slice(bytes))];
    let sent = client
        .put_blob(
            "files",
            &sha(bytes),
            bytes.len() as u64,
            stream::iter(parts),
        )
        .await;
    sent.unwrap();
}

async fn prepared(fixture: &Fixture, client: &Client, path: &str) -> Bundle {
    let seed = format!("/seed-{}.txt", sha(path.as_bytes()));
    assert!(
        upload(fixture, "source", &seed, path.as_bytes())
            .await
            .is_success()
    );
    let mut named = support::bundle_of(fixture, &astrum_format_raw::Raw, "source", &seed).await;
    let parent = &path[..path.rfind('/').unwrap()];
    (named.namespace, named.name, named.version) =
        (parent.to_owned(), path.to_owned(), String::new());
    named.assets[0].path = path.to_owned();
    stage(client, path.as_bytes()).await;
    named
}

async fn number(fixture: &Fixture, sql: &str) -> i64 {
    fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap()
        .try_get::<i64>("", "n")
        .unwrap()
}

fn assert_clean(what: &str, outcome: &str) {
    assert!(!outcome.contains("deadlock"), "{what}: {outcome}");
    assert!(!outcome.contains("500"), "{what}: {outcome}");
}

#[tokio::test]
async fn replication_commits_tombstones_and_uploads_in_parallel_do_not_deadlock() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    let mut doomed = Vec::new();
    for number in 0..WIDTH {
        let path = format!("/old/{number}.txt");
        let bundle = prepared(&fixture, &client, &path).await;
        client.commit("files", &bundle).await.unwrap();
        doomed.push((path, fingerprint(&bundle)));
    }
    let mut arriving = Vec::new();
    for number in 0..WIDTH {
        arriving.push(prepared(&fixture, &client, &format!("/new/{number}.txt")).await);
    }
    let commits = arriving
        .iter()
        .map(|bundle| async { format!("{:?}", client.commit("files", bundle).await.map(|_| ())) });
    let tombstones = doomed.iter().map(|(path, print)| async {
        format!(
            "{:?}",
            client.tombstone("files", &tombstone(path, print)).await
        )
    });
    let uploads = (0..WIDTH).map(|number| {
        let fixture = &fixture;
        async move {
            let path = match number % 2 {
                0 => format!("/up/{number}.txt"),
                _ => format!("/new/{number}.txt"),
            };
            upload(fixture, "files", &path, b"uploaded")
                .await
                .to_string()
        }
    });
    let (committed, removed, uploaded) = tokio::time::timeout(
        Duration::from_secs(60),
        futures_util::future::join3(
            futures_util::future::join_all(commits),
            futures_util::future::join_all(tombstones),
            futures_util::future::join_all(uploads),
        ),
    )
    .await
    .expect("a writer never finished");
    for outcome in committed.iter().chain(&removed).chain(&uploaded) {
        assert_clean("a parallel writer", outcome);
    }
    let rows = number(
        &fixture,
        "SELECT count(*) AS n FROM asset WHERE repository = 'files'",
    )
    .await;
    let counted = number(
        &fixture,
        "SELECT assets AS n FROM repository_usage WHERE repository = 'files'",
    )
    .await;
    let sum = number(
        &fixture,
        "SELECT coalesce(sum(size), 0)::bigint AS n FROM asset WHERE repository = 'files'",
    )
    .await;
    let bytes = number(
        &fixture,
        "SELECT bytes AS n FROM repository_usage WHERE repository = 'files'",
    )
    .await;
    assert_eq!((counted, bytes), (rows, sum));
}

#[tokio::test]
async fn a_tombstone_takes_the_repository_lock_before_the_component() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    let bundle = prepared(&fixture, &client, "/held/a.txt").await;
    client.commit("files", &bundle).await.unwrap();
    let request = tombstone("/held/a.txt", &fingerprint(&bundle));
    let holder = fixture.state.db().conn().begin().await.unwrap();
    holder
        .execute_unprepared("SELECT 1 FROM repository WHERE name = 'files' FOR UPDATE")
        .await
        .unwrap();
    let removal = {
        let client = client.clone();
        tokio::spawn(async move { client.tombstone("files", &request).await.map(|_| ()) })
    };
    let deadline = Instant::now() + Duration::from_secs(5);
    while number(
        &fixture,
        "SELECT count(*) AS n FROM pg_locks l JOIN pg_stat_activity a ON a.pid = l.pid \
         WHERE NOT l.granted AND l.locktype <> 'advisory' AND a.datname = current_database()",
    )
    .await
        == 0
    {
        assert!(Instant::now() < deadline, "the tombstone never waited");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    holder
        .execute_unprepared("DELETE FROM component WHERE repository = 'files'")
        .await
        .unwrap();
    holder.commit().await.unwrap();
    let outcome = tokio::time::timeout(Duration::from_secs(30), removal)
        .await
        .expect("the tombstone never finished")
        .unwrap();
    assert_clean("the tombstone", &format!("{outcome:?}"));
}
