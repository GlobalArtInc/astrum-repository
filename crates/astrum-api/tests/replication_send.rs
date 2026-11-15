mod accounts;
mod support;

use std::sync::Arc;

use accounts::{create_role, create_user};
use axum::body::Body;
use axum::http::{Method, StatusCode, header};
use sea_orm::{ConnectionTrait, DbBackend, Statement};
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};
use support::{Auth, Fixture, Reply};

const PASSWORD: &str = "pw-pw-pw-1";
const BASE: &str = "/service/rest/v1/replication";
const PATH: &str = "/docs/a.txt";

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
    fixture
        .state
        .content()
        .register(Arc::new(astrum_format_raw::Raw));
    repository(&fixture, "hosted", "files", json!({})).await;
    repository(&fixture, "hosted", "other", json!({})).await;
    let group = json!({"group": {"memberNames": ["files"]}});
    repository(&fixture, "group", "everything", group).await;
    let read = "as-repository-view-*-*-read";
    let manager = [
        "as-replication-read",
        "as-replication-create",
        "as-replication-update",
        "as-replication-delete",
        read,
    ];
    for (name, privileges) in [
        ("puller", vec!["as-replication-pull", read]),
        ("receiver", vec!["as-replication-receive", read]),
        ("blind", vec!["as-replication-pull"]),
        ("manager", manager.to_vec()),
        ("plain", vec!["as-repository-view-*-*-*"]),
    ] {
        create_role(&fixture, name, &privileges, &[]).await;
        create_user(&fixture, name, PASSWORD, &[name]).await;
    }
    fixture
}

async fn upload(fixture: &Fixture, repo: &str, path: &str, bytes: &'static [u8]) {
    let uri = format!("/repository/{repo}{path}");
    let reply = fixture
        .send_body(Method::PUT, &uri, Auth::Admin, Body::from(bytes), &[])
        .await;
    assert_eq!(reply.status, StatusCode::CREATED, "{}", reply.text());
}

fn bundle_uri(repo: &str, name: &str) -> String {
    format!("{BASE}/{repo}/bundle?namespace=%2Fdocs&name={name}&version=")
}

async fn get(fixture: &Fixture, auth: Auth<'_>, uri: &str) -> Reply {
    fixture.call(Method::GET, uri, auth, None).await
}

async fn execute(fixture: &Fixture, sql: &str) {
    let db = fixture.state.db().conn();
    db.execute(Statement::from_string(DbBackend::Postgres, sql))
        .await
        .unwrap();
}

#[tokio::test]
async fn bundle_and_blob_need_the_pull_privilege_and_read_on_the_repository() {
    let fixture = ready().await;
    upload(&fixture, "files", PATH, b"alpha").await;
    let bundle = bundle_uri("files", "%2Fdocs%2Fa.txt");
    let blob = format!("{BASE}/files/blobs/{}", sha(b"alpha"));
    for uri in [&bundle, &blob] {
        for (who, auth, wanted) in [
            ("anonymous", Auth::None, StatusCode::UNAUTHORIZED),
            ("admin", Auth::Admin, StatusCode::OK),
            ("puller", user("puller"), StatusCode::OK),
            ("receiver", user("receiver"), StatusCode::FORBIDDEN),
            ("plain", user("plain"), StatusCode::FORBIDDEN),
            ("manager", user("manager"), StatusCode::FORBIDDEN),
            ("blind", user("blind"), StatusCode::NOT_FOUND),
        ] {
            let reply = get(&fixture, auth, uri).await;
            assert_eq!(reply.status, wanted, "{who} on {uri}: {}", reply.text());
        }
    }
    let nowhere = get(&fixture, user("blind"), &bundle_uri("nowhere", "x")).await;
    let known = get(&fixture, user("blind"), &bundle).await;
    let same = known.text().replace("files", "X") == nowhere.text().replace("nowhere", "X");
    assert!(same, "{} / {}", known.text(), nowhere.text());
}

#[tokio::test]
async fn a_blob_is_served_only_for_components_of_the_repository_and_a_group_is_refused() {
    let fixture = ready().await;
    upload(&fixture, "files", PATH, b"alpha").await;
    upload(&fixture, "other", "/docs/b.txt", b"beta").await;
    let puller = user("puller");
    let foreign = format!("{BASE}/files/blobs/{}", sha(b"beta"));
    assert_eq!(
        get(&fixture, puller, &foreign).await.status,
        StatusCode::NOT_FOUND
    );
    execute(
        &fixture,
        "UPDATE asset SET component_id = NULL WHERE path = '/docs/a.txt'",
    )
    .await;
    let loose = format!("{BASE}/files/blobs/{}", sha(b"alpha"));
    assert_eq!(
        get(&fixture, user("puller"), &loose).await.status,
        StatusCode::NOT_FOUND
    );
    let malformed = get(&fixture, user("puller"), &format!("{BASE}/files/blobs/ABC")).await;
    assert_eq!(malformed.status, StatusCode::BAD_REQUEST);
    for uri in [
        bundle_uri("everything", "%2Fdocs%2Fa.txt"),
        format!("{BASE}/everything/blobs/{}", sha(b"alpha")),
    ] {
        let refused = get(&fixture, user("puller"), &uri).await;
        assert_eq!(refused.status, StatusCode::BAD_REQUEST, "{uri}");
        assert_eq!(refused.json()["code"], "not_hosted");
    }
}

