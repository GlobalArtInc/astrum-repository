mod accounts;
mod support;

use std::net::SocketAddr;

use accounts::{create_role, create_user};
use astrum_api::router;
use astrum_content::transfer::{AssetStatus, Bundle, Outcome, Status, fingerprint};
use astrum_outbound::Snapshot;
use astrum_replication::protocol::{InventoryKey, Tombstone, component_key};
use astrum_replication::{Client, Error, Remote};
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

const PASSWORD: &str = "pw-pw-pw-1";
const SENDER: &str = "0b8e6f0a-0000-4000-8000-0000000000a1";
const THIRD: &str = "0b8e6f0a-0000-4000-8000-0000000000b2";
const STAGED: &str = "SELECT count(*) AS n FROM replication_staged_blob";

fn sha(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

fn user(name: &'static str) -> Auth<'static> {
    Auth::Basic(name, PASSWORD)
}

fn key(path: &str) -> String {
    let parent = &path[..path.rfind('/').unwrap()];
    component_key(if parent.is_empty() { "/" } else { parent }, path, "")
}

fn url(repo: &str, tail: &str) -> String {
    format!("/service/rest/v1/replication/{repo}/{tail}")
}

async fn repository(fixture: &Fixture, name: &str, policy: &str) {
    let body = json!({
        "name": name, "online": true,
        "storage": {"blobStoreName": "default", "strictContentTypeValidation": false, "writePolicy": policy},
        "raw": {"contentDisposition": "INLINE"}
    });
    let path = "/service/rest/v1/repositories/raw/hosted";
    let reply = fixture
        .call(Method::POST, path, Auth::Admin, Some(body))
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
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
    for (name, policy) in [
        ("source", "ALLOW"),
        ("files", "ALLOW"),
        ("once", "ALLOW_ONCE"),
    ] {
        repository(&fixture, name, policy).await;
    }
    let view = |action: &str| format!("as-repository-view-*-*-{action}");
    let receive = "as-replication-receive".to_owned();
    for (name, privileges) in [
        ("adder", vec![receive.clone(), view("read"), view("add")]),
        ("reader", vec![receive.clone(), view("read")]),
        (
            "remover",
            vec![receive.clone(), view("read"), view("delete")],
        ),
        ("blind", vec![receive.clone()]),
        ("plain", vec![view("*")]),
    ] {
        let privileges: Vec<&str> = privileges.iter().map(String::as_str).collect();
        create_role(&fixture, name, &privileges, &[]).await;
        create_user(&fixture, name, PASSWORD, &[name]).await;
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

async fn upload(fixture: &Fixture, repo: &str, path: &str, bytes: &[u8]) {
    let uri = format!("/repository/{repo}{path}");
    let body = Body::from(bytes.to_vec());
    let reply = fixture
        .send_body(Method::PUT, &uri, Auth::Admin, body, &[])
        .await;
    assert!(reply.status.is_success(), "{}", reply.text());
}

async fn download(fixture: &Fixture, repo: &str, path: &str) -> Reply {
    let uri = format!("/repository/{repo}{path}");
    fixture.call(Method::GET, &uri, Auth::Admin, None).await
}

async fn bundle(fixture: &Fixture, path: &str) -> Bundle {
    support::bundle_of(fixture, &astrum_format_raw::Raw, "source", path).await
}

async fn stage(client: &Client, repo: &str, bytes: &[u8]) {
    let parts = vec![Ok(Bytes::copy_from_slice(bytes))];
    let sent = client
        .put_blob(repo, &sha(bytes), bytes.len() as u64, stream::iter(parts))
        .await;
    sent.unwrap();
}

async fn push(
    fixture: &Fixture,
    client: &Client,
    repo: &str,
    path: &str,
    bytes: &[u8],
) -> Result<Outcome, Error> {
    upload(fixture, "source", path, bytes).await;
    let bundle = bundle(fixture, path).await;
    stage(client, repo, bytes).await;
    client.commit(repo, &bundle).await
}

async fn count(fixture: &Fixture, sql: &str) -> i64 {
    let row = fixture
        .state
        .db()
        .conn()
        .query_one(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap()
        .unwrap();
    row.try_get::<i64>("", "n").unwrap()
}

async fn commit_raw(fixture: &Fixture, auth: Auth<'_>, repo: &str, bundle: &Bundle) -> Reply {
    let body = Some(serde_json::to_value(bundle).unwrap());
    fixture
        .call(Method::POST, &url(repo, "components"), auth, body)
        .await
}

fn tombstone(path: &str, fingerprint: &str, origin: &str) -> Tombstone {
    let parent = &path[..path.rfind('/').unwrap()];
    Tombstone {
        namespace: if parent.is_empty() { "/" } else { parent }.to_owned(),
        name: path.to_owned(),
        version: String::new(),
        fingerprint: fingerprint.to_owned(),
        origin: origin.to_owned(),
    }
}

fn permanent(error: Error) -> (u16, String) {
    match error {
        Error::Permanent { status, code, .. } => (status, code),
        other => panic!("{other:?}"),
    }
}

#[tokio::test]
async fn a_staged_component_commits_with_its_origin_and_commits_again_unchanged() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    let first = push(&fixture, &client, "files", "/d/a.txt", b"alpha")
        .await
        .unwrap();
    assert_eq!(first.status, Status::Copied);
    assert_eq!(first.bytes_copied, 5);
    assert_eq!(download(&fixture, "files", "/d/a.txt").await.body, b"alpha");
    assert_eq!(count(&fixture, STAGED).await, 0);
    let held = "SELECT count(*) AS n FROM blob_hold WHERE held_until > now()";
    assert_eq!(count(&fixture, held).await, 0);
    let marked = format!(
        "SELECT count(*) AS n FROM component WHERE repository = 'files' \
         AND attributes #>> '{{replication,origin}}' = '{SENDER}' \
         AND attributes #>> '{{replication,via}}' = 'replication'"
    );
    assert_eq!(count(&fixture, &marked).await, 1);
    let audited = "SELECT count(*) AS n FROM audit_event WHERE domain = 'repository.content' \
         AND type = 'replicated' AND initiator LIKE 'admin%' AND attributes ->> 'via' = 'replication'";
    assert_eq!(count(&fixture, audited).await, 1);

    let again = client
        .commit("files", &bundle(&fixture, "/d/a.txt").await)
        .await
        .unwrap();
    assert_eq!(again.status, Status::Unchanged);
    assert_eq!(again.component_ids, first.component_ids);
    assert_eq!(count(&fixture, audited).await, 1);

    upload(&fixture, "source", "/d/b.txt", b"bravo").await;
    let mut named = bundle(&fixture, "/d/b.txt").await;
    named.origin = Some(THIRD.to_owned());
    stage(&client, "files", b"bravo").await;
    client.commit("files", &named).await.unwrap();
    let third = marked.replace(SENDER, THIRD);
    assert_eq!(count(&fixture, &third).await, 1);
}

#[tokio::test]
async fn a_bundle_naming_blobs_nobody_has_gets_the_list_of_them() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    upload(&fixture, "source", "/d/a.txt", b"alpha").await;
    let mut wanted = bundle(&fixture, "/d/a.txt").await;
    stage(&client, "files", b"alpha").await;
    let mut ghost = wanted.assets[0].clone();
    ghost.path = "/d/ghost.txt".to_owned();
    ghost.sha256 = sha(b"ghost");
    wanted.assets.push(ghost);
    let asked = commit_raw(
        &fixture,
        Auth::Admin,
        "files",
        &Bundle {
            origin: Some(SENDER.to_owned()),
            ..wanted.clone()
        },
    )
    .await;
    assert_eq!(asked.status, StatusCode::CONFLICT, "{}", asked.text());
    assert_eq!(asked.json()["code"], "blobs_missing");
    assert_eq!(asked.json()["missing"], json!([sha(b"ghost")]));
    match client.commit("files", &wanted).await {
        Err(Error::BlobsMissing(list)) => assert_eq!(list, [sha(b"ghost")]),
        other => panic!("{other:?}"),
    }
    assert_eq!(
        count(
            &fixture,
            "SELECT count(*) AS n FROM asset WHERE repository = 'files'"
        )
        .await,
        0
    );

    let mut anonymous = wanted.clone();
    anonymous.assets.pop();
    let reply = commit_raw(&fixture, Auth::Admin, "files", &anonymous).await;
    assert_eq!(reply.status, StatusCode::BAD_REQUEST);
    assert_eq!(reply.json()["code"], "origin_missing");
    anonymous.origin = Some("not-a-uuid".to_owned());
    let reply = commit_raw(&fixture, Auth::Admin, "files", &anonymous).await;
    assert_eq!(reply.json()["code"], "bad_instance");
}

#[tokio::test]
async fn a_blob_an_asset_already_holds_is_copied_without_staging() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    upload(&fixture, "files", "/d/x.txt", b"shared").await;
    upload(&fixture, "source", "/d/y.txt", b"shared").await;
    let outcome = client
        .commit("files", &bundle(&fixture, "/d/y.txt").await)
        .await
        .unwrap();
    assert_eq!(outcome.status, Status::Copied);
    assert_eq!(count(&fixture, STAGED).await, 0);
    let blobs = "SELECT count(DISTINCT blob_id) AS n FROM asset WHERE repository = 'files'";
    assert_eq!(count(&fixture, blobs).await, 2);
    assert_eq!(
        download(&fixture, "files", "/d/y.txt").await.body,
        b"shared"
    );
}

#[tokio::test]
async fn conflicts_follow_the_write_policy_and_replacing_needs_edit() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    push(&fixture, &client, "once", "/d/a.txt", b"alpha")
        .await
        .unwrap();
    let refused = push(&fixture, &client, "once", "/d/a.txt", b"beta")
        .await
        .unwrap_err();
    let (status, code) = permanent(refused);
    assert_eq!((status, code.as_str()), (409, "conflict"));
    assert_eq!(download(&fixture, "once", "/d/a.txt").await.body, b"alpha");

    push(&fixture, &client, "files", "/d/a.txt", b"alpha")
        .await
        .unwrap();
    upload(&fixture, "source", "/d/a.txt", b"beta").await;
    let mut changed = bundle(&fixture, "/d/a.txt").await;
    changed.origin = Some(SENDER.to_owned());
    stage(&client, "files", b"beta").await;
    let without_edit = commit_raw(&fixture, user("adder"), "files", &changed).await;
    assert_eq!(
        without_edit.status,
        StatusCode::CONFLICT,
        "{}",
        without_edit.text()
    );
    assert_eq!(download(&fixture, "files", "/d/a.txt").await.body, b"alpha");
    let replaced = client.commit("files", &changed).await.unwrap();
    assert_eq!(replaced.status, Status::Copied);
    assert_eq!(replaced.assets[0].status, AssetStatus::Replaced);
    assert_eq!(download(&fixture, "files", "/d/a.txt").await.body, b"beta");
}

#[tokio::test]
async fn rights_decide_for_commit_tombstone_and_inventory() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    let pushed = push(&fixture, &client, "files", "/d/a.txt", b"alpha")
        .await
        .unwrap();
    assert_eq!(pushed.status, Status::Copied);
    let mut wanted = bundle(&fixture, "/d/a.txt").await;
    wanted.origin = Some(SENDER.to_owned());
    let fp = fingerprint(&wanted);
    let removal = serde_json::to_value(tombstone("/d/a.txt", &fp, SENDER)).unwrap();
    let compare = json!({"items": [{"key": key("/d/a.txt"), "fingerprint": fp}]});
    let post = serde_json::to_value(&wanted).unwrap();
    let call = async |method: Method,
                      who: Option<&'static str>,
                      repo: &str,
                      tail: &str,
                      body: Option<Value>| {
        let auth = who.map_or(Auth::None, user);
        fixture
            .call(method, &url(repo, tail), auth, body)
            .await
            .status
            .as_u16()
    };
    for (who, repo, commit, delete, read) in [
        (None, "files", 401, 401, 401),
        (Some("plain"), "files", 403, 403, 403),
        (Some("blind"), "files", 404, 404, 404),
        (Some("blind"), "nowhere", 404, 404, 404),
        (Some("reader"), "files", 403, 403, 200),
        (Some("adder"), "files", 200, 403, 200),
        (Some("remover"), "files", 403, 204, 200),
    ] {
        let committed = call(Method::POST, who, repo, "components", Some(post.clone())).await;
        assert_eq!(committed, commit, "commit by {who:?} into {repo}");
        let compared = call(
            Method::POST,
            who,
            repo,
            "inventory/compare",
            Some(compare.clone()),
        )
        .await;
        assert_eq!(compared, read, "compare by {who:?}");
        let listed = call(Method::GET, who, repo, "inventory", None).await;
        assert_eq!(listed, read, "inventory by {who:?}");
        let deleted = call(
            Method::DELETE,
            who,
            repo,
            "components",
            Some(removal.clone()),
        )
        .await;
        assert_eq!(deleted, delete, "tombstone by {who:?}");
    }
}

