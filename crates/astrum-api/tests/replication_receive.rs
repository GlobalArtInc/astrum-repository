mod accounts;
mod support;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use accounts::{create_role, create_user};
use astrum_api::{AppState, router};
use astrum_blob::BlobId;
use astrum_outbound::Snapshot;
use astrum_replication::{Client, Remote};
use astrum_security::SecretBox;
use astrum_security::ssrf::{Guard, SsrfProtection};
use axum::body::Body;
use axum::http::{Method, StatusCode};
use bytes::Bytes;
use futures_util::stream;
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use support::{Auth, Fixture, Reply};
use tower::ServiceExt;

const CAPABILITIES: &str = "/service/rest/v1/replication/capabilities";
const TASKS: &str = "/service/rest/v1/tasks";
const PASSWORD: &str = "pw-pw-pw-1";
const STAGED: &str = "SELECT count(*) AS n FROM replication_staged_blob";
const EXPIRE: &str = "UPDATE replication_staged_blob SET expires_at = now() - interval '1 minute'";

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn user(name: &'static str) -> Auth<'static> {
    Auth::Basic(name, PASSWORD)
}

async fn repository(fixture: &Fixture, kind: &str, name: &str, extra: Value) {
    let mut body = json!({
        "name": name, "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
        "raw": {"contentDisposition": "INLINE"}
    });
    body.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    let path = format!("/service/rest/v1/repositories/raw/{kind}");
    let reply = fixture
        .call(Method::POST, &path, Auth::Admin, Some(body))
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn ready() -> Fixture {
    let fixture = Fixture::new().await;
    astrum_blob::bootstrap(fixture.state.db().conn(), fixture.data.path())
        .await
        .unwrap();
    let content = fixture.state.content();
    content.register(Arc::new(astrum_format_raw::Raw));
    repository(&fixture, "hosted", "files", json!({})).await;
    repository(&fixture, "hosted", "other", json!({})).await;
    let group = json!({"group": {"memberNames": ["files"]}});
    repository(&fixture, "group", "everything", group).await;
    let read = "as-repository-view-*-*-read";
    for (name, privileges) in [
        (
            "receiver",
            vec!["as-replication-receive", read, "as-repository-view-*-*-add"],
        ),
        ("reader", vec!["as-replication-receive", read]),
        ("blind", vec!["as-replication-receive"]),
        ("plain", vec!["as-repository-view-*-*-*"]),
    ] {
        create_role(&fixture, name, &privileges, &[]).await;
        create_user(&fixture, name, PASSWORD, &[name]).await;
    }
    fixture
}

async fn put_blob(f: &Fixture, auth: Auth<'_>, repo: &str, digest: &str, body: Body) -> Reply {
    let path = format!("/service/rest/v1/replication/{repo}/blobs/{digest}");
    f.send_body(Method::PUT, &path, auth, body, &[]).await
}

async fn stage(fixture: &Fixture, repo: &str, bytes: &'static [u8]) -> StatusCode {
    let body = Body::from(bytes);
    put_blob(fixture, user("receiver"), repo, &sha(bytes), body)
        .await
        .status
}

async fn ask(fixture: &Fixture, auth: Auth<'_>, repo: &str, digests: &[String]) -> Reply {
    let path = format!("/service/rest/v1/replication/{repo}/blobs/missing");
    let body = Some(json!({"sha256": digests}));
    fixture.call(Method::POST, &path, auth, body).await
}

async fn missing(fixture: &Fixture, repo: &str, digests: &[String]) -> Value {
    let reply = ask(fixture, user("receiver"), repo, digests).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    reply.json()["missing"].clone()
}

async fn upload_asset(fixture: &Fixture, repo: &str, bytes: &'static [u8]) {
    let path = format!("/repository/{repo}/{}.txt", sha(bytes));
    let body = Body::from(bytes);
    let reply = fixture
        .send_body(Method::PUT, &path, Auth::Admin, body, &[])
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

async fn execute(fixture: &Fixture, sql: &str) {
    let db = fixture.state.db().conn();
    db.execute(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap();
}

async fn count(fixture: &Fixture, sql: &str) -> i64 {
    let db = fixture.state.db().conn();
    let row = db
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap();
    row.try_get::<i64>("", "n").unwrap()
}

async fn blob_id(fixture: &Fixture, sql: &str) -> BlobId {
    let db = fixture.state.db().conn();
    let row = db
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap();
    row.try_get::<uuid::Uuid>("", "blob_id").unwrap().into()
}

async fn blobs_in_store(fixture: &Fixture) -> usize {
    let store = fixture.state.blob_stores().get("default").await.unwrap();
    store.list(None, 1000).await.unwrap().len()
}

async fn instance_on(state: &AppState) -> Value {
    use base64::Engine;
    let pair = format!("receiver:{PASSWORD}");
    let credentials = base64::engine::general_purpose::STANDARD.encode(pair);
    let request = axum::http::Request::builder()
        .uri(CAPABILITIES)
        .header("authorization", format!("Basic {credentials}"))
        .body(Body::empty())
        .unwrap();
    let response = router(state.clone()).oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = http_body_util::BodyExt::collect(response.into_body())
        .await
        .unwrap()
        .to_bytes();
    serde_json::from_slice::<Value>(&bytes).unwrap()["instanceId"].clone()
}

async fn second_server(fixture: &Fixture) -> AppState {
    let connection = fixture.test.connect().await;
    let key = SecretBox::from_key([7; 32]);
    AppState::load(connection, fixture.data.path().to_owned(), key)
        .await
        .unwrap()
}

#[tokio::test]
async fn capabilities_need_the_receive_privilege_and_the_instance_id_is_stable() {
    let fixture = ready().await;
    let call = async |auth: Auth<'_>| fixture.call(Method::GET, CAPABILITIES, auth, None).await;
    assert_eq!(call(Auth::None).await.status, StatusCode::UNAUTHORIZED);
    assert_eq!(call(user("plain")).await.status, StatusCode::FORBIDDEN);

    let reply = call(user("receiver")).await;
    assert_eq!(reply.status, StatusCode::OK, "{}", reply.text());
    let body = reply.json();
    assert_eq!(body["protocol"], 1);
    assert_eq!(body["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(body["formats"], json!(["raw"]));
    uuid::Uuid::parse_str(body["instanceId"].as_str().unwrap()).unwrap();
    let again = call(user("receiver")).await.json();
    assert_eq!(again["instanceId"], body["instanceId"]);
    let other = second_server(&fixture).await;
    assert_eq!(instance_on(&other).await, body["instanceId"]);
}

#[tokio::test]
async fn two_servers_asking_first_at_once_agree_on_one_instance_id() {
    let fixture = ready().await;
    let other = second_server(&fixture).await;
    let (first, second) = tokio::join!(instance_on(&fixture.state), instance_on(&other));
    assert_eq!(first, second);
    let sql = "SELECT count(*) AS n FROM setting WHERE key = 'replication.instanceId'";
    assert_eq!(count(&fixture, sql).await, 1);
}

#[tokio::test]
async fn missing_lists_what_no_asset_and_no_staged_blob_of_that_repository_covers() {
    let fixture = ready().await;
    upload_asset(&fixture, "files", b"an asset").await;
    assert_eq!(
        stage(&fixture, "files", b"staged").await,
        StatusCode::CREATED
    );
    let (asset, staged) = (sha(b"an asset"), sha(b"staged"));
    let (a, b) = (sha(b"absent a"), sha(b"absent b"));

    let all = [a.clone(), asset.clone(), staged.clone(), a.clone()];
    assert_eq!(missing(&fixture, "files", &all).await, json!([a]));
    let twice = [b.clone(), a.clone(), b.clone()];
    assert_eq!(missing(&fixture, "files", &twice).await, json!([b, a]));
    let both = [asset.clone(), staged.clone()];
    assert_eq!(
        missing(&fixture, "other", &both).await,
        json!([asset, staged])
    );

    let thousand: Vec<String> = (0..1000).map(|n| sha(n.to_string().as_bytes())).collect();
    assert_eq!(
        missing(&fixture, "files", &thousand)
            .await
            .as_array()
            .unwrap()
            .len(),
        1000
    );
    let mut too_many = thousand;
    too_many.push(sha(b"one more"));
    let refused = ask(&fixture, user("receiver"), "files", &too_many).await;
    assert_eq!(refused.status, StatusCode::BAD_REQUEST);
    assert!(
        refused.text().contains("at most 1000"),
        "{}",
        refused.text()
    );
    let empty = ask(&fixture, user("receiver"), "files", &[]).await;
    assert_eq!(empty.status, StatusCode::BAD_REQUEST);
    for bad in [sha(b"x").to_uppercase(), "abc".to_owned(), "g".repeat(64)] {
        let refused = ask(&fixture, user("receiver"), "files", &[sha(b"ok"), bad]).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST);
        assert!(refused.text().contains("position 1"), "{}", refused.text());
    }
}

#[tokio::test]
async fn rights_decide_and_an_unreadable_repository_is_not_revealed() {
    let fixture = ready().await;
    let digest = sha(b"blob");
    let put = async |auth: Auth<'_>, repo: &str| {
        put_blob(&fixture, auth, repo, &digest, Body::from("blob")).await
    };
    let ask = async |auth: Auth<'_>, repo: &str| {
        ask(&fixture, auth, repo, std::slice::from_ref(&digest)).await
    };
    assert_eq!(
        ask(Auth::None, "files").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        put(Auth::None, "files").await.status,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        ask(user("plain"), "files").await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        put(user("plain"), "files").await.status,
        StatusCode::FORBIDDEN
    );

    let asked = (
        ask(user("blind"), "files").await,
        ask(user("blind"), "nowhere").await,
    );
    let put_to = (
        put(user("blind"), "files").await,
        put(user("blind"), "nowhere").await,
    );
    for (known, unknown) in [asked, put_to] {
        assert_eq!(known.status, StatusCode::NOT_FOUND);
        assert_eq!(unknown.status, StatusCode::NOT_FOUND);
        let same = known.text().replace("files", "X") == unknown.text().replace("nowhere", "X");
        assert!(same, "{} / {}", known.text(), unknown.text());
    }
    assert_eq!(ask(user("reader"), "files").await.status, StatusCode::OK);
    assert_eq!(
        put(user("reader"), "files").await.status,
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        put(user("reader"), "nowhere").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(count(&fixture, STAGED).await, 0);
}

#[tokio::test]
async fn a_blob_is_staged_once_under_a_hold_and_only_the_right_digest_stays() {
    let fixture = ready().await;
    let before = blobs_in_store(&fixture).await;
    let bytes: &'static [u8] = b"hello replication";
    let digest = sha(bytes);
    let parts: Vec<std::io::Result<Bytes>> = [&bytes[..6], &bytes[6..12], &bytes[12..]]
        .map(|part| Ok(Bytes::from_static(part)))
        .into();
    let body = Body::from_stream(stream::iter(parts));
    let created = put_blob(&fixture, user("receiver"), "files", &digest, body).await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    assert_eq!(blobs_in_store(&fixture).await, before + 1);
    assert_eq!(
        missing(&fixture, "files", std::slice::from_ref(&digest)).await,
        json!([])
    );
    let held = format!(
        "SELECT count(*) AS n FROM replication_staged_blob s JOIN blob_hold h \
         ON h.blob_id = s.blob_id AND h.blob_store = s.blob_store \
         WHERE s.repository = 'files' AND s.sha256 = '{digest}' AND s.size = {} \
         AND s.uploaded_by = 'receiver' AND s.expires_at > now() + interval '23 hours' \
         AND s.expires_at < now() + interval '25 hours' AND h.held_until > s.expires_at",
        bytes.len()
    );
    assert_eq!(count(&fixture, &held).await, 1);

    assert_eq!(stage(&fixture, "files", bytes).await, StatusCode::OK);
    upload_asset(&fixture, "other", b"named by an asset").await;
    let settled = blobs_in_store(&fixture).await;
    assert_eq!(settled, before + 2);
    assert_eq!(
        stage(&fixture, "other", b"named by an asset").await,
        StatusCode::OK
    );
    assert_eq!(blobs_in_store(&fixture).await, settled);
    assert_eq!(count(&fixture, STAGED).await, 1);
    assert_eq!(
        count(&fixture, "SELECT count(*) AS n FROM blob_hold").await,
        1
    );

    let wrong = put_blob(
        &fixture,
        user("receiver"),
        "files",
        &sha(b"other"),
        Body::from(bytes),
    )
    .await;
    assert_eq!(
        wrong.status,
        StatusCode::UNPROCESSABLE_ENTITY,
        "{}",
        wrong.text()
    );
    assert_eq!(wrong.json()["code"], "digest_mismatch");
    for bad in ["not-a-digest".to_owned(), digest.to_uppercase()] {
        let reply = put_blob(&fixture, user("receiver"), "files", &bad, Body::from(bytes)).await;
        assert_eq!(reply.status, StatusCode::BAD_REQUEST, "{bad}");
    }
    let group = put_blob(
        &fixture,
        user("receiver"),
        "everything",
        &digest,
        Body::from(bytes),
    )
    .await;
    assert_eq!(group.status, StatusCode::BAD_REQUEST);
    assert_eq!(group.json()["code"], "not_hosted");
    assert_eq!(blobs_in_store(&fixture).await, settled);
}

async fn task(fixture: &Fixture, body: Value) -> String {
    let created = fixture
        .call(Method::POST, TASKS, Auth::Admin, Some(body))
        .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    created.json()["id"].as_str().unwrap().to_owned()
}

async fn run(fixture: &Fixture, id: &str) -> Value {
    let path = format!("{TASKS}/{id}/run");
    let started = fixture.call(Method::POST, &path, Auth::Admin, None).await;
    assert_eq!(started.status, StatusCode::NO_CONTENT, "{}", started.text());
    fixture.state.tasks().run_pending().await.unwrap();
    let runs = format!("/service/rest/internal/ui/tasks/{id}/runs");
    let listed = fixture.call(Method::GET, &runs, Auth::Admin, None).await;
    let run = listed.json()[0].clone();
    assert_eq!(run["state"], "done", "{run}");
    run
}

fn age(fixture: &Fixture, id: BlobId) {
    let content = fixture.data.path().join("blobs/default/content");
    let file = std::fs::File::options()
        .write(true)
        .open(content.join(id.location()))
        .unwrap();
    file.set_modified(SystemTime::now() - Duration::from_secs(3 * 86_400))
        .unwrap();
}

fn purge() -> Value {
    json!({"type": "security.purge-expired", "name": "Expire"})
}

#[tokio::test]
async fn a_staged_blob_survives_compaction_and_goes_with_its_expiry() {
    let fixture = ready().await;
    upload_asset(&fixture, "files", b"kept by an asset").await;
    assert_eq!(
        stage(&fixture, "files", b"staged bytes").await,
        StatusCode::CREATED
    );
    let store = fixture.state.blob_stores().get("default").await.unwrap();
    let loose = store
        .put(astrum_blob::content_of(b"loose".to_vec()))
        .await
        .unwrap()
        .id;
    let staged = blob_id(&fixture, "SELECT blob_id FROM replication_staged_blob").await;
    let named = blob_id(&fixture, "SELECT blob_id FROM asset").await;
    for id in [loose, staged, named] {
        age(&fixture, id);
    }
    let compact = json!({"type": "blobstore.compact", "name": "Compact", "properties": {"blobstoreName": "*", "blobsOlderThan": "1"}});
    run(&fixture, &task(&fixture, compact).await).await;
    assert!(
        store.exists(staged).await.unwrap(),
        "the hold keeps a staged blob"
    );
    assert!(store.exists(named).await.unwrap());
    assert!(!store.exists(loose).await.unwrap());

    let purge = task(&fixture, purge()).await;
    run(&fixture, &purge).await;
    assert!(store.exists(staged).await.unwrap(), "not expired yet");
    execute(&fixture, EXPIRE).await;
    let lapsed = missing(&fixture, "files", &[sha(b"staged bytes")]).await;
    assert_eq!(
        lapsed.as_array().unwrap().len(),
        1,
        "an expired blob is not staged"
    );
    let expired = run(&fixture, &purge).await;
    assert!(
        expired["message"]
            .as_str()
            .unwrap()
            .contains("1 unfinished upload"),
        "{expired}"
    );
    assert!(!store.exists(staged).await.unwrap(), "nothing names it");
    assert!(store.exists(named).await.unwrap(), "an asset names it");
    assert_eq!(count(&fixture, STAGED).await, 0);
    assert_eq!(
        count(&fixture, "SELECT count(*) AS n FROM blob_hold").await,
        0
    );
}

#[tokio::test]
async fn an_expired_staged_blob_is_replaced_by_a_new_upload() {
    let fixture = ready().await;
    let staged = "SELECT blob_id FROM replication_staged_blob";
    assert_eq!(
        stage(&fixture, "files", b"again").await,
        StatusCode::CREATED
    );
    let first = blob_id(&fixture, staged).await;
    execute(&fixture, EXPIRE).await;
    assert_eq!(
        stage(&fixture, "files", b"again").await,
        StatusCode::CREATED
    );
    let second = blob_id(&fixture, staged).await;
    assert_ne!(first, second);
    run(&fixture, &task(&fixture, purge()).await).await;
    let store = fixture.state.blob_stores().get("default").await.unwrap();
    assert!(!store.exists(first).await.unwrap());
    assert!(store.exists(second).await.unwrap());
    assert_eq!(
        missing(&fixture, "files", &[sha(b"again")]).await,
        json!([])
    );
}

#[tokio::test]
async fn the_real_client_asks_and_uploads_against_the_receiver() {
    let fixture = ready().await;
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
    let snapshot = Snapshot::default();
    let client = Client::new(&remote, &token, &snapshot, &secrets, &Guard::new(&off)).unwrap();

    let capabilities = client.check().await.unwrap();
    assert_eq!(capabilities.formats, ["raw"]);
    assert_eq!(capabilities.version, env!("CARGO_PKG_VERSION"));
    let bytes: &'static [u8] = b"sent by the real client";
    let digest = [sha(bytes)];
    assert_eq!(client.missing("files", &digest).await.unwrap(), digest);
    let parts: Vec<std::io::Result<Bytes>> = vec![
        Ok(Bytes::from_static(&bytes[..9])),
        Ok(Bytes::from_static(&bytes[9..])),
    ];
    let size = bytes.len() as u64;
    let sent = client.put_blob("files", &digest[0], size, stream::iter(parts));
    sent.await.unwrap();
    assert!(client.missing("files", &digest).await.unwrap().is_empty());
}