#[tokio::test]
async fn a_bundle_names_the_origin_and_a_blob_streams_with_its_length() {
    let fixture = ready().await;
    upload(&fixture, "files", PATH, b"alpha").await;
    let uri = bundle_uri("files", "%2Fdocs%2Fa.txt");
    let own = get(&fixture, user("puller"), &format!("{BASE}/capabilities")).await;
    assert_eq!(own.status, StatusCode::OK, "{}", own.text());
    let bundle = get(&fixture, user("puller"), &uri).await.json();
    assert_eq!(bundle["origin"], own.json()["instanceId"]);
    assert_eq!(bundle["format"], "raw");
    assert_eq!(bundle["assets"][0]["sha256"], sha(b"alpha"));
    let third = "3f1c2d9e-0000-4000-8000-000000000001";
    execute(
        &fixture,
        &format!(
            "UPDATE component SET attributes = jsonb_build_object('replication', \
             jsonb_build_object('origin', '{third}'))"
        ),
    )
    .await;
    let marked = get(&fixture, user("puller"), &uri).await.json();
    assert_eq!(marked["origin"], third);

    let blob = format!("{BASE}/files/blobs/{}", sha(b"alpha"));
    let streamed = get(&fixture, user("puller"), &blob).await;
    assert_eq!(streamed.status, StatusCode::OK);
    assert_eq!(streamed.body, b"alpha");
    assert_eq!(streamed.headers[header::CONTENT_LENGTH], "5");
    let unknown = get(&fixture, user("puller"), &bundle_uri("files", "%2Fnone")).await;
    assert_eq!(unknown.status, StatusCode::NOT_FOUND);
    assert_eq!(unknown.json()["code"], "component_not_found");
    let nameless = get(
        &fixture,
        user("puller"),
        &format!("{BASE}/files/bundle?namespace=%2Fdocs"),
    )
    .await;
    assert_eq!(nameless.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn the_inventory_routes_and_the_handshake_accept_either_privilege() {
    let fixture = ready().await;
    upload(&fixture, "files", PATH, b"alpha").await;
    let compare = format!("{BASE}/files/inventory/compare");
    for who in ["puller", "receiver"] {
        let inventory = get(&fixture, user(who), &format!("{BASE}/files/inventory")).await;
        assert_eq!(inventory.status, StatusCode::OK, "{who}");
        assert_eq!(inventory.json()["items"].as_array().unwrap().len(), 1);
        let asked = fixture
            .call(
                Method::POST,
                &compare,
                user(who),
                Some(json!({"items": []})),
            )
            .await;
        assert_eq!(asked.status, StatusCode::OK, "{who}");
        let shaken = get(&fixture, user(who), &format!("{BASE}/capabilities")).await;
        assert_eq!(shaken.status, StatusCode::OK, "{who}");
    }
    for who in ["plain", "manager"] {
        let inventory = get(&fixture, user(who), &format!("{BASE}/files/inventory")).await;
        assert_eq!(inventory.status, StatusCode::FORBIDDEN, "{who}");
        let shaken = get(&fixture, user(who), &format!("{BASE}/capabilities")).await;
        assert_eq!(shaken.status, StatusCode::FORBIDDEN, "{who}");
    }
    let missing = fixture
        .call(
            Method::POST,
            &format!("{BASE}/files/blobs/missing"),
            user("puller"),
            Some(json!({"sha256": [sha(b"alpha")]})),
        )
        .await;
    assert_eq!(missing.status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn a_role_that_reads_through_a_content_selector_gets_no_bundle_and_no_blob() {
    let fixture = ready().await;
    upload(&fixture, "files", PATH, b"alpha").await;
    let steps = [
        (
            "/service/rest/v1/security/content-selectors",
            json!({"name": "docs-only", "description": "", "expression": "format == \"raw\" && path =^ \"/docs/\""}),
        ),
        (
            "/service/rest/v1/security/privileges/repository-content-selector",
            json!({"name": "docs-read", "description": "", "actions": ["READ", "BROWSE"], "format": "raw", "repository": "files", "contentSelector": "docs-only"}),
        ),
    ];
    for (path, body) in steps {
        let reply = fixture
            .call(Method::POST, path, Auth::Admin, Some(body))
            .await;
        assert!(reply.status.is_success(), "{path}: {}", reply.text());
    }
    create_role(
        &fixture,
        "selected",
        &["as-replication-pull", "docs-read"],
        &[],
    )
    .await;
    create_user(&fixture, "selected", PASSWORD, &["selected"]).await;
    let direct = get(&fixture, user("selected"), "/repository/files/docs/a.txt").await;
    assert_eq!(direct.status, StatusCode::OK, "{}", direct.text());
    let blob = format!("{BASE}/files/blobs/{}", sha(b"alpha"));
    for uri in [bundle_uri("files", "%2Fdocs%2Fa.txt"), blob] {
        let refused = get(&fixture, user("selected"), &uri).await;
        assert_eq!(refused.status, StatusCode::FORBIDDEN, "{uri}");
    }
}