#[tokio::test]
async fn a_tombstone_deletes_only_what_the_sender_replicated_and_has_not_seen_change() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    push(&fixture, &client, "files", "/d/a.txt", b"alpha")
        .await
        .unwrap();
    let fp = fingerprint(&bundle(&fixture, "/d/a.txt").await);
    let other = fingerprint(&{
        upload(&fixture, "source", "/d/o.txt", b"other").await;
        bundle(&fixture, "/d/o.txt").await
    });
    let refuse = async |request: Tombstone| {
        permanent(client.tombstone("files", &request).await.unwrap_err())
    };
    assert_eq!(
        refuse(tombstone("/d/a.txt", &other, SENDER)).await,
        (409, "fingerprint_mismatch".to_owned())
    );
    assert_eq!(
        refuse(tombstone("/d/a.txt", &fp, THIRD)).await,
        (409, "origin_mismatch".to_owned())
    );
    assert_eq!(
        download(&fixture, "files", "/d/a.txt").await.status,
        StatusCode::OK
    );
    client
        .tombstone("files", &tombstone("/d/a.txt", &fp, SENDER))
        .await
        .unwrap();
    assert_eq!(
        download(&fixture, "files", "/d/a.txt").await.status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        count(
            &fixture,
            "SELECT count(*) AS n FROM component WHERE repository = 'files'"
        )
        .await,
        0
    );
    assert_eq!(refuse(tombstone("/d/a.txt", &fp, SENDER)).await.0, 404);
    let audited = "SELECT count(*) AS n FROM audit_event WHERE domain = 'repository.content' \
         AND type = 'deleted' AND attributes ->> 'via' = 'replication'";
    assert_eq!(count(&fixture, audited).await, 1);

    push(&fixture, &client, "files", "/d/b.txt", b"one")
        .await
        .unwrap();
    upload(&fixture, "files", "/d/b.txt", b"two").await;
    let republished = "SELECT count(*) AS n FROM component WHERE repository = 'files' AND name = '/d/b.txt' \
         AND NOT attributes ? 'replication'";
    assert_eq!(count(&fixture, republished).await, 1);
    upload(&fixture, "source", "/d/b.txt", b"two").await;
    let current = fingerprint(&bundle(&fixture, "/d/b.txt").await);
    assert_eq!(
        refuse(tombstone("/d/b.txt", &current, SENDER)).await.1,
        "origin_mismatch"
    );

    upload(&fixture, "files", "/d/c.txt", b"local").await;
    upload(&fixture, "source", "/d/c.txt", b"local").await;
    let local = fingerprint(&bundle(&fixture, "/d/c.txt").await);
    assert_eq!(
        refuse(tombstone("/d/c.txt", &local, SENDER)).await.1,
        "origin_mismatch"
    );
}

