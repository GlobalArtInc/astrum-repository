mod support;
mod ui_support;

use axum::http::{Method, StatusCode};
use serde_json::{Value, json};
use support::{Auth, Fixture, Reply};
use ui_support::*;

const STORES: &str = "/service/rest/v1/blobstores";
const GROUP: &str = "/service/rest/v1/blobstores/group";

async fn file_store(fixture: &Fixture, name: &str) {
    let reply = fixture
        .call(
            Method::POST,
            &format!("{STORES}/file"),
            Auth::Admin,
            Some(json!({"name": name, "path": name})),
        )
        .await;
    assert_eq!(reply.status, StatusCode::NO_CONTENT, "{}", reply.text());
}

async fn send(fixture: &Fixture, method: Method, path: &str, body: Value) -> Reply {
    fixture.call(method, path, Auth::Admin, Some(body)).await
}

async fn stores() -> Fixture {
    let fixture = ready().await;
    for name in ["one", "two", "three"] {
        file_store(&fixture, name).await;
    }
    fixture
}

fn messages(reply: &Reply) -> String {
    reply.text()
}

#[tokio::test]
async fn groups_are_created_read_and_changed_like_the_documented_api() {
    let fixture = stores().await;
    let created = send(
        &fixture,
        Method::POST,
        GROUP,
        json!({"name": "pool", "members": ["one", "two"], "fillPolicy": "roundRobin", "softQuota": {"type": "spaceUsedQuota", "limit": 1000000}}),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED, "{}", created.text());
    let read = get(&fixture, &format!("{GROUP}/pool"), Auth::Admin).await;
    assert_eq!(read.status, StatusCode::OK);
    assert_eq!(
        read.json(),
        json!({"name": "pool", "softQuota": {"type": "spaceUsedQuota", "limit": 1000000}, "members": ["one", "two"], "fillPolicy": "roundRobin"})
    );
    let listed = get(&fixture, STORES, Auth::Admin).await.json();
    let pool = listed
        .as_array()
        .unwrap()
        .iter()
        .find(|store| store["name"] == "pool")
        .unwrap();
    assert_eq!(pool["type"], "Group");

    let changed = send(
        &fixture,
        Method::PUT,
        &format!("{GROUP}/pool"),
        json!({"members": ["one", "two", "three"], "fillPolicy": "writeToFirst"}),
    )
    .await;
    assert_eq!(changed.status, StatusCode::NO_CONTENT, "{}", changed.text());
    let read = get(&fixture, &format!("{GROUP}/pool"), Auth::Admin)
        .await
        .json();
    assert_eq!(read["members"], json!(["one", "two", "three"]));
    assert_eq!(read["fillPolicy"], "writeToFirst");
    assert_eq!(read["softQuota"], Value::Null);
    let defaulted = send(
        &fixture,
        Method::PUT,
        &format!("{GROUP}/pool"),
        json!({"members": ["one", "two", "three"]}),
    )
    .await;
    assert_eq!(defaulted.status, StatusCode::NO_CONTENT);

    assert_eq!(
        get(&fixture, &format!("{GROUP}/one"), Auth::Admin)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    assert_eq!(
        get(&fixture, &format!("{GROUP}/missing"), Auth::Admin)
            .await
            .status,
        StatusCode::NOT_FOUND
    );
    let not_a_group = send(
        &fixture,
        Method::PUT,
        &format!("{GROUP}/one"),
        json!({"members": ["two"]}),
    )
    .await;
    assert_eq!(not_a_group.status, StatusCode::BAD_REQUEST);
    let file_put = send(
        &fixture,
        Method::PUT,
        &format!("{STORES}/file/pool"),
        json!({"path": "pool"}),
    )
    .await;
    assert_eq!(file_put.status, StatusCode::BAD_REQUEST);

    let events = fixture
        .call(
            Method::GET,
            "/service/rest/internal/audit?domain=blobstore&context=pool",
            Auth::Admin,
            None,
        )
        .await
        .json();
    assert_eq!(names(&events, "type"), ["updated", "updated", "created"]);
    assert_eq!(events[2]["attributes"]["members"], json!(["one", "two"]));
}

#[tokio::test]
async fn group_members_must_be_free_plain_stores() {
    let fixture = stores().await;
    raw_hosted(&fixture, "files").await;
    let create = |body: Value| {
        let fixture = &fixture;
        async move { send(fixture, Method::POST, GROUP, body).await }
    };
    for (body, expected) in [
        (json!({"name": "g", "members": []}), "at least one member"),
        (json!({"name": "g"}), "at least one member"),
        (
            json!({"name": "g", "members": ["missing"]}),
            "no blob store named missing",
        ),
        (json!({"name": "g", "members": ["g"]}), "its own member"),
        (
            json!({"name": "g", "members": ["one", "one"]}),
            "more than once",
        ),
        (
            json!({"name": "g", "members": ["default"]}),
            "stores the repositories files",
        ),
        (
            json!({"name": "g", "members": ["one"], "fillPolicy": "random"}),
            "not a fill policy",
        ),
        (json!({"name": "", "members": ["one"]}), "must not be empty"),
        (
            json!({"name": "default", "members": ["one"]}),
            "used by another blob store",
        ),
    ] {
        let reply = create(body.clone()).await;
        assert_eq!(
            reply.status,
            StatusCode::BAD_REQUEST,
            "{body}: {}",
            reply.text()
        );
        assert!(
            messages(&reply).contains(expected),
            "{body}: {}",
            reply.text()
        );
    }
    assert_eq!(
        create(json!({"name": "first", "members": ["one"]}))
            .await
            .status,
        StatusCode::CREATED
    );
    let taken = create(json!({"name": "second", "members": ["one", "two"]})).await;
    assert!(
        messages(&taken).contains("already a member of the group first"),
        "{}",
        taken.text()
    );
    let nested = create(json!({"name": "second", "members": ["first"]})).await;
    assert!(
        messages(&nested).contains("cannot be a member of another group"),
        "{}",
        nested.text()
    );
    let cycle = send(
        &fixture,
        Method::PUT,
        &format!("{GROUP}/first"),
        json!({"members": ["first"]}),
    )
    .await;
    assert!(
        messages(&cycle).contains("its own member"),
        "{}",
        cycle.text()
    );

    let direct = fixture
        .call(
            Method::POST,
            "/service/rest/v1/repositories/raw/hosted",
            Auth::Admin,
            Some(json!({
                "name": "direct",
                "online": true,
                "storage": {"blobStoreName": "one", "strictContentTypeValidation": false, "writePolicy": "ALLOW"},
                "raw": {"contentDisposition": "INLINE"}
            })),
        )
        .await;
    assert_eq!(direct.status, StatusCode::BAD_REQUEST);
    assert!(
        direct.text().contains("member of the group"),
        "{}",
        direct.text()
    );

    let deleted = fixture
        .call(Method::DELETE, &format!("{STORES}/one"), Auth::Admin, None)
        .await;
    assert_eq!(deleted.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn members_holding_blobs_stay_in_their_group() {
    let fixture = stores().await;
    let created = send(
        &fixture,
        Method::POST,
        GROUP,
        json!({"name": "pool", "members": ["one", "two"], "fillPolicy": "writeToFirst"}),
    )
    .await;
    assert_eq!(created.status, StatusCode::CREATED);
    raw_hosted_in(&fixture, "pooled", "pool").await;
    put(&fixture, "/repository/pooled/a.txt", "stored in one").await;
    let read = get(&fixture, "/repository/pooled/a.txt", Auth::Admin).await;
    assert_eq!(read.text(), "stored in one");

    let losing = send(
        &fixture,
        Method::PUT,
        &format!("{GROUP}/pool"),
        json!({"members": ["two"]}),
    )
    .await;
    assert_eq!(losing.status, StatusCode::BAD_REQUEST);
    assert!(
        messages(&losing).contains("would lose them"),
        "{}",
        losing.text()
    );
    let keeping = send(
        &fixture,
        Method::PUT,
        &format!("{GROUP}/pool"),
        json!({"members": ["one"]}),
    )
    .await;
    assert_eq!(keeping.status, StatusCode::NO_CONTENT, "{}", keeping.text());
    let used = fixture
        .call(Method::DELETE, &format!("{STORES}/pool"), Auth::Admin, None)
        .await;
    assert_eq!(used.status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn group_changes_need_blob_store_permissions() {
    let fixture = stores().await;
    user_with(&fixture, "reader", &["as-blobstores-read"]).await;
    let body = json!({"name": "pool", "members": ["one"]});
    let refused = fixture
        .call(Method::POST, GROUP, user("reader"), Some(body.clone()))
        .await;
    assert_eq!(refused.status, StatusCode::FORBIDDEN);
    let anonymous = fixture
        .call(Method::POST, GROUP, Auth::None, Some(body.clone()))
        .await;
    assert_eq!(anonymous.status, StatusCode::UNAUTHORIZED);
    assert_eq!(
        send(&fixture, Method::POST, GROUP, body).await.status,
        StatusCode::CREATED
    );
    assert_eq!(
        get(&fixture, &format!("{GROUP}/pool"), user("reader"))
            .await
            .status,
        StatusCode::OK
    );
    let update = fixture
        .call(
            Method::PUT,
            &format!("{GROUP}/pool"),
            user("reader"),
            Some(json!({"members": ["two"]})),
        )
        .await;
    assert_eq!(update.status, StatusCode::FORBIDDEN);
}