#[tokio::test]
async fn compare_names_absent_and_differing_keys_and_equal_content_has_equal_fingerprints() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    push(&fixture, &client, "files", "/d/a.txt", b"alpha")
        .await
        .unwrap();
    push(&fixture, &client, "files", "/d/b.txt", b"bravo")
        .await
        .unwrap();
    upload(&fixture, "files", "/d/c.txt", b"charlie").await;
    let mut fingerprints = Vec::new();
    for (path, bytes) in [
        ("/d/a.txt", &b"alpha"[..]),
        ("/d/b.txt", b"bravo"),
        ("/d/c.txt", b"charlie"),
    ] {
        upload(&fixture, "source", path, bytes).await;
        fingerprints.push(fingerprint(&bundle(&fixture, path).await));
    }
    let item = |path: &str, fingerprint: &str| InventoryKey {
        key: key(path),
        fingerprint: fingerprint.to_owned(),
    };
    let items = [
        item("/d/a.txt", &fingerprints[0]),
        item("/d/b.txt", &fingerprints[0]),
        item("/d/c.txt", &fingerprints[2]),
        item("/d/missing.txt", &fingerprints[0]),
        item("/d/missing.txt", &fingerprints[0]),
    ];
    let reply = client.inventory_compare("files", &items).await.unwrap();
    assert_eq!(reply.differing, [key("/d/b.txt"), key("/d/missing.txt")]);

    upload(&fixture, "once", "/d/a.txt", b"alpha").await;
    let local = client.inventory("once", None, 10).await.unwrap();
    let replicated = client.inventory("files", None, 1).await.unwrap();
    assert_eq!(local.items[0].fingerprint, replicated.items[0].fingerprint);
    assert_eq!(local.items[0].fingerprint, fingerprints[0]);

    let call = async |body: Value| {
        fixture
            .call(
                Method::POST,
                &url("files", "inventory/compare"),
                Auth::Admin,
                Some(body),
            )
            .await
    };
    let many: Vec<Value> = (0..1001)
        .map(|n| json!({"key": format!("n{n}@1"), "fingerprint": fingerprints[0]}))
        .collect();
    assert_eq!(
        call(json!({"items": many})).await.status,
        StatusCode::BAD_REQUEST
    );
    let bad_key = json!({"items": [{"key": "no-at-sign", "fingerprint": fingerprints[0]}]});
    assert_eq!(call(bad_key).await.status, StatusCode::BAD_REQUEST);
    let bad_print = json!({"items": [{"key": key("/d/a.txt"), "fingerprint": "xyz"}]});
    assert_eq!(call(bad_print).await.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_inventory_pages_in_key_order_and_shows_origins() {
    let fixture = ready().await;
    let client = connect(&fixture).await;
    for path in ["/d/b.txt", "/e/c@x.txt", "/z.txt"] {
        push(&fixture, &client, "files", path, path.as_bytes())
            .await
            .unwrap();
    }
    for path in ["/d/a.txt", "/d/d.txt"] {
        upload(&fixture, "files", path, path.as_bytes()).await;
    }
    let names = ["/d/a.txt", "/d/b.txt", "/d/d.txt", "/e/c@x.txt", "/z.txt"];
    let mut expected: Vec<String> = names.iter().map(|path| key(path)).collect();
    expected.sort();
    let mut seen = Vec::new();
    let mut after: Option<String> = None;
    let mut pages = 0;
    loop {
        let page = client
            .inventory("files", after.as_deref(), 2)
            .await
            .unwrap();
        assert!(page.items.len() <= 2);
        for entry in &page.items {
            let replicated = ["/d/b.txt", "/e/c@x.txt", "/z.txt"]
                .iter()
                .any(|path| key(path) == entry.key);
            assert_eq!(
                entry.origin.as_deref(),
                replicated.then_some(SENDER),
                "{}",
                entry.key
            );
            seen.push(entry.key.clone());
        }
        pages += 1;
        match page.next {
            Some(next) => after = Some(next),
            None => break,
        }
    }
    assert_eq!((seen, pages), (expected.clone(), 3));
    let rest = client
        .inventory("files", Some(&expected[2]), 10)
        .await
        .unwrap();
    let rest: Vec<&str> = rest.items.iter().map(|entry| entry.key.as_str()).collect();
    assert_eq!(rest, [expected[3].as_str(), expected[4].as_str()]);
    let call = async |query: &str| {
        fixture
            .call(
                Method::GET,
                &url("files", &format!("inventory?{query}")),
                Auth::Admin,
                None,
            )
            .await
            .status
    };
    for bad in ["limit=0", "limit=1001", "limit=x", "after=nonsense"] {
        assert_eq!(call(bad).await, StatusCode::BAD_REQUEST, "{bad}");
    }
}
